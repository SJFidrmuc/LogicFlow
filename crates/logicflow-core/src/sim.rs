//! Překlad obvodu na netlist a událostní simulátor.
//!
//! Každá součástka s výstupem má zpoždění (hradla a klopné obvody 1 takt,
//! zdroje 0). Změna výstupu se naplánuje do fronty událostí; po jejím provedení
//! se přepočítají sítě (propojené vodiče) a vyhodnotí součástky, které z nich
//! čtou. Díky tomu jde ukázat šíření signálu krok po kroku, hazardy i kmitání.
//!
//! [en] Translation of a circuit into a netlist and the event-driven simulator.
//!
//! Every component with an output has a delay (gates and flip-flops 1 tick,
//! sources 0). An output change is scheduled in the event queue; once it is
//! applied, the nets (connected wires) are recalculated and the components
//! reading from them are evaluated. This makes it possible to show signal
//! propagation step by step, hazards and oscillation.

use crate::LogicValue;
use crate::circuit::{
    Circuit, CompId, ComponentKind, FlipFlopKind, MAX_SUBCIRCUIT_PINS, Point, SubcircuitDef, WireId,
};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Zpoždění hradla v taktech simulace.
///
/// [en] Gate delay in simulation ticks.
pub const GATE_DELAY: u64 = 1;

/// Naplánovaná změna jednoho výstupu.
///
/// [en] Scheduled change of one output.
#[derive(Clone, Copy, Debug)]
struct Event {
    /// Čas (takt), kdy se změna provede.
    ///
    /// [en] Time (tick) at which the change takes place.
    time: u64,
    /// Pořadové číslo – události ve stejném taktu se provedou v pořadí naplánování.
    ///
    /// [en] Sequence number – events in the same tick run in the order they were scheduled.
    seq: u64,
    /// Globální index výstupu.
    ///
    /// [en] Global output index.
    out: usize,
    /// Nová hodnota výstupu.
    ///
    /// [en] New output value.
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
    ///
    /// [en] Reversed order: `BinaryHeap` then returns the earliest event.
    fn cmp(&self, other: &Self) -> Ordering {
        (other.time, other.seq).cmp(&(self.time, self.seq))
    }
}

/// Jednoduché sjednocování množin pro hledání sítí.
///
/// [en] Simple union-find (disjoint sets) for finding nets.
struct UnionFind(Vec<usize>);

impl UnionFind {
    /// Každý prvek je zatím sám sobě množinou.
    ///
    /// [en] Every element starts as its own set.
    fn new(n: usize) -> Self {
        Self((0..n).collect())
    }
    /// Kořen množiny prvku `i` (se zkracováním cesty).
    ///
    /// [en] Root of the set of element `i` (with path halving).
    fn find(&mut self, mut i: usize) -> usize {
        while self.0[i] != i {
            self.0[i] = self.0[self.0[i]];
            i = self.0[i];
        }
        i
    }
    /// Spojí množiny prvků `a` a `b`.
    ///
    /// [en] Joins the sets of elements `a` and `b`.
    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.0[ra] = rb;
        }
    }
}

/// Výsledek přeložení obvodu: které vývody a vodiče patří do které sítě.
///
/// [en] Result of translating a circuit: which pins and wires belong to which net.
#[derive(Clone, Debug, Default)]
pub struct Netlist {
    /// Počet sítí (sítě mají čísla 0..net_count).
    ///
    /// [en] Number of nets (nets are numbered 0..net_count).
    pub net_count: usize,
    /// Síť každého vodiče.
    ///
    /// [en] Net of each wire.
    pub wire_net: HashMap<WireId, usize>,
    /// Sítě vstupů každé součástky (pořadí jako `Circuit::components`).
    ///
    /// [en] Nets of each component's inputs (same order as `Circuit::components`).
    pub input_nets: Vec<Vec<usize>>,
    /// Sítě výstupů každé součástky.
    ///
    /// [en] Nets of each component's outputs.
    pub output_nets: Vec<Vec<usize>>,
}

