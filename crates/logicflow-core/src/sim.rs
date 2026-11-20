//! Překlad obvodu na netlist a událostní simulátor.
//!
//! Každá součástka s výstupem má zpoždění (hradla a klopné obvody 1 takt,
//! zdroje 0). Změna výstupu se naplánuje do fronty událostí; po jejím provedení
//! se přepočítají sítě (propojené vodiče) a vyhodnotí součástky, které z nich
//! čtou. Díky tomu jde ukázat šíření signálu krok po kroku, hazardy i kmitání.

use crate::LogicValue;
use crate::circuit::{Circuit, CompId, ComponentKind, FlipFlopKind, Point, WireId};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Zpoždění hradla v taktech simulace.
pub const GATE_DELAY: u64 = 1;

#[derive(Clone, Copy, Debug)]
struct Event {
    time: u64,
    seq: u64,
    /// Globální index výstupu.
    out: usize,
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
    /// Síť každého vodiče.
    pub wire_net: HashMap<WireId, usize>,
    /// Sítě vstupů každé součástky (pořadí jako `Circuit::components`).
    pub input_nets: Vec<Vec<usize>>,
    /// Sítě výstupů každé součástky.
    pub output_nets: Vec<Vec<usize>>,
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
        let mut output_nets = Vec::with_capacity(circuit.components.len());
        for c in &circuit.components {
            let mut nets_of = |pins: Vec<Point>| -> Vec<usize> {
                pins.into_iter()
                    .map(|p| net(&mut uf, w + point_index[&p]))
                    .collect()
            };
            input_nets.push(nets_of(c.input_pins()));
            output_nets.push(nets_of(c.output_pins()));
        }
        Self {
            net_count: net_of_root.len(),
            wire_net,
            input_nets,
            output_nets,
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
    /// Začátek výstupů každé součástky v `outputs`.
    out_start: Vec<usize>,
    /// Vlastník každého výstupu: (součástka, pořadí výstupu).
    out_owner: Vec<(usize, usize)>,
    net_drivers: Vec<Vec<usize>>,
    net_readers: Vec<Vec<usize>>,
    outputs: Vec<LogicValue>,
    pending: Vec<Option<LogicValue>>,
    net_values: Vec<LogicValue>,
    /// Vnitřní stav klopných obvodů (Q) a poslední hodnota hodinového vstupu.
    ff_q: Vec<LogicValue>,
    ff_clock: Vec<LogicValue>,
    /// Vnitřní stav sekvenčních funkčních bloků (bity čítače, registru…).
    reg: Vec<Vec<LogicValue>>,
    queue: BinaryHeap<Event>,
    seq: u64,
    time: u64,
    oscillating: bool,
}

impl Simulator {
    /// Postaví simulátor pro obvod. Pokud je zadán předchozí simulátor,
    /// převezme z něj výstupy a stav součástek se stejným id – paměťové obvody
    /// si tak po úpravě schématu pamatují stav.
    pub fn new(circuit: &Circuit, previous: Option<&Simulator>) -> Self {
        let netlist = Netlist::build(circuit);
        let n = circuit.components.len();
        let mut out_start = Vec::with_capacity(n);
        let mut out_owner = Vec::new();
        for (i, outs) in netlist.output_nets.iter().enumerate() {
            out_start.push(out_owner.len());
            out_owner.extend((0..outs.len()).map(|k| (i, k)));
        }
        let mut net_drivers = vec![Vec::new(); netlist.net_count];
        let mut net_readers = vec![Vec::new(); netlist.net_count];
        for (i, &start) in out_start.iter().enumerate() {
            for (k, &net) in netlist.output_nets[i].iter().enumerate() {
                net_drivers[net].push(start + k);
            }
            for &net in &netlist.input_nets[i] {
                if !net_readers[net].contains(&i) {
                    net_readers[net].push(i);
                }
            }
        }
        let ids: Vec<CompId> = circuit.components.iter().map(|c| c.id).collect();
        let outputs = out_owner
            .iter()
            .map(|&(i, k)| {
                previous
                    .and_then(|p| p.outputs(ids[i]).get(k).copied())
                    .unwrap_or(LogicValue::X)
            })
            .collect();
        let ff_q = circuit
            .components
            .iter()
            .map(|c| match c.kind {
                ComponentKind::FlipFlop { initial, .. } => previous
                    .and_then(|p| p.flip_flop_state(c.id))
                    .unwrap_or(initial.into()),
                _ => LogicValue::X,
            })
            .collect();
        let ff_clock = ids
            .iter()
            .map(|id| {
                previous
                    .and_then(|p| p.index.get(id).map(|&j| p.ff_clock[j]))
                    .unwrap_or(LogicValue::X)
            })
            .collect();
        let reg = circuit
            .components
            .iter()
            .map(|c| match c.kind {
                ComponentKind::Block { block } if block.is_sequential() => previous
                    .and_then(|p| p.block_state(c.id))
                    .filter(|s| s.len() == block.state_bits())
                    .unwrap_or_else(|| vec![LogicValue::Zero; block.state_bits()]),
                _ => Vec::new(),
            })
            .collect();
        let total_outputs = out_owner.len();
        let mut sim = Self {
            index: ids.iter().enumerate().map(|(i, id)| (*id, i)).collect(),
            ids,
            kinds: circuit.components.iter().map(|c| c.kind).collect(),
            sources: circuit
                .components
                .iter()
                .map(|c| c.source_value())
                .collect(),
            out_start,
            out_owner,
            net_drivers,
            net_readers,
            outputs,
            pending: vec![None; total_outputs],
            net_values: vec![LogicValue::Z; netlist.net_count],
            ff_q,
            ff_clock,
            reg,
            netlist,
            queue: BinaryHeap::new(),
            seq: 0,
            time: previous.map_or(0, |p| p.time),
            oscillating: false,
        };
        // Hodnoty zdrojů (např. hodin) převezme z předchozího simulátoru.
        if let Some(p) = previous {
            for (i, id) in sim.ids.iter().enumerate() {
                if matches!(sim.kinds[i], ComponentKind::Clock { .. })
                    && let Some(v) = p.output(*id)
                    && v != LogicValue::X
                {
                    sim.sources[i] = Some(v);
                }
            }
        }
        for net in 0..sim.net_values.len() {
            sim.net_values[net] = sim.resolve_net(net);
        }
        for i in 0..n {
            if let ComponentKind::Block { block } = sim.kinds[i]
                && block.is_sequential()
            {
                let out = block.outputs_from_state(&sim.reg[i]);
                sim.schedule(i, out);
            } else if sim.kinds[i].is_sequential() {
                // Klopné obvody jen vystaví uložený stav; jejich vstupy se teprve ustalují.
                let q = sim.ff_q[i];
                sim.schedule(i, vec![q, q.not()]);
            } else {
                sim.evaluate(i);
            }
        }
        sim
    }

