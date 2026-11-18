//! Překlad obvodu na netlist a událostní simulátor.
//!
//! Každá součástka s výstupem má zpoždění (hradla 1 takt, zdroje 0). Změna
//! výstupu se naplánuje do fronty událostí; po jejím provedení se přepočítají
//! sítě (propojené vodiče) a vyhodnotí součástky, které z nich čtou.
//! Díky tomu jde ukázat šíření signálu krok po kroku, hazardy i kmitání.

use crate::LogicValue;
use crate::circuit::{Circuit, CompId, ComponentKind, Point, WireId};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Zpoždění hradla v taktech simulace.
pub const GATE_DELAY: u64 = 1;

#[derive(Clone, Copy, Debug)]
struct Event {
    time: u64,
    seq: u64,
    comp: usize,
    value: LogicValue,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.seq == other.seq
    }
}
impl Eq for Event {}
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Event {
    /// Obrácené pořadí: `BinaryHeap` pak vrací nejdřívější událost.
    fn cmp(&self, other: &Self) -> Ordering {
        (other.time, other.seq).cmp(&(self.time, self.seq))
    }
}

/// Jednoduché sjednocování množin pro hledání sítí.
struct UnionFind(Vec<usize>);

impl UnionFind {
    fn new(n: usize) -> Self {
        Self((0..n).collect())
    }
    fn find(&mut self, mut i: usize) -> usize {
        while self.0[i] != i {
            self.0[i] = self.0[self.0[i]];
            i = self.0[i];
        }
        i
    }
    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.0[ra] = rb;
        }
    }
}

/// Výsledek přeložení obvodu: které vývody a vodiče patří do které sítě.
#[derive(Clone, Debug, Default)]
pub struct Netlist {
    pub net_count: usize,
    /// Síť každého vodiče (ve stejném pořadí jako `Circuit::wires`).
    pub wire_net: HashMap<WireId, usize>,
    /// Sítě vstupů každé součástky (pořadí jako `Circuit::components`).
    pub input_nets: Vec<Vec<usize>>,
    /// Síť výstupu každé součástky.
    pub output_net: Vec<Option<usize>>,
}

impl Netlist {
    pub fn build(circuit: &Circuit) -> Self {
        // Body, ve kterých se může vodič s něčím spojit.
        let mut points: Vec<Point> = circuit.wires.iter().flat_map(|w| [w.a, w.b]).collect();
        for c in &circuit.components {
            points.extend(c.pins());
        }
        points.sort();
        points.dedup();
        let point_index: HashMap<Point, usize> =
            points.iter().enumerate().map(|(i, p)| (*p, i)).collect();

        let w = circuit.wires.len();
        let mut uf = UnionFind::new(w + points.len());
        for (wi, wire) in circuit.wires.iter().enumerate() {
            for (pi, p) in points.iter().enumerate() {
                if wire.contains(*p) {
                    uf.union(wi, w + pi);
                }
            }
        }

        // Očíslování sítí 0..n.
        let mut net_of_root: HashMap<usize, usize> = HashMap::new();
        let mut net = |uf: &mut UnionFind, node: usize| {
            let root = uf.find(node);
            let next = net_of_root.len();
            *net_of_root.entry(root).or_insert(next)
        };

        let wire_net = circuit
            .wires
            .iter()
            .enumerate()
            .map(|(wi, wire)| (wire.id, net(&mut uf, wi)))
            .collect();
        let mut input_nets = Vec::with_capacity(circuit.components.len());
        let mut output_net = Vec::with_capacity(circuit.components.len());
        for c in &circuit.components {
            input_nets.push(
                c.input_pins()
                    .into_iter()
                    .map(|p| net(&mut uf, w + point_index[&p]))
                    .collect(),
            );
            output_net.push(c.output_pin().map(|p| net(&mut uf, w + point_index[&p])));
        }
        Self {
            net_count: net_of_root.len(),
            wire_net,
            input_nets,
            output_net,
        }
    }
}

/// Událostní simulátor jednoho obvodu.
#[derive(Clone, Debug)]
pub struct Simulator {
    netlist: Netlist,
    ids: Vec<CompId>,
    index: HashMap<CompId, usize>,
    kinds: Vec<ComponentKind>,
    sources: Vec<Option<LogicValue>>,
    net_drivers: Vec<Vec<usize>>,
    net_readers: Vec<Vec<usize>>,
    outputs: Vec<LogicValue>,
    pending: Vec<Option<LogicValue>>,
    net_values: Vec<LogicValue>,
    queue: BinaryHeap<Event>,
    seq: u64,
    time: u64,
    oscillating: bool,
}