impl Netlist {
    /// Najde sítě: propojí vodiče ve styčných bodech a přiřadí vývodům sítě.
    ///
    /// [en] Finds the nets: joins wires at their contact points and assigns nets to pins.
    pub fn build(circuit: &Circuit) -> Self {
        // Body, ve kterých se může vodič s něčím spojit.
        // [en] Points where a wire can connect to something.
        let mut points: Vec<Point> = circuit.wires.iter().flat_map(|w| [w.a, w.b]).collect();
        for c in &circuit.components {
            points.extend(c.pins());
        }
        points.sort();
        points.dedup();
        let point_index: HashMap<Point, usize> =
            points.iter().enumerate().map(|(i, p)| (*p, i)).collect();

        // Konce vodičů: v nich se vodiče propojují (i s jiným vodičem, který
        // jimi prochází – odbočka „T“). Vývod součástky se připojí jen ke konci
        // vodiče; vodič, který přes vývod jen prochází, se s ním nespojí.
        // [en] Wire ends: wires connect there (also with another wire passing through
        // them – a “T” junction). A component pin connects only to a wire end;
        // a wire that merely passes over a pin does not connect to it.
        let wire_ends: std::collections::HashSet<Point> =
            circuit.wires.iter().flat_map(|w| [w.a, w.b]).collect();
        let w = circuit.wires.len();
        let mut uf = UnionFind::new(w + points.len());
        for (wi, wire) in circuit.wires.iter().enumerate() {
            for (pi, p) in points.iter().enumerate() {
                if wire.contains(*p) && (wire.is_endpoint(*p) || wire_ends.contains(p)) {
                    uf.union(wi, w + pi);
                }
            }
        }

        // Očíslování sítí 0..n.
        // [en] Numbering of nets 0..n.
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

/// Nejvyšší hloubka vnoření podobvodů (ochrana proti zacyklení).
///
/// [en] Maximum nesting depth of subcircuits (protection against cycles).
const MAX_DEPTH: usize = 16;

/// Výsledek rozvinutí: netlist, cesty, druhy, zdroje a sítě součástek hlavního obvodu.
///
/// [en] Result of flattening: netlist, paths, kinds, sources and the nets of the main circuit's components.
type Flat = (
    Netlist,
    Vec<Vec<CompId>>,
    Vec<ComponentKind>,
    Vec<Option<LogicValue>>,
    HashMap<CompId, (Vec<usize>, Vec<usize>)>,
);

/// Rozvine obvod s podobvody do jednoho seznamu součástek a sítí.
///
/// Každý obvod (hlavní i každá instance podobvodu) dostane vlastní rozsah čísel
/// sítí. Přepínače a LED uvnitř instance (rozhraní) se nesimulují – jejich sítě
/// se spojí se sítěmi na vývodech značky.
///
/// [en] Flattens a circuit with subcircuits into one list of components and nets.
///
/// Each circuit (the main one and every subcircuit instance) gets its own range
/// of net numbers. Switches and LEDs inside an instance (the interface) are not
/// simulated – their nets are joined with the nets at the symbol's pins.
#[derive(Default)]
struct Flattener<'a> {
    /// Definice podobvodů.
    ///
    /// [en] Subcircuit definitions.
    defs: &'a [SubcircuitDef],
    /// Cesta ke každé součástce: id instancí podobvodů a nakonec id součástky.
    ///
    /// [en] Path to each component: ids of the subcircuit instances and finally the component id.
    keys: Vec<Vec<CompId>>,
    /// Druh každé součástky.
    ///
    /// [en] Kind of each component.
    kinds: Vec<ComponentKind>,
    /// Hodnota zdroje každé součástky (přepínače, konstanty, hodiny).
    ///
    /// [en] Source value of each component (switches, constants, clocks).
    sources: Vec<Option<LogicValue>>,
    /// Sítě vstupů každé součástky.
    ///
    /// [en] Nets of each component's inputs.
    input_nets: Vec<Vec<usize>>,
    /// Sítě výstupů každé součástky.
    ///
    /// [en] Nets of each component's outputs.
    output_nets: Vec<Vec<usize>>,
    /// Počet zatím přidělených sítí.
    ///
    /// [en] Number of nets allocated so far.
    net_count: usize,
    /// Dvojice sítí, které jsou ve skutečnosti jedna (rozhraní podobvodů).
    ///
    /// [en] Pairs of nets that are in fact one (subcircuit interfaces).
    links: Vec<(usize, usize)>,
}

impl Flattener<'_> {
    /// Přidá obvod; vrací jeho netlist a posun čísel jeho sítí.
    ///
    /// [en] Adds a circuit; returns its netlist and the offset of its net numbers.
    fn add(
        &mut self,
        circuit: &Circuit,
        path: &[CompId],
        interface: bool,
        depth: usize,
    ) -> (Netlist, usize) {
        let defs = self.defs;
        let nl = Netlist::build(circuit);
        let base = self.net_count;
        self.net_count += nl.net_count;
        let skip: Vec<CompId> = if interface {
            let (ins, outs) = circuit.io();
            ins.iter()
                .take(MAX_SUBCIRCUIT_PINS)
                .chain(outs.iter().take(MAX_SUBCIRCUIT_PINS))
                .map(|c| c.id)
                .collect()
        } else {
            Vec::new()
        };
        for (i, c) in circuit.components.iter().enumerate() {
            if skip.contains(&c.id) {
                continue;
            }
            let mut key = path.to_vec();
            key.push(c.id);
            if let ComponentKind::Subcircuit { id, .. } = c.kind {
                let Some(def) = defs.iter().find(|d| d.id == id) else {
                    continue;
                };
                if depth >= MAX_DEPTH {
                    continue;
                }
                let inner = &def.circuit;
                let (inl, ibase) = self.add(inner, &key, true, depth + 1);
                let pos = |id: CompId| inner.components.iter().position(|k| k.id == id);
                let (ins, outs) = inner.io();
                for (k, comp) in ins.iter().take(MAX_SUBCIRCUIT_PINS).enumerate() {
                    if let (Some(&outer), Some(j)) = (nl.input_nets[i].get(k), pos(comp.id)) {
                        self.links
                            .push((outer + base, inl.output_nets[j][0] + ibase));
                    }
                }
                for (k, comp) in outs.iter().take(MAX_SUBCIRCUIT_PINS).enumerate() {
                    if let (Some(&outer), Some(j)) = (nl.output_nets[i].get(k), pos(comp.id)) {
                        self.links
                            .push((outer + base, inl.input_nets[j][0] + ibase));
                    }
                }
                continue;
            }
            self.keys.push(key);
            self.kinds.push(c.kind);
            self.sources.push(c.source_value());
            self.input_nets
                .push(nl.input_nets[i].iter().map(|n| n + base).collect());
            self.output_nets
                .push(nl.output_nets[i].iter().map(|n| n + base).collect());
        }
        (nl, base)
    }