    fn delay(&self, i: usize) -> u64 {
        match self.kinds[i] {
            ComponentKind::Gate { .. }
            | ComponentKind::FlipFlop { .. }
            | ComponentKind::Block { .. } => GATE_DELAY,
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

    /// Nový stav klopného obvodu podle vstupů. Mění uložený stav i poslední hodnotu hodin.
    fn step_flip_flop(&mut self, i: usize, ff: FlipFlopKind, falling: bool) -> LogicValue {
        use LogicValue::{One, X, Zero};
        let ins: Vec<LogicValue> = self
            .input_values(i)
            .into_iter()
            .map(LogicValue::as_input)
            .collect();
        let q = self.ff_q[i];
        let new_q = match ff {
            FlipFlopKind::Rs => match (ins[0], ins[1]) {
                (Zero, Zero) => q,
                (One, Zero) => One,
                (Zero, One) => Zero,
                // S = R = 1 je zakázaný stav.
                _ => X,
            },
            FlipFlopKind::DLatch => match ins[1] {
                One => ins[0],
                Zero => q,
                _ if q == ins[0] => q,
                _ => X,
            },
            FlipFlopKind::D | FlipFlopKind::Jk | FlipFlopKind::T => {
                let clk = ins[1];
                let prev = self.ff_clock[i];
                self.ff_clock[i] = clk;
                let edge = if falling {
                    prev == One && clk == Zero
                } else {
                    prev == Zero && clk == One
                };
                if !edge {
                    q
                } else {
                    match ff {
                        FlipFlopKind::D => ins[0],
                        FlipFlopKind::Jk => match (ins[0], ins[2]) {
                            (Zero, Zero) => q,
                            (One, Zero) => One,
                            (Zero, One) => Zero,
                            (One, One) => q.not(),
                            _ => X,
                        },
                        _ => match ins[0] {
                            Zero => q,
                            One => q.not(),
                            _ => X,
                        },
                    }
                }
            }
        };
        self.ff_q[i] = new_q;
        new_q
    }

    /// Vyhodnotí součástku a případné změny výstupů naplánuje.
    fn evaluate(&mut self, i: usize) {
        let new: Vec<LogicValue> = match self.kinds[i] {
            ComponentKind::Gate { gate, .. } => vec![gate.eval(&self.input_values(i))],
            ComponentKind::Led | ComponentKind::SevenSegment | ComponentKind::HexDisplay => {
                return;
            }
            ComponentKind::Block { block } if block.is_sequential() => {
                let ins = self.input_values(i);
                let mut state = std::mem::take(&mut self.reg[i]);
                let out = block.step(&ins, &mut state, &mut self.ff_clock[i]);
                self.reg[i] = state;
                out
            }
            ComponentKind::Block { block } => block.eval(&self.input_values(i)),
            ComponentKind::FlipFlop { ff, falling, .. } => {
                let q = self.step_flip_flop(i, ff, falling);
                vec![q, q.not()]
            }
            _ => vec![self.sources[i].unwrap_or(LogicValue::X)],
        };
        self.schedule(i, new);
    }

    /// Naplánuje změny výstupů součástky po jejím zpoždění.
    fn schedule(&mut self, i: usize, new: Vec<LogicValue>) {
        let delay = self.delay(i);
        for (k, v) in new
            .into_iter()
            .enumerate()
            .take(self.netlist.output_nets[i].len())
        {
            let g = self.out_start[i] + k;
            let expected = self.pending[g].unwrap_or(self.outputs[g]);
            if v != expected {
                self.seq += 1;
                self.queue.push(Event {
                    time: self.time + delay,
                    seq: self.seq,
                    out: g,
                    value: v,
                });
                self.pending[g] = Some(v);
            }
        }
    }

    /// Změní hodnotu zdroje (přepínač, tlačítko, konstanta, hodiny).
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
            if self.pending[e.out] == Some(e.value) {
                self.pending[e.out] = None;
            }
            if self.outputs[e.out] != e.value {
                self.outputs[e.out] = e.value;
                let (i, k) = self.out_owner[e.out];
                changed_nets.push(self.netlist.output_nets[i][k]);
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

    /// Hodnoty všech výstupů součástky.
    pub fn outputs(&self, id: CompId) -> Vec<LogicValue> {
        self.index.get(&id).map_or_else(Vec::new, |&i| {
            let start = self.out_start[i];
            self.outputs[start..start + self.netlist.output_nets[i].len()].to_vec()
        })
    }

    /// Hodnota prvního (hlavního) výstupu součástky.
    pub fn output(&self, id: CompId) -> Option<LogicValue> {
        self.outputs(id).first().copied()
    }

    /// Hodnoty na vstupních vývodech součástky.
    pub fn inputs(&self, id: CompId) -> Vec<LogicValue> {
        self.index
            .get(&id)
            .map_or_else(Vec::new, |&i| self.input_values(i))
    }

    /// Hodnoty sítí připojených k výstupům (mohou se lišit od výstupů při kolizi).
    pub fn output_net_values(&self, id: CompId) -> Vec<LogicValue> {
        self.index.get(&id).map_or_else(Vec::new, |&i| {
            self.netlist.output_nets[i]
                .iter()
                .map(|&n| self.net_values[n])
                .collect()
        })
    }

    pub fn output_net_value(&self, id: CompId) -> Option<LogicValue> {
        self.output_net_values(id).first().copied()
    }

    /// Vnitřní stav (Q) klopného obvodu.
    pub fn flip_flop_state(&self, id: CompId) -> Option<LogicValue> {
        let &i = self.index.get(&id)?;
        self.kinds[i].is_sequential().then(|| self.ff_q[i])
    }

    /// Vnitřní stav sekvenčního funkčního bloku (bity od nejnižšího).
    pub fn block_state(&self, id: CompId) -> Option<Vec<LogicValue>> {
        let &i = self.index.get(&id)?;
        match self.kinds[i] {
            ComponentKind::Block { block } if block.is_sequential() => Some(self.reg[i].clone()),
            _ => None,
        }
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
    fn counter_block_counts_switch_edges() {
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 1));
        let block = crate::BlockKind::Counter {
            bits: 4,
            down: false,
        };
        let ctr = c.add_component(ComponentKind::Block { block }, Point::new(10, 0));
        let clk = c.component(ctr).unwrap().input_pins()[2];
        assert_eq!(clk, Point::new(6, 1));
        c.add_wire(Point::new(2, 1), clk);
        let mut sim = Simulator::new(&c, None);
        sim.run(1000);
        for k in 1..=5u32 {
            sim.set_source(sw, One);
            sim.run(1000);
            sim.set_source(sw, Zero);
            sim.run(1000);
            let st = sim.block_state(ctr).unwrap();
            assert_eq!(crate::block::number(&st), Some(k));
            assert_eq!(sim.outputs(ctr)[..4], st[..]);
        }
        // Stav přežije přestavbu simulátoru.
        let again = Simulator::new(&c, Some(&sim));
        assert_eq!(
            crate::block::number(&again.block_state(ctr).unwrap()),
            Some(5)
        );
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

    /// Klopný obvod daného druhu: vstupy z přepínačů, vrací (obvod, id přepínačů, id KO).
    fn ff_circuit(ff: FlipFlopKind, falling: bool) -> (Circuit, Vec<CompId>, CompId) {
        let mut c = Circuit::default();
        let kind = ComponentKind::FlipFlop {
            ff,
            falling,
            initial: false,
        };
        let id = c.add_component(kind, Point::new(10, 0));
        let pins = c.component(id).unwrap().input_pins();
        let mut switches = Vec::new();
        for (k, p) in pins.iter().enumerate() {
            let k = k as i32;
            // Přepínače pod sebou vlevo dole, každý vodič má vlastní svislou dráhu.
            let s = c.add_component(
                ComponentKind::Switch { on: false },
                Point::new(-20, 20 + 10 * k),
            );
            let out = c.component(s).unwrap().output_pin().unwrap();
            let x = 5 - k;
            c.add_wire(out, Point::new(x, out.y));
            c.add_wire(Point::new(x, out.y), Point::new(x, p.y));
            c.add_wire(Point::new(x, p.y), *p);
            switches.push(s);
        }
        (c, switches, id)
    }

    fn q(sim: &Simulator, id: CompId) -> (LogicValue, LogicValue) {
        let o = sim.outputs(id);
        (o[0], o[1])
    }

    #[test]
    fn rs_flip_flop() {
        let (c, sw, ff) = ff_circuit(FlipFlopKind::Rs, false);
        let mut sim = Simulator::new(&c, None);
        sim.run(1000);
        assert_eq!(q(&sim, ff), (Zero, One), "počáteční stav 0");
        sim.set_source(sw[0], One);
        sim.run(1000);
        assert_eq!(q(&sim, ff), (One, Zero));
        sim.set_source(sw[0], Zero);
        sim.run(1000);
        assert_eq!(q(&sim, ff), (One, Zero), "pamatuje si 1");
        sim.set_source(sw[0], One);
        sim.set_source(sw[1], One);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, X, "zakázaný stav");
    }

    #[test]
    fn d_flip_flop_samples_on_rising_edge() {
        let (c, sw, ff) = ff_circuit(FlipFlopKind::D, false);
        let (d, clk) = (sw[0], sw[1]);
        let mut sim = Simulator::new(&c, None);
        sim.run(1000);
        sim.set_source(d, One);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, Zero, "bez hrany se nic nemění");
        sim.set_source(clk, One);
        sim.run(1000);
        assert_eq!(q(&sim, ff), (One, Zero), "náběžná hrana uloží D");
        sim.set_source(d, Zero);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, One, "při C = 1 se D neprojeví");
        sim.set_source(clk, Zero);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, One, "sestupná hrana nic nedělá");
    }