impl Simulator {
    /// Postaví simulátor pro obvod. Pokud je zadán předchozí simulátor,
    /// převezme z něj výstupy součástek se stejným id – paměťové obvody
    /// (třeba klopný obvod z hradel) si tak po úpravě schématu pamatují stav.
    pub fn new(circuit: &Circuit, previous: Option<&Simulator>) -> Self {
        let netlist = Netlist::build(circuit);
        let n = circuit.components.len();
        let mut net_drivers = vec![Vec::new(); netlist.net_count];
        let mut net_readers = vec![Vec::new(); netlist.net_count];
        for i in 0..n {
            if let Some(net) = netlist.output_net[i] {
                net_drivers[net].push(i);
            }
            for &net in &netlist.input_nets[i] {
                if !net_readers[net].contains(&i) {
                    net_readers[net].push(i);
                }
            }
        }
        let ids: Vec<CompId> = circuit.components.iter().map(|c| c.id).collect();
        let outputs = ids
            .iter()
            .map(|id| {
                previous
                    .and_then(|p| p.output(*id))
                    .unwrap_or(LogicValue::X)
            })
            .collect();
        let mut sim = Self {
            index: ids.iter().enumerate().map(|(i, id)| (*id, i)).collect(),
            ids,
            kinds: circuit.components.iter().map(|c| c.kind).collect(),
            sources: circuit
                .components
                .iter()
                .map(|c| c.source_value())
                .collect(),
            net_drivers,
            net_readers,
            outputs,
            pending: vec![None; n],
            net_values: vec![LogicValue::Z; netlist.net_count],
            netlist,
            queue: BinaryHeap::new(),
            seq: 0,
            time: previous.map_or(0, |p| p.time),
            oscillating: false,
        };
        for net in 0..sim.net_values.len() {
            sim.net_values[net] = sim.resolve_net(net);
        }
        for i in 0..n {
            sim.evaluate(i);
        }
        sim
    }

    fn delay(&self, i: usize) -> u64 {
        match self.kinds[i] {
            ComponentKind::Gate { .. } => GATE_DELAY,
            _ => 0,
        }
    }

    fn resolve_net(&self, net: usize) -> LogicValue {
        self.net_drivers[net]
            .iter()
            .map(|&d| self.outputs[d])
            .fold(LogicValue::Z, LogicValue::resolve)
    }

    fn input_values(&self, i: usize) -> Vec<LogicValue> {
        self.netlist.input_nets[i]
            .iter()
            .map(|&n| self.net_values[n])
            .collect()
    }

    /// Vyhodnotí součástku a případnou změnu výstupu naplánuje.
    fn evaluate(&mut self, i: usize) {
        let new = match self.kinds[i] {
            ComponentKind::Gate { gate, .. } => gate.eval(&self.input_values(i)),
            ComponentKind::Led => return,
            _ => self.sources[i].unwrap_or(LogicValue::X),
        };
        let expected = self.pending[i].unwrap_or(self.outputs[i]);
        if new != expected {
            self.seq += 1;
            self.queue.push(Event {
                time: self.time + self.delay(i),
                seq: self.seq,
                comp: i,
                value: new,
            });
            self.pending[i] = Some(new);
        }
    }

    /// Změní hodnotu zdroje (přepínač, tlačítko, konstanta).
    pub fn set_source(&mut self, id: CompId, value: LogicValue) {
        if let Some(&i) = self.index.get(&id) {
            self.sources[i] = Some(value);
            self.evaluate(i);
        }
    }

    /// Provede všechny události nejbližšího taktu. Vrací počet událostí (0 = klid).
    pub fn step(&mut self) -> usize {
        let Some(first) = self.queue.peek() else {
            return 0;
        };
        self.time = first.time;
        let mut changed_nets = Vec::new();
        let mut count = 0;
        while self.queue.peek().is_some_and(|e| e.time == self.time) {
            let e = self.queue.pop().expect("fronta není prázdná");
            count += 1;
            if self.pending[e.comp] == Some(e.value) {
                self.pending[e.comp] = None;
            }
            if self.outputs[e.comp] != e.value {
                self.outputs[e.comp] = e.value;
                if let Some(net) = self.netlist.output_net[e.comp] {
                    changed_nets.push(net);
                }
            }
        }
        let mut to_eval = Vec::new();
        for net in changed_nets {
            let v = self.resolve_net(net);
            if v != self.net_values[net] {
                self.net_values[net] = v;
                to_eval.extend(self.net_readers[net].iter().copied());
            }
        }
        to_eval.sort_unstable();
        to_eval.dedup();
        for i in to_eval {
            self.evaluate(i);
        }
        count
    }