    /// Spojí propojené sítě a očísluje je znovu od nuly.
    ///
    /// [en] Joins linked nets and renumbers them from zero.
    fn finish(self, circuit: &Circuit, top: &Netlist) -> Flat {
        let mut uf = UnionFind::new(self.net_count);
        for &(a, b) in &self.links {
            uf.union(a, b);
        }
        let mut renumber: HashMap<usize, usize> = HashMap::new();
        let mut net = |n: usize| {
            let root = uf.find(n);
            let next = renumber.len();
            *renumber.entry(root).or_insert(next)
        };
        let map = |v: &Vec<usize>, net: &mut dyn FnMut(usize) -> usize| -> Vec<usize> {
            v.iter().map(|&n| net(n)).collect()
        };
        let input_nets: Vec<Vec<usize>> =
            self.input_nets.iter().map(|v| map(v, &mut net)).collect();
        let output_nets: Vec<Vec<usize>> =
            self.output_nets.iter().map(|v| map(v, &mut net)).collect();
        // Hlavní obvod má posun 0.
        // [en] The main circuit has offset 0.
        let wire_net = top.wire_net.iter().map(|(&w, &n)| (w, net(n))).collect();
        let top_nets = circuit
            .components
            .iter()
            .enumerate()
            .map(|(i, c)| {
                (
                    c.id,
                    (
                        map(&top.input_nets[i], &mut net),
                        map(&top.output_nets[i], &mut net),
                    ),
                )
            })
            .collect();
        let net_count = renumber.len();
        (
            Netlist {
                net_count,
                wire_net,
                input_nets,
                output_nets,
            },
            self.keys,
            self.kinds,
            self.sources,
            top_nets,
        )
    }
}

/// Událostní simulátor jednoho obvodu.
///
/// [en] Event-driven simulator of one circuit.
#[derive(Clone, Debug)]
pub struct Simulator {
    /// Netlist rozvinutého obvodu (součástky včetně těch uvnitř podobvodů).
    ///
    /// [en] Netlist of the flattened circuit (components including those inside subcircuits).
    netlist: Netlist,
    /// Id součástek hlavního obvodu.
    ///
    /// [en] Ids of the main circuit's components.
    ids: Vec<CompId>,
    /// Index součástky hlavního obvodu v rozvinutém seznamu.
    ///
    /// [en] Index of a main circuit component in the flattened list.
    index: HashMap<CompId, usize>,
    /// Index podle cesty k součástce: id instancí podobvodů a nakonec id součástky.
    ///
    /// [en] Index by the path to a component: ids of the subcircuit instances and finally the component id.
    key_index: HashMap<Vec<CompId>, usize>,
    /// Sítě vstupů a výstupů každé součástky hlavního obvodu (i instancí podobvodů).
    ///
    /// [en] Nets of the inputs and outputs of every main circuit component (subcircuit instances too).
    top_nets: HashMap<CompId, (Vec<usize>, Vec<usize>)>,
    /// Druh každé součástky rozvinutého obvodu.
    ///
    /// [en] Kind of each component of the flattened circuit.
    kinds: Vec<ComponentKind>,
    /// Hodnota zdroje každé součástky (None = není zdroj).
    ///
    /// [en] Source value of each component (None = not a source).
    sources: Vec<Option<LogicValue>>,
    /// Začátek výstupů každé součástky v `outputs`.
    ///
    /// [en] Start of each component's outputs in `outputs`.
    out_start: Vec<usize>,
    /// Vlastník každého výstupu: (součástka, pořadí výstupu).
    ///
    /// [en] Owner of each output: (component, output index).
    out_owner: Vec<(usize, usize)>,
    /// Výstupy, které budí danou síť.
    ///
    /// [en] Outputs driving the given net.
    net_drivers: Vec<Vec<usize>>,
    /// Součástky, které ze sítě čtou (vyhodnotí se při její změně).
    ///
    /// [en] Components reading from the net (evaluated when it changes).
    net_readers: Vec<Vec<usize>>,
    /// Aktuální hodnota každého výstupu.
    ///
    /// [en] Current value of each output.
    outputs: Vec<LogicValue>,
    /// Hodnota naplánovaná ve frontě pro každý výstup (None = nic nečeká).
    ///
    /// [en] Value scheduled in the queue for each output (None = nothing pending).
    pending: Vec<Option<LogicValue>>,
    /// Aktuální hodnota každé sítě (výsledek všech jejích budičů).
    ///
    /// [en] Current value of each net (the result of all its drivers).
    net_values: Vec<LogicValue>,
    /// Vnitřní stav klopných obvodů (Q) a poslední hodnota hodinového vstupu.
    ///
    /// [en] Internal state of flip-flops (Q) and the last value of the clock input.
    ff_q: Vec<LogicValue>,
    /// Poslední hodnota hodinového vstupu (pro poznání hrany).
    ///
    /// [en] Last value of the clock input (to detect an edge).
    ff_clock: Vec<LogicValue>,
    /// Vnitřní stav sekvenčních funkčních bloků (bity čítače, registru…).
    ///
    /// [en] Internal state of sequential function blocks (bits of a counter, register…).
    reg: Vec<Vec<LogicValue>>,
    /// Fronta naplánovaných událostí (nejdřívější nahoře).
    ///
    /// [en] Queue of scheduled events (earliest on top).
    queue: BinaryHeap<Event>,
    /// Počítadlo pro pořadí událostí.
    ///
    /// [en] Counter for the order of events.
    seq: u64,
    /// Aktuální čas simulace v taktech.
    ///
    /// [en] Current simulation time in ticks.
    time: u64,
    /// Nepřestal obvod měnit stav při posledním `run`?
    ///
    /// [en] Did the circuit keep changing during the last `run`?
    oscillating: bool,
    /// Sítě se zkratem napájení: budí je zároveň VCC (Konstanta 1) i GND (Konstanta 0).
    ///
    /// [en] Nets with a power short: driven by VCC (Constant 1) and GND (Constant 0) at once.
    power_short: Vec<bool>,
}