    #[test]
    fn d_latch_is_transparent() {
        let (c, sw, ff) = ff_circuit(FlipFlopKind::DLatch, false);
        let (d, en) = (sw[0], sw[1]);
        let mut sim = Simulator::new(&c, None);
        sim.set_source(en, One);
        sim.set_source(d, One);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, One);
        sim.set_source(d, Zero);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, Zero, "při C = 1 sleduje D");
        sim.set_source(en, Zero);
        sim.set_source(d, One);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, Zero, "při C = 0 drží");
    }

    fn pulse(sim: &mut Simulator, clk: CompId) {
        sim.set_source(clk, One);
        sim.run(1000);
        sim.set_source(clk, Zero);
        sim.run(1000);
    }

    #[test]
    fn jk_flip_flop_truth_table() {
        let (c, sw, ff) = ff_circuit(FlipFlopKind::Jk, false);
        let (j, clk, k) = (sw[0], sw[1], sw[2]);
        let mut sim = Simulator::new(&c, None);
        sim.run(1000);
        sim.set_source(j, One);
        pulse(&mut sim, clk);
        assert_eq!(q(&sim, ff).0, One, "J = 1 nastaví");
        sim.set_source(j, Zero);
        pulse(&mut sim, clk);
        assert_eq!(q(&sim, ff).0, One, "J = K = 0 drží");
        sim.set_source(k, One);
        pulse(&mut sim, clk);
        assert_eq!(q(&sim, ff).0, Zero, "K = 1 nuluje");
        sim.set_source(j, One);
        pulse(&mut sim, clk);
        assert_eq!(q(&sim, ff).0, One, "J = K = 1 překlápí");
        pulse(&mut sim, clk);
        assert_eq!(q(&sim, ff).0, Zero);
    }

    #[test]
    fn t_flip_flop_on_falling_edge() {
        let (c, sw, ff) = ff_circuit(FlipFlopKind::T, true);
        let (t, clk) = (sw[0], sw[1]);
        let mut sim = Simulator::new(&c, None);
        sim.set_source(t, One);
        sim.run(1000);
        sim.set_source(clk, One);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, Zero, "náběžná hrana nic");
        sim.set_source(clk, Zero);
        sim.run(1000);
        assert_eq!(q(&sim, ff).0, One, "sestupná hrana překlopí");
        pulse(&mut sim, clk);
        assert_eq!(q(&sim, ff).0, Zero);
    }

    #[test]
    fn flip_flop_state_survives_rebuild() {
        let (c, sw, ff) = ff_circuit(FlipFlopKind::T, false);
        let mut sim = Simulator::new(&c, None);
        sim.run(1000);
        sim.set_source(sw[0], One);
        pulse(&mut sim, sw[1]);
        assert_eq!(q(&sim, ff).0, One);
        let rebuilt = Simulator::new(&c, Some(&sim));
        assert_eq!(rebuilt.flip_flop_state(ff), Some(One));
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