    /// Simuluje, dokud se obvod neustálí, nejvýše však `budget` událostí.
    /// Vrací `true`, pokud je obvod v klidu.
    pub fn run(&mut self, budget: usize) -> bool {
        let mut done = 0;
        while !self.queue.is_empty() {
            if done >= budget {
                self.oscillating = true;
                return false;
            }
            done += self.step();
        }
        self.oscillating = false;
        true
    }

    /// Je ve frontě ještě nějaká událost?
    pub fn is_stable(&self) -> bool {
        self.queue.is_empty()
    }

    /// Obvod nepřestal měnit stav ani po vyčerpání limitu událostí.
    pub fn is_oscillating(&self) -> bool {
        self.oscillating
    }

    /// Aktuální čas simulace v taktech.
    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn netlist(&self) -> &Netlist {
        &self.netlist
    }

    pub fn wire_value(&self, id: WireId) -> LogicValue {
        self.netlist
            .wire_net
            .get(&id)
            .map_or(LogicValue::Z, |&n| self.net_values[n])
    }

    /// Hodnota výstupu součástky.
    pub fn output(&self, id: CompId) -> Option<LogicValue> {
        let &i = self.index.get(&id)?;
        self.netlist.output_net[i].map(|_| self.outputs[i])
    }

    /// Hodnoty na vstupních vývodech součástky.
    pub fn inputs(&self, id: CompId) -> Vec<LogicValue> {
        self.index
            .get(&id)
            .map_or_else(Vec::new, |&i| self.input_values(i))
    }

    /// Hodnota sítě připojené k výstupu (může se lišit od výstupu při kolizi).
    pub fn output_net_value(&self, id: CompId) -> Option<LogicValue> {
        let &i = self.index.get(&id)?;
        self.netlist.output_net[i].map(|n| self.net_values[n])
    }