impl Simulator {
    /// Postaví simulátor pro obvod bez podobvodů. Viz [`Simulator::with_subcircuits`].
    ///
    /// [en] Builds a simulator for a circuit without subcircuits. See [`Simulator::with_subcircuits`].
    pub fn new(circuit: &Circuit, previous: Option<&Simulator>) -> Self {
        Self::with_subcircuits(circuit, &[], previous)
    }

    /// Postaví simulátor pro obvod; instance podobvodů se rozvinou podle `defs`.
    /// Pokud je zadán předchozí simulátor, převezme z něj výstupy a stav součástek
    /// se stejným id (i uvnitř podobvodů) – paměťové obvody si tak po úpravě
    /// schématu pamatují stav.
    ///
    /// [en] Builds a simulator for a circuit; subcircuit instances are flattened
    /// according to `defs`. If a previous simulator is given, outputs and states of
    /// components with the same id (also inside subcircuits) are taken over from
    /// it – memory circuits thus keep their state after the schematic is edited.
    pub fn with_subcircuits(
        circuit: &Circuit,
        defs: &[SubcircuitDef],
        previous: Option<&Simulator>,
    ) -> Self {
        let mut flat = Flattener {
            defs,
            ..Default::default()
        };
        let (top, _) = flat.add(circuit, &[], false, 0);
        let (netlist, keys, kinds, sources, top_nets) = flat.finish(circuit, &top);
        let n = keys.len();
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
        // Zkrat napájení: síť, kterou budí pevná 1 i pevná 0 (VCC/GND, konstanty).
        // Nezávisí na průběhu simulace, takže se určí hned ze zapojení.
        // [en] Power short: a net driven by a fixed 1 and a fixed 0 (VCC/GND, constants).
        // It does not depend on the course of the simulation, so it is found from the wiring.
        let power_short = net_drivers
            .iter()
            .map(|drivers| {
                let level = |want: bool| {
                    drivers.iter().any(|&g| {
                        let (i, _) = out_owner[g];
                        kinds[i].supply_level() == Some(want)
                    })
                };
                level(true) && level(false)
            })
            .collect();
        let key_index: HashMap<Vec<CompId>, usize> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| (k.clone(), i))
            .collect();
        // Index stejné součástky v předchozím simulátoru.
        // [en] Index of the same component in the previous simulator.
        let prev: Vec<Option<usize>> = keys
            .iter()
            .map(|k| previous.and_then(|p| p.key_index.get(k).copied()))
            .collect();
        let outputs = out_owner
            .iter()
            .map(|&(i, k)| {
                previous
                    .zip(prev[i])
                    .filter(|(p, j)| k < p.netlist.output_nets[*j].len())
                    .map(|(p, j)| p.outputs[p.out_start[j] + k])
                    .unwrap_or(LogicValue::X)
            })
            .collect();
        let ff_q = (0..n)
            .map(|i| match kinds[i] {
                ComponentKind::FlipFlop { initial, .. } => previous
                    .zip(prev[i])
                    .filter(|(p, j)| matches!(p.kinds[*j], ComponentKind::FlipFlop { .. }))
                    .map(|(p, j)| p.ff_q[j])
                    .unwrap_or(initial.into()),
                _ => LogicValue::X,
            })
            .collect();
        let ff_clock = (0..n)
            .map(|i| {
                previous
                    .zip(prev[i])
                    .map_or(LogicValue::X, |(p, j)| p.ff_clock[j])
            })
            .collect();
        let reg = (0..n)
            .map(|i| match kinds[i] {
                ComponentKind::Block { block } if block.is_sequential() => previous
                    .zip(prev[i])
                    .map(|(p, j)| p.reg[j].clone())
                    .filter(|s| s.len() == block.state_bits())
                    .unwrap_or_else(|| vec![LogicValue::Zero; block.state_bits()]),
                _ => Vec::new(),
            })
            .collect();
        let total_outputs = out_owner.len();
        let mut sim = Self {
            index: keys
                .iter()
                .enumerate()
                .filter(|(_, k)| k.len() == 1)
                .map(|(i, k)| (k[0], i))
                .collect(),
            ids: circuit.components.iter().map(|c| c.id).collect(),
            key_index,
            top_nets,
            kinds,
            sources,
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
            power_short,
        };
        // Hodnoty zdrojů (např. hodin) převezme z předchozího simulátoru.
        // [en] Source values (e.g. clocks) are taken over from the previous simulator.
        if let Some(p) = previous {
            for (i, &pj) in prev.iter().enumerate() {
                if matches!(sim.kinds[i], ComponentKind::Clock { .. })
                    && let Some(j) = pj
                    && let Some(&v) = p.outputs.get(p.out_start[j])
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
                // [en] Flip-flops only expose the stored state; their inputs are still settling.
                let q = sim.ff_q[i];
                sim.schedule(i, vec![q, q.not()]);
            } else {
                sim.evaluate(i);
            }
        }
        sim
    }

    /// Zpoždění součástky: hradla, klopné obvody a bloky 1 takt, zdroje 0.
    ///
    /// [en] Delay of a component: gates, flip-flops and blocks 1 tick, sources 0.
    fn delay(&self, i: usize) -> u64 {
        match self.kinds[i] {
            ComponentKind::Gate { .. }
            | ComponentKind::FlipFlop { .. }
            | ComponentKind::Block { .. } => GATE_DELAY,
            _ => 0,
        }
    }

    /// Výsledná hodnota sítě ze všech jejích budičů.
    ///
    /// [en] Resulting value of a net from all its drivers.
    fn resolve_net(&self, net: usize) -> LogicValue {
        self.net_drivers[net]
            .iter()
            .map(|&d| self.outputs[d])
            .fold(LogicValue::Z, LogicValue::resolve)
    }

    /// Hodnoty sítí na vstupech součástky.
    ///
    /// [en] Net values at the component's inputs.
    fn input_values(&self, i: usize) -> Vec<LogicValue> {
        self.netlist.input_nets[i]
            .iter()
            .map(|&n| self.net_values[n])
            .collect()
    }

    /// Nový stav klopného obvodu podle vstupů. Mění uložený stav i poslední hodnotu hodin.
    ///
    /// [en] New flip-flop state according to the inputs. Changes the stored state and the last clock value.
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
                // [en] S = R = 1 is the forbidden state.
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
    ///
    /// [en] Evaluates a component and schedules any output changes.
    fn evaluate(&mut self, i: usize) {
        let new: Vec<LogicValue> = match self.kinds[i] {
            ComponentKind::Gate { gate, .. } => vec![gate.eval(&self.input_values(i))],
            ComponentKind::Led { .. }
            | ComponentKind::LedAk { .. }
            | ComponentKind::SevenSegment { .. }
            | ComponentKind::HexDisplay { .. } => {
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
    ///
    /// [en] Schedules the component's output changes after its delay.
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
    ///
    /// [en] Changes the value of a source (switch, button, constant, clock).
    pub fn set_source(&mut self, id: CompId, value: LogicValue) {
        if let Some(&i) = self.index.get(&id) {
            self.sources[i] = Some(value);
            self.evaluate(i);
        }
    }

    /// Provede všechny události nejbližšího taktu. Vrací počet událostí (0 = klid).
    ///
    /// [en] Runs all events of the nearest tick. Returns the number of events (0 = quiet).
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
    ///
    /// [en] Simulates until the circuit settles, but at most `budget` events.
    /// Returns `true` if the circuit is quiet.
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
    ///
    /// [en] Is there still an event in the queue?
    pub fn is_stable(&self) -> bool {
        self.queue.is_empty()
    }

    /// Obvod nepřestal měnit stav ani po vyčerpání limitu událostí.
    ///
    /// [en] The circuit kept changing even after the event limit was used up.
    pub fn is_oscillating(&self) -> bool {
        self.oscillating
    }

    /// Aktuální čas simulace v taktech.
    ///
    /// [en] Current simulation time in ticks.
    pub fn time(&self) -> u64 {
        self.time
    }

    /// Je v obvodu (i uvnitř podobvodů) zkrat napájení – síť spojená s VCC i GND?
    ///
    /// [en] Is there a power short in the circuit (also inside subcircuits) – a net connected to VCC and GND?
    pub fn has_power_short(&self) -> bool {
        self.power_short.iter().any(|&s| s)
    }

    /// Patří vodič do sítě se zkratem napájení?
    ///
    /// [en] Does the wire belong to a net with a power short?
    pub fn wire_in_power_short(&self, id: WireId) -> bool {
        self.netlist
            .wire_net
            .get(&id)
            .is_some_and(|&n| self.power_short[n])
    }

    /// Netlist rozvinutého obvodu.
    ///
    /// [en] Netlist of the flattened circuit.
    pub fn netlist(&self) -> &Netlist {
        &self.netlist
    }

    /// Hodnota sítě, do které patří vodič (Z, pokud vodič nezná).
    ///
    /// [en] Value of the net the wire belongs to (Z if the wire is unknown).
    pub fn wire_value(&self, id: WireId) -> LogicValue {
        self.netlist
            .wire_net
            .get(&id)
            .map_or(LogicValue::Z, |&n| self.net_values[n])
    }

    /// Hodnoty všech výstupů součástky (u podobvodu hodnoty jeho výstupních sítí).
    ///
    /// [en] Values of all outputs of a component (for a subcircuit the values of its output nets).
    pub fn outputs(&self, id: CompId) -> Vec<LogicValue> {
        match self.index.get(&id) {
            Some(&i) => {
                let start = self.out_start[i];
                self.outputs[start..start + self.netlist.output_nets[i].len()].to_vec()
            }
            None => self.output_net_values(id),
        }
    }

    /// Hodnota prvního (hlavního) výstupu součástky.
    ///
    /// [en] Value of the component's first (main) output.
    pub fn output(&self, id: CompId) -> Option<LogicValue> {
        self.outputs(id).first().copied()
    }

    /// Hodnoty na vstupních vývodech součástky.
    ///
    /// [en] Values at the component's input pins.
    pub fn inputs(&self, id: CompId) -> Vec<LogicValue> {
        self.top_nets.get(&id).map_or_else(Vec::new, |(ins, _)| {
            ins.iter().map(|&n| self.net_values[n]).collect()
        })
    }

    /// Hodnoty sítí připojených k výstupům (mohou se lišit od výstupů při kolizi).
    ///
    /// [en] Values of the nets connected to the outputs (may differ from the outputs in a conflict).
    pub fn output_net_values(&self, id: CompId) -> Vec<LogicValue> {
        self.top_nets.get(&id).map_or_else(Vec::new, |(_, outs)| {
            outs.iter().map(|&n| self.net_values[n]).collect()
        })
    }

    /// Hodnota sítě na prvním výstupu.
    ///
    /// [en] Net value at the first output.
    pub fn output_net_value(&self, id: CompId) -> Option<LogicValue> {
        self.output_net_values(id).first().copied()
    }

    /// Vnitřní stav (Q) klopného obvodu.
    ///
    /// [en] Internal state (Q) of a flip-flop.
    pub fn flip_flop_state(&self, id: CompId) -> Option<LogicValue> {
        let &i = self.index.get(&id)?;
        self.kinds[i].is_sequential().then(|| self.ff_q[i])
    }

    /// Vnitřní stav sekvenčního funkčního bloku (bity od nejnižšího).
    ///
    /// [en] Internal state of a sequential function block (bits from the lowest).
    pub fn block_state(&self, id: CompId) -> Option<Vec<LogicValue>> {
        let &i = self.index.get(&id)?;
        match self.kinds[i] {
            ComponentKind::Block { block } if block.is_sequential() => Some(self.reg[i].clone()),
            _ => None,
        }
    }

    /// Id součástek hlavního obvodu.
    ///
    /// [en] Ids of the main circuit's components.
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
    ///
    /// [en] Switch at (0,0) → gates in a row → LED. Returns (circuit, switch id, LED id).
    fn chain(gates: &[GateKind]) -> (Circuit, CompId, CompId) {
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        // výstup přepínače
        // [en] switch output
        let mut x = 2;
        for &g in gates {
            let id = c.add_component(ComponentKind::gate(g), Point::new(x + 4, 0));
            c.add_wire(Point::new(x, 0), Point::new(x + 1, 0));
            x = c.component(id).unwrap().output_pin().unwrap().x;
        }
        let led = c.add_component(ComponentKind::LED, Point::new(x + 3, 0));
        c.add_wire(Point::new(x, 0), Point::new(x + 1, 0));
        (c, sw, led)
    }

    fn led(sim: &Simulator, id: CompId) -> LogicValue {
        sim.inputs(id)[0]
    }

    /// Podobvod „poloviční sčítačka“: vstupy A, B, výstupy C (přenos), S (součet).
    ///
    /// [en] Subcircuit “half adder”: inputs A, B, outputs C (carry), S (sum).
    fn half_adder_def(id: u32) -> SubcircuitDef {
        let mut c = Circuit::default();
        let a = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let b = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 10));
        assert_eq!(c.component(a).unwrap().label, "A");
        assert_eq!(c.component(b).unwrap().label, "B");
        // XOR na (10,1): vstupy (7,0),(7,2), výstup (13,1); AND na (10,9): (7,8),(7,10) → (13,9)
        // [en] XOR at (10,1): inputs (7,0),(7,2), output (13,1); AND at (10,9): (7,8),(7,10) → (13,9)
        c.add_component(ComponentKind::gate(GateKind::Xor), Point::new(10, 1));
        c.add_component(ComponentKind::gate(GateKind::And), Point::new(10, 9));
        c.add_wire(Point::new(2, 0), Point::new(7, 0));
        c.add_wire(Point::new(4, 0), Point::new(4, 8));
        c.add_wire(Point::new(4, 8), Point::new(7, 8));
        c.add_wire(Point::new(2, 10), Point::new(7, 10));
        c.add_wire(Point::new(5, 10), Point::new(5, 2));
        c.add_wire(Point::new(5, 2), Point::new(7, 2));
        let s = c.add_component(ComponentKind::LED, Point::new(17, 1));
        let k = c.add_component(ComponentKind::LED, Point::new(17, 9));
        c.add_wire(Point::new(13, 1), Point::new(15, 1));
        c.add_wire(Point::new(13, 9), Point::new(15, 9));
        c.component_mut(s).unwrap().label = "S".into();
        c.component_mut(k).unwrap().label = "C".into();
        SubcircuitDef {
            id,
            name: "Poloviční sčítačka".into(),
            circuit: c,
        }
    }

    /// Hlavní obvod: přepínače X, Y → instance podobvodu → LED na výstupech.
    ///
    /// [en] Main circuit: switches X, Y → subcircuit instance → LEDs at the outputs.
    fn with_instance(def: &SubcircuitDef) -> (Circuit, CompId, CompId, CompId, Vec<CompId>) {
        let mut c = Circuit::default();
        let kind = def.kind();
        assert_eq!(
            kind,
            ComponentKind::Subcircuit {
                id: def.id,
                inputs: 2,
                outputs: 2
            }
        );
        let inst = c.add_component(kind, Point::new(20, 0));
        let pins = c.component(inst).unwrap().clone();
        let (ins, outs) = (pins.input_pins(), pins.output_pins());
        let x = c.add_component(ComponentKind::Switch { on: false }, ins[0].offset(-2, 0));
        let y = c.add_component(ComponentKind::Switch { on: false }, ins[1].offset(-2, 0));
        let leds = outs
            .iter()
            .map(|p| c.add_component(ComponentKind::LED, p.offset(2, 0)))
            .collect();
        (c, inst, x, y, leds)
    }

    #[test]
    fn subcircuit_instance_computes() {
        let def = half_adder_def(1);
        let (c, inst, x, y, leds) = with_instance(&def);
        let mut sim = Simulator::with_subcircuits(&c, std::slice::from_ref(&def), None);
        sim.run(1000);
        // Výstupy podle popisku: C, S.
        // [en] Outputs by label: C, S.
        let out = |sim: &Simulator| leds.iter().map(|&l| led(sim, l)).collect::<Vec<_>>();
        assert_eq!(out(&sim), [Zero, Zero]);
        sim.set_source(x, One);
        sim.run(1000);
        assert_eq!(out(&sim), [Zero, One]);
        sim.set_source(y, One);
        sim.run(1000);
        assert_eq!(out(&sim), [One, Zero]);
        // Hodnoty na vývodech značky.
        // [en] Values at the symbol's pins.
        assert_eq!(sim.inputs(inst), [One, One]);
        assert_eq!(sim.output_net_values(inst), [One, Zero]);
        // Bez definice zůstanou výstupy nepřipojené.
        // [en] Without a definition the outputs stay unconnected.
        let mut bare = Simulator::new(&c, None);
        bare.run(1000);
        assert_eq!(led(&bare, leds[0]), Z);
    }

    #[test]
    fn subcircuit_keeps_gate_delays() {
        let def = half_adder_def(1);
        let (c, _, x, _, leds) = with_instance(&def);
        let mut sim = Simulator::with_subcircuits(&c, std::slice::from_ref(&def), None);
        sim.run(1000);
        let t0 = sim.time();
        sim.set_source(x, One);
        while led(&sim, leds[1]) != One {
            assert!(sim.step() > 0);
        }
        assert_eq!(sim.time() - t0, 1, "XOR uvnitř = jeden takt");
    }

    #[test]
    fn nested_subcircuits_and_cycles() {
        let inner = half_adder_def(1);
        // Vnější podobvod obsahuje instanci vnitřního a vlastní vstupy/výstupy.
        // [en] The outer subcircuit contains an instance of the inner one and its own inputs/outputs.
        let (mut outer_c, _, _, _, _) = with_instance(&inner);
        let names = ["A", "B"];
        let mut k = 0;
        for comp in &mut outer_c.components {
            if comp.kind.is_input() {
                comp.label = names[k].into();
                k += 1;
            }
        }
        let outer = SubcircuitDef {
            id: 2,
            name: "Obal".into(),
            circuit: outer_c,
        };
        let defs = vec![inner.clone(), outer.clone()];
        assert!(outer.uses(1, &defs));
        assert!(!inner.uses(2, &defs));
        let (c, _, x, y, leds) = with_instance(&outer);
        let mut sim = Simulator::with_subcircuits(&c, &defs, None);
        sim.set_source(x, One);
        sim.set_source(y, One);
        sim.run(1000);
        assert_eq!(led(&sim, leds[0]), One);
        assert_eq!(led(&sim, leds[1]), Zero);
        // Podobvod, který obsahuje sám sebe, nesmí simulátor zacyklit.
        // [en] A subcircuit that contains itself must not make the simulator loop.
        let mut selfref = inner.clone();
        selfref
            .circuit
            .add_component(selfref.kind(), Point::new(40, 0));
        let mut sim = Simulator::with_subcircuits(&c, &[selfref, outer], None);
        sim.run(1000);
    }

    #[test]
    fn state_inside_subcircuit_survives_rebuild() {
        // Podobvod: přepínač C → hodiny čítače, LED na Q0.
        // [en] Subcircuit: switch C → counter clock, LED on Q0.
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 1));
        let block = crate::BlockKind::Counter {
            bits: 4,
            down: false,
        };
        c.add_component(ComponentKind::Block { block }, Point::new(10, 0));
        c.add_wire(Point::new(2, 1), Point::new(6, 1));
        c.add_component(ComponentKind::LED, Point::new(16, -3));
        let def = SubcircuitDef {
            id: 7,
            name: "Čítač".into(),
            circuit: c,
        };
        let mut top = Circuit::default();
        let inst = top.add_component(def.kind(), Point::new(10, 0));
        let pin = top.component(inst).unwrap().input_pins()[0];
        let sw = top.add_component(ComponentKind::Switch { on: false }, pin.offset(-2, 0));
        let defs = [def];
        let mut sim = Simulator::with_subcircuits(&top, &defs, None);
        sim.run(1000);
        for v in [One, Zero] {
            sim.set_source(sw, v);
            sim.run(1000);
        }
        assert_eq!(sim.output_net_values(inst), [One]);
        let again = Simulator::with_subcircuits(&top, &defs, Some(&sim));
        assert_eq!(again.output_net_values(inst), [One]);
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
        // [en] The state survives a rebuild of the simulator.
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
    ///
    /// [en] RS flip-flop made of two NOR gates.
    fn sr_latch() -> (Circuit, CompId, CompId, CompId) {
        let mut c = Circuit::default();
        let s = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let r = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 10));
        // NOR1 (výstup Qn) vstupy (7,-1) a (7,1), výstup (13,0)
        // NOR2 (výstup Q)  vstupy (7,9) a (7,11), výstup (13,10)
        // [en] NOR1 (output Qn) inputs (7,-1) and (7,1), output (13,0)
        // NOR2 (output Q)  inputs (7,9) and (7,11), output (13,10)
        c.add_component(ComponentKind::gate(GateKind::Nor), Point::new(10, 0));
        let q = c.add_component(ComponentKind::gate(GateKind::Nor), Point::new(10, 10));
        // S → NOR1 horní vstup
        // [en] S → NOR1 upper input
        c.add_wire(Point::new(2, 0), Point::new(4, 0));
        c.add_wire(Point::new(4, 0), Point::new(4, -1));
        c.add_wire(Point::new(4, -1), Point::new(7, -1));
        // R → NOR2 dolní vstup
        // [en] R → NOR2 lower input
        c.add_wire(Point::new(2, 10), Point::new(4, 10));
        c.add_wire(Point::new(4, 10), Point::new(4, 11));
        c.add_wire(Point::new(4, 11), Point::new(7, 11));
        // Qn (13,0) → NOR2 horní vstup (7,9)
        // [en] Qn (13,0) → NOR2 upper input (7,9)
        c.add_wire(Point::new(13, 0), Point::new(15, 0));
        c.add_wire(Point::new(15, 0), Point::new(15, 5));
        c.add_wire(Point::new(15, 5), Point::new(6, 5));
        c.add_wire(Point::new(6, 5), Point::new(6, 9));
        c.add_wire(Point::new(6, 9), Point::new(7, 9));
        // Q (13,10) → NOR1 dolní vstup (7,1)
        // [en] Q (13,10) → NOR1 lower input (7,1)
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
        // [en] Rebuilding the simulator (editing the schematic) keeps the state.
        let mut rebuilt = Simulator::new(&c, Some(&sim));
        assert!(rebuilt.run(1000));
        assert_eq!(rebuilt.output(q), Some(Zero));
    }

    #[test]
    fn ring_oscillator_is_detected() {
        // NAND se zpětnou vazbou: při EN = 1 kmitá.
        // [en] NAND with feedback: oscillates when EN = 1.
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
        // [en] Switching it off calms the circuit down.
        sim.set_source(en, Zero);
        assert!(sim.run(1000));
        assert_eq!(sim.output(g), Some(One));
    }

    /// Klopný obvod daného druhu: vstupy z přepínačů, vrací (obvod, id přepínačů, id KO).
    ///
    /// [en] Flip-flop of the given kind: inputs from switches, returns (circuit, switch ids, flip-flop id).
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
            // [en] Switches below each other at the bottom left, each wire has its own vertical track.
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
        let l = c.add_component(ComponentKind::LED, Point::new(8, 0));
        c.add_wire(Point::new(2, 0), Point::new(6, 0));
        c.add_wire(Point::new(2, 4), Point::new(4, 4));
        // odbočka „T“ do prvního vodiče
        // [en] “T” branch into the first wire
        c.add_wire(Point::new(4, 4), Point::new(4, 0));
        let mut sim = Simulator::new(&c, None);
        sim.run(100);
        assert_eq!(led(&sim, l), X);
    }

    #[test]
    fn crossing_wires_do_not_connect() {
        let mut c = Circuit::default();
        let a = c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        let l = c.add_component(ComponentKind::LED, Point::new(12, 0));
        c.add_wire(Point::new(2, 0), Point::new(10, 0));
        // Svislý vodič přes střed vodorovného – jen křížení.
        // [en] A vertical wire across the middle of a horizontal one – just a crossing.
        let w = c.add_wire(Point::new(5, -3), Point::new(5, 3)).unwrap();
        let mut sim = Simulator::new(&c, None);
        sim.run(100);
        assert_eq!(led(&sim, l), One);
        assert_eq!(sim.wire_value(w), Z, "nepřipojený vodič");
        let _ = a;
    }

    /// VCC dává 1, GND 0; LED připojená k VCC svítí a bez spojení VCC s GND není zkrat.
    ///
    /// [en] VCC gives 1, GND 0; an LED connected to VCC lights up and there is no short
    /// without VCC joined to GND.
    #[test]
    fn power_supply_values() {
        let mut c = Circuit::default();
        // VCC na (0,0) má vývod (0,1); LED na (3,1) má vstup (1,1).
        // [en] VCC at (0,0) has its pin at (0,1); LED at (3,1) has its input at (1,1).
        let vcc = c.add_component(ComponentKind::Vcc, Point::new(0, 0));
        let led = c.add_component(ComponentKind::LED, Point::new(3, 1));
        let gnd = c.add_component(ComponentKind::Gnd, Point::new(10, 0));
        c.add_wire(Point::new(0, 1), Point::new(1, 1)).unwrap();
        assert_eq!(c.component(vcc).unwrap().label, "", "napájení bez popisku");
        assert_eq!(c.component(gnd).unwrap().pins(), vec![Point::new(10, -1)]);
        let mut sim = Simulator::new(&c, None);
        sim.run(100);
        assert_eq!(sim.inputs(led), vec![One]);
        assert_eq!(sim.output(gnd), Some(Zero));
        assert!(!sim.has_power_short());
    }

    /// Vodič mezi VCC a GND (nebo Konstantou 0) je zkrat napájení: síť je X a označená.
    ///
    /// [en] A wire between VCC and GND (or Constant 0) is a power short: the net is X and marked.
    #[test]
    fn power_short_detected() {
        for low in [ComponentKind::Gnd, ComponentKind::Constant { value: false }] {
            let mut c = Circuit::default();
            c.add_component(ComponentKind::Vcc, Point::new(0, 0));
            // GND na (4,2) má vývod (4,1); Konstanta 0 na (2,1) má výstup (4,1).
            // [en] GND at (4,2) has its pin at (4,1); Constant 0 at (2,1) has its output at (4,1).
            let pos = if low == ComponentKind::Gnd {
                Point::new(4, 2)
            } else {
                Point::new(2, 1)
            };
            c.add_component(low, pos);
            let w = c.add_wire(Point::new(0, 1), Point::new(4, 1)).unwrap();
            let other = c.add_wire(Point::new(0, 5), Point::new(4, 5)).unwrap();
            let mut sim = Simulator::new(&c, None);
            sim.run(100);
            assert!(sim.has_power_short(), "{low:?}");
            assert!(sim.wire_in_power_short(w));
            assert!(!sim.wire_in_power_short(other));
            assert_eq!(sim.wire_value(w), X, "kolize budičů");
        }
    }

    /// Zkrat napájení uvnitř podobvodu se ohlásí i v nadřazeném obvodu.
    ///
    /// [en] A power short inside a subcircuit is reported in the parent circuit too.
    #[test]
    fn power_short_inside_subcircuit() {
        let mut inner = Circuit::default();
        inner.add_component(ComponentKind::Vcc, Point::new(0, 0));
        inner.add_component(ComponentKind::Gnd, Point::new(4, 2));
        inner.add_wire(Point::new(0, 1), Point::new(4, 1)).unwrap();
        let led = inner.add_component(ComponentKind::LED, Point::new(10, 0));
        inner.component_mut(led).unwrap().label = "Y".into();
        let def = SubcircuitDef {
            id: 1,
            name: "Zkrat".into(),
            circuit: inner,
        };
        let mut c = Circuit::default();
        c.add_component(
            ComponentKind::Subcircuit {
                id: 1,
                inputs: 0,
                outputs: 1,
            },
            Point::new(0, 0),
        );
        let sim = Simulator::with_subcircuits(&c, &[def], None);
        assert!(sim.has_power_short());
    }
}