    pub fn component_ids(&self) -> &[CompId] {
        &self.ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GateKind;
    use crate::LogicValue::*;

    /// Přepínač na (0,0) → hradla v řadě → LED. Vrací (obvod, id přepínače, id LED).
    fn chain(gates: &[GateKind]) -> (Circuit, CompId, CompId) {
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let mut x = 2; // výstup přepínače
        for &g in gates {
            let id = c.add_component(ComponentKind::gate(g), Point::new(x + 4, 0));
            c.add_wire(Point::new(x, 0), Point::new(x + 1, 0));
            x = c.component(id).unwrap().output_pin().unwrap().x;
        }
        let led = c.add_component(ComponentKind::Led, Point::new(x + 3, 0));
        c.add_wire(Point::new(x, 0), Point::new(x + 1, 0));
        (c, sw, led)
    }

    fn led(sim: &Simulator, id: CompId) -> LogicValue {
        sim.inputs(id)[0]
    }

    #[test]
    fn inverter_chain() {
        let (c, sw, l) = chain(&[GateKind::Not]);
        let mut sim = Simulator::new(&c, None);
        assert!(sim.run(1000));
        assert_eq!(led(&sim, l), One);
        sim.set_source(sw, One);
        assert!(sim.run(1000));
        assert_eq!(led(&sim, l), Zero);
    }

    #[test]
    fn propagation_takes_one_tick_per_gate() {
        let (c, sw, l) = chain(&[GateKind::Not, GateKind::Not, GateKind::Not]);
        let mut sim = Simulator::new(&c, None);
        sim.run(1000);
        assert_eq!(led(&sim, l), One);
        let t0 = sim.time();
        sim.set_source(sw, One);
        let mut ticks = 0;
        while led(&sim, l) == One {
            assert!(sim.step() > 0, "signál musí dorazit");
            ticks = sim.time() - t0;
        }
        assert_eq!(ticks, 3, "tři hradla = tři takty");
    }

    #[test]
    fn unconnected_input_is_undefined() {
        let mut c = Circuit::default();
        let g = c.add_component(ComponentKind::gate(GateKind::And), Point::new(0, 0));
        let mut sim = Simulator::new(&c, None);
        sim.run(100);
        assert_eq!(sim.output(g), Some(X));
    }

    /// Klopný obvod RS ze dvou hradel NOR.
    fn sr_latch() -> (Circuit, CompId, CompId, CompId) {
        let mut c = Circuit::default();
        let s = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let r = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 10));
        // NOR1 (výstup Qn) vstupy (7,-1) a (7,1), výstup (13,0)
        // NOR2 (výstup Q)  vstupy (7,9) a (7,11), výstup (13,10)
        c.add_component(ComponentKind::gate(GateKind::Nor), Point::new(10, 0));
        let q = c.add_component(ComponentKind::gate(GateKind::Nor), Point::new(10, 10));
        // S → NOR1 horní vstup
        c.add_wire(Point::new(2, 0), Point::new(4, 0));
        c.add_wire(Point::new(4, 0), Point::new(4, -1));
        c.add_wire(Point::new(4, -1), Point::new(7, -1));
        // R → NOR2 dolní vstup
        c.add_wire(Point::new(2, 10), Point::new(4, 10));
        c.add_wire(Point::new(4, 10), Point::new(4, 11));
        c.add_wire(Point::new(4, 11), Point::new(7, 11));
        // Qn (13,0) → NOR2 horní vstup (7,9)
        c.add_wire(Point::new(13, 0), Point::new(15, 0));
        c.add_wire(Point::new(15, 0), Point::new(15, 5));
        c.add_wire(Point::new(15, 5), Point::new(6, 5));
        c.add_wire(Point::new(6, 5), Point::new(6, 9));
        c.add_wire(Point::new(6, 9), Point::new(7, 9));
        // Q (13,10) → NOR1 dolní vstup (7,1)
        c.add_wire(Point::new(13, 10), Point::new(16, 10));
        c.add_wire(Point::new(16, 10), Point::new(16, 3));
        c.add_wire(Point::new(16, 3), Point::new(5, 3));
        c.add_wire(Point::new(5, 3), Point::new(5, 1));
        c.add_wire(Point::new(5, 1), Point::new(7, 1));
        (c, s, r, q)
    }

    #[test]
    fn sr_latch_remembers_state() {
        let (c, s, r, q) = sr_latch();
        let mut sim = Simulator::new(&c, None);
        sim.run(1000);
        assert_eq!(sim.output(q), Some(X), "bez nastavení je stav neurčitý");
        sim.set_source(s, One);
        assert!(sim.run(1000));
        assert_eq!(sim.output(q), Some(One));
        sim.set_source(s, Zero);
        assert!(sim.run(1000));
        assert_eq!(sim.output(q), Some(One), "obvod si pamatuje 1");
        sim.set_source(r, One);
        sim.run(1000);
        sim.set_source(r, Zero);
        sim.run(1000);
        assert_eq!(sim.output(q), Some(Zero), "po R si pamatuje 0");

        // Přestavění simulátoru (úprava schématu) stav zachová.
        let mut rebuilt = Simulator::new(&c, Some(&sim));
        assert!(rebuilt.run(1000));
        assert_eq!(rebuilt.output(q), Some(Zero));
    }

    #[test]
    fn ring_oscillator_is_detected() {
        // NAND se zpětnou vazbou: při EN = 1 kmitá.
        let mut c = Circuit::default();
        let en = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, -1));
        let g = c.add_component(ComponentKind::gate(GateKind::Nand), Point::new(10, 0));
        c.add_wire(Point::new(2, -1), Point::new(7, -1));
        c.add_wire(Point::new(13, 0), Point::new(13, 3));
        c.add_wire(Point::new(13, 3), Point::new(5, 3));
        c.add_wire(Point::new(5, 3), Point::new(5, 1));
        c.add_wire(Point::new(5, 1), Point::new(7, 1));
        let mut sim = Simulator::new(&c, None);
        assert!(sim.run(1000));
        assert_eq!(sim.output(g), Some(One));
        assert!(!sim.is_oscillating());
        sim.set_source(en, One);
        assert!(!sim.run(1000));
        assert!(sim.is_oscillating());
        // Vypnutím se obvod uklidní.
        sim.set_source(en, Zero);
        assert!(sim.run(1000));
        assert_eq!(sim.output(g), Some(One));
    }

    #[test]
    fn two_drivers_collide() {
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 4));
        let l = c.add_component(ComponentKind::Led, Point::new(8, 0));
        c.add_wire(Point::new(2, 0), Point::new(6, 0));
        c.add_wire(Point::new(2, 4), Point::new(4, 4));
        c.add_wire(Point::new(4, 4), Point::new(4, 0)); // odbočka „T“ do prvního vodiče
        let mut sim = Simulator::new(&c, None);
        sim.run(100);
        assert_eq!(led(&sim, l), X);
    }

    #[test]
    fn crossing_wires_do_not_connect() {
        let mut c = Circuit::default();
        let a = c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        let l = c.add_component(ComponentKind::Led, Point::new(12, 0));
        c.add_wire(Point::new(2, 0), Point::new(10, 0));
        // Svislý vodič přes střed vodorovného – jen křížení.
        let w = c.add_wire(Point::new(5, -3), Point::new(5, 3)).unwrap();
        let mut sim = Simulator::new(&c, None);
        sim.run(100);
        assert_eq!(led(&sim, l), One);
        assert_eq!(sim.wire_value(w), Z, "nepřipojený vodič");
        let _ = a;
    }
}
