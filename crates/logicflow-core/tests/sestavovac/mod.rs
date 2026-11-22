//! Sestavovač schémat pro generované knihovny.
//!
//! Zapojení se popíše jako netlist (vstupy, hradla, klopné obvody, podobvody,
//! výstupy) a sestavovač ho rozmístí do sloupců a propojí vodiči: každá síť má
//! v kanálu mezi sloupci vlastní svislou stopu, ze které vedou vodorovné
//! odbočky k vývodům. Spoje do vzdálenějšího sloupce nebo zpětné vazby vedou
//! vlastní řadou pod schématem. Nakonec se z hotového schématu postaví netlist
//! a porovná se zamýšleným zapojením – omylem spojené nebo rozpojené sítě
//! sestavovač odhalí sám.
//!
//! [en]
//!
//! Schematic builder for generated libraries.
//!
//! A circuit is described as a netlist (inputs, gates, flip-flops, subcircuits,
//! outputs) and the builder places it in columns and connects it with wires:
//! every net has its own vertical track in the channel between columns, with
//! horizontal branches to the pins. Connections to a more distant column and
//! feedback loops run along their own row below the schematic. Finally a netlist
//! is built from the finished schematic and compared with the intended circuit –
//! the builder itself detects nets connected or disconnected by mistake.

use std::collections::{BTreeMap, BTreeSet};

use logicflow_core::sim::Netlist;
use logicflow_core::{Circuit, ComponentKind, FlipFlopKind, GateKind, Point, SubcircuitDef};

/// Signál (síť) ve sestavovaném obvodu.
///
/// [en] A signal (net) in the circuit being built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sig(usize);

/// Součástka ve sestavovaném obvodu.
///
/// [en] A component in the circuit being built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part(usize);

/// Rozpracovaná součástka: druh, sloupec, popisek a připojené signály.
///
/// [en] Component being built: kind, column, label and connected signals.
struct PartInfo {
    /// Druh součástky.
    ///
    /// [en] Kind of component.
    kind: ComponentKind,
    /// Sloupec (0 = vstupy; `None` = výstupy v posledním sloupci).
    ///
    /// [en] Column (0 = inputs; `None` = outputs in the last column).
    col: Option<usize>,
    /// Popisek.
    ///
    /// [en] Label.
    label: String,
    /// Signály na vstupech (nepřipojený = `None`).
    ///
    /// [en] Signals at the inputs (unconnected = `None`).
    ins: Vec<Option<Sig>>,
    /// Signály na výstupech.
    ///
    /// [en] Signals at the outputs.
    outs: Vec<Sig>,
}

/// Konec vodiče na vývodu: součástka, číslo vývodu, výstup?
///
/// [en] Wire end at a pin: component, pin number, output?
type PinRef = (usize, usize, bool);

/// Vzdálenost stop v kanálu (body mřížky).
///
/// [en] Distance between tracks in a channel (grid points).
const TRACK: i32 = 2;

/// Sestavovač schématu.
///
/// [en] Schematic builder.
#[derive(Default)]
pub struct Sestavovac {
    /// Součástky v pořadí vložení (stejné pořadí bude v obvodu).
    ///
    /// [en] Components in insertion order (the circuit keeps the same order).
    parts: Vec<PartInfo>,
    /// Zdroj každého signálu (součástka, výstup).
    ///
    /// [en] Source of each signal (component, output).
    sources: Vec<(usize, usize)>,
}

impl Sestavovac {
    /// Prázdný sestavovač.
    ///
    /// [en] Empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Přidá součástku a vytvoří signály pro všechny její výstupy.
    ///
    /// [en] Adds a component and creates signals for all its outputs.
    fn add(&mut self, kind: ComponentKind, col: Option<usize>, label: &str) -> Part {
        let idx = self.parts.len();
        let n_out = kind.output_count();
        let outs = (0..n_out)
            .map(|o| {
                self.sources.push((idx, o));
                Sig(self.sources.len() - 1)
            })
            .collect();
        self.parts.push(PartInfo {
            kind,
            col,
            label: label.to_owned(),
            ins: vec![None; kind.input_count()],
            outs,
        });
        Part(idx)
    }

    /// Sloupec pro součástku se vstupy `ins`: o jeden za nejvzdálenějším zdrojem.
    ///
    /// [en] Column for a component with inputs `ins`: one after the farthest source.
    fn auto_col(&self, ins: &[Sig]) -> usize {
        ins.iter()
            .map(|s| {
                self.parts[self.sources[s.0].0]
                    .col
                    .expect("zdroj není výstup / source is not an output")
                    + 1
            })
            .max()
            .unwrap_or(1)
    }

    /// Vstup obvodu – přepínač s popiskem (tvoří rozhraní prvku).
    ///
    /// [en] Circuit input – a switch with a label (forms the item's interface).
    pub fn vstup(&mut self, label: &str) -> Sig {
        let p = self.add(ComponentKind::Switch { on: false }, Some(0), label);
        self.parts[p.0].outs[0]
    }

    /// Konstanta (logická 0 nebo 1) ve sloupci vstupů.
    ///
    /// [en] Constant (logic 0 or 1) in the inputs column.
    pub fn konstanta(&mut self, value: bool) -> Sig {
        let p = self.add(ComponentKind::Constant { value }, Some(0), "");
        self.parts[p.0].outs[0]
    }

    /// Hradlo se vstupy `ins` ve sloupci za jejich zdroji.
    ///
    /// [en] Gate with inputs `ins` in the column after their sources.
    pub fn hradlo(&mut self, gate: GateKind, ins: &[Sig]) -> Sig {
        let col = self.auto_col(ins);
        self.hradlo_ve(col, gate, ins)
    }

    /// Hradlo ve zvoleném sloupci (pro zpětné vazby).
    ///
    /// [en] Gate in a chosen column (for feedback loops).
    pub fn hradlo_ve(&mut self, col: usize, gate: GateKind, ins: &[Sig]) -> Sig {
        let p = self.hradlo_bez(col, gate, ins.len());
        for (i, &s) in ins.iter().enumerate() {
            self.spoj(p, i, s);
        }
        self.vystupy(p)[0]
    }

    /// Hradlo ve zvoleném sloupci s nepřipojenými vstupy (připojí se později přes [`Sestavovac::spoj`]).
    ///
    /// [en] Gate in a chosen column with unconnected inputs (connected later via [`Sestavovac::spoj`]).
    pub fn hradlo_bez(&mut self, col: usize, gate: GateKind, inputs: usize) -> Part {
        let inputs = if gate.is_unary() { 1 } else { inputs as u8 };
        self.add(ComponentKind::Gate { gate, inputs }, Some(col), "")
    }

    /// Klopný obvod ve zvoleném sloupci; vstupy se připojí přes [`Sestavovac::spoj`].
    /// Výstupy: Q, Q̄.
    ///
    /// [en] Flip-flop in a chosen column; inputs are connected via [`Sestavovac::spoj`].
    /// Outputs: Q, Q̄.
    pub fn klopny(&mut self, col: usize, ff: FlipFlopKind, initial: bool) -> Part {
        let kind = ComponentKind::FlipFlop {
            ff,
            falling: false,
            initial,
        };
        self.add(kind, Some(col), "")
    }

    /// Instance podobvodu se vstupy `ins` (v pořadí rozhraní); vrací výstupy.
    ///
    /// [en] Subcircuit instance with inputs `ins` (in interface order); returns the outputs.
    pub fn podobvod(&mut self, def: &SubcircuitDef, ins: &[Sig]) -> Vec<Sig> {
        let col = self.auto_col(ins);
        let p = self.add(def.kind(), Some(col), "");
        assert_eq!(
            ins.len(),
            self.parts[p.0].ins.len(),
            "počet vstupů / input count"
        );
        for (i, &s) in ins.iter().enumerate() {
            self.spoj(p, i, s);
        }
        self.vystupy(p)
    }

    /// Připojí signál na vstup `pin` součástky.
    ///
    /// [en] Connects a signal to input `pin` of a component.
    pub fn spoj(&mut self, part: Part, pin: usize, sig: Sig) {
        self.parts[part.0].ins[pin] = Some(sig);
    }

    /// Výstupní signály součástky.
    ///
    /// [en] Output signals of a component.
    pub fn vystupy(&self, part: Part) -> Vec<Sig> {
        self.parts[part.0].outs.clone()
    }

    /// Výstup obvodu – LED s popiskem v posledním sloupci (tvoří rozhraní prvku).
    ///
    /// [en] Circuit output – an LED with a label in the last column (forms the item's interface).
    pub fn vystup(&mut self, label: &str, sig: Sig) {
        let p = self.add(ComponentKind::Led, None, label);
        self.spoj(p, 0, sig);
    }

    /// Rozmístí součástky, propojí je vodiči a ověří zapojení podle netlistu.
    ///
    /// [en] Places the components, connects them with wires and verifies the circuit against the netlist.
    pub fn sestav(self) -> Circuit {
        let last = self.parts.iter().filter_map(|p| p.col).max().unwrap_or(0) + 1;
        let col_of = |p: &PartInfo| p.col.unwrap_or(last);
        let n_cols = last + 1;

        // Odběratelé každé sítě.
        // [en] Sinks of every net.
        let mut sinks: Vec<Vec<PinRef>> = vec![Vec::new(); self.sources.len()];
        for (pi, p) in self.parts.iter().enumerate() {
            for (k, s) in p.ins.iter().enumerate() {
                if let Some(s) = s {
                    sinks[s.0].push((pi, k, false));
                }
            }
        }
        let net_of_input = |pi: usize, k: usize| self.parts[pi].ins[k];

        // 1. Svislé rozmístění po sloupcích: součástka se snaží být na úrovni
        //    svých zdrojů, ale nesmí mít vstup na stejné výšce jako výstup jiné
        //    sítě v předchozím sloupci (odbočka by se dotkla cizího vodiče).
        // [en] 1. Vertical placement per column: a component tries to be level with
        //    its sources, but must not have an input at the same height as an output
        //    of another net in the previous column (the branch would touch a foreign wire).
        let mut ys: Vec<i32> = vec![0; self.parts.len()];
        let mut placed: Vec<bool> = vec![false; self.parts.len()];
        // Výška výstupů každé sítě (zdroj) po rozmístění.
        // [en] Height of each net's outputs (source) after placement.
        let src_y = |ys: &[i32], placed: &[bool], s: Sig| -> Option<i32> {
            let (pi, o) = self.sources[s.0];
            placed[pi].then(|| ys[pi] + self.parts[pi].kind.pins_local().1[o].1)
        };
        let mut bottom = 0;
        for col in 0..n_cols {
            let mut cursor = i32::MIN;
            // Výstupy předchozího sloupce: výška → síť.
            // [en] Outputs of the previous column: height → net.
            let mut prev_out: BTreeMap<i32, Sig> = BTreeMap::new();
            if col > 0 {
                for (pi, p) in self.parts.iter().enumerate() {
                    if col_of(p) == col - 1 {
                        for (o, &s) in p.outs.iter().enumerate() {
                            prev_out.insert(ys[pi] + p.kind.pins_local().1[o].1, s);
                        }
                    }
                }
            }
            for pi in 0..self.parts.len() {
                let p = &self.parts[pi];
                if col_of(p) != col {
                    continue;
                }
                let (_, y1, _, y2) = p.kind.body_local();
                let (pins_in, _) = p.kind.pins_local();
                // Přednostní výška: průměr výšek zdrojů (vstupy proti nim).
                // [en] Preferred height: average height of the sources (inputs facing them).
                let offs: Vec<i32> = p
                    .ins
                    .iter()
                    .enumerate()
                    .filter_map(|(k, s)| {
                        s.and_then(|s| src_y(&ys, &placed, s))
                            .map(|y| y - pins_in[k].1)
                    })
                    .collect();
                let preferred = if offs.is_empty() {
                    cursor.max(0)
                } else {
                    let sum: i32 = offs.iter().sum();
                    (sum as f32 / offs.len() as f32).round() as i32
                };
                let top_ok = |cy: i32| cursor == i32::MIN || cy as f32 + y1 >= cursor as f32;
                let mut cy = preferred;
                while !top_ok(cy) {
                    cy += 1;
                }
                let clash = |cy: i32| {
                    pins_in.iter().enumerate().any(|(k, pin)| {
                        prev_out
                            .get(&(cy + pin.1))
                            .is_some_and(|&s| Some(s) != net_of_input(pi, k))
                    })
                };
                while clash(cy) {
                    cy += 1;
                }
                ys[pi] = cy;
                placed[pi] = true;
                let gap = if p.kind.is_input() || matches!(p.kind, ComponentKind::Led) {
                    1.0
                } else {
                    2.0
                };
                cursor = (cy as f32 + y2 + gap).ceil() as i32;
                bottom = bottom.max((cy as f32 + y2).ceil() as i32);
            }
        }

        // 2. Které sítě potřebují stopu v kterém kanálu a které řadu.
        //    Kanál k leží vlevo od sloupce k.
        // [en] 2. Which nets need a track in which channel and which need a row.
        //    Channel k lies to the left of column k.
        let mut channel_nets: Vec<BTreeSet<Sig>> = vec![BTreeSet::new(); n_cols + 1];
        let mut row_of: BTreeMap<Sig, i32> = BTreeMap::new();
        let mut next_row = bottom + 3;
        for (si, list) in sinks.iter().enumerate() {
            if list.is_empty() {
                continue;
            }
            let s = Sig(si);
            let src_col = col_of(&self.parts[self.sources[si].0]);
            channel_nets[src_col + 1].insert(s);
            for &(pi, _, _) in list {
                let c = col_of(&self.parts[pi]);
                if c != src_col + 1 {
                    channel_nets[c].insert(s);
                    if let std::collections::btree_map::Entry::Vacant(e) = row_of.entry(s) {
                        e.insert(next_row);
                        next_row += TRACK;
                    }
                }
            }
        }

        // 3. Vodorovné rozmístění: šířka sloupce podle součástek, šířka kanálu podle počtu stop.
        //    Stopy v kanálu jsou seřazené podle výšky zdroje.
        // [en] 3. Horizontal placement: column width by its components, channel width by
        //    the number of tracks. Tracks in a channel are sorted by source height.
        let extent = |p: &PartInfo| {
            let (x1, _, x2, _) = p.kind.body_local();
            let (pi, po) = p.kind.pins_local();
            let pins = pi.iter().chain(&po).map(|q| q.0);
            let lo = pins.clone().min().unwrap_or(0).min(x1.floor() as i32);
            let hi = pins.max().unwrap_or(0).max(x2.ceil() as i32);
            (lo, hi)
        };
        let mut col_x = vec![0; n_cols];
        let mut track_x: BTreeMap<(usize, Sig), i32> = BTreeMap::new();
        let mut right = 0;
        for col in 0..n_cols {
            let members: Vec<&PartInfo> = self.parts.iter().filter(|p| col_of(p) == col).collect();
            let lo = members.iter().map(|p| extent(p).0).min().unwrap_or(0);
            let hi = members.iter().map(|p| extent(p).1).max().unwrap_or(0);
            if col > 0 {
                let mut nets: Vec<Sig> = channel_nets[col].iter().copied().collect();
                nets.sort_by_key(|&s| {
                    let (pi, _) = self.sources[s.0];
                    let local = col_of(&self.parts[pi]) + 1 == col;
                    (!local, src_y(&ys, &placed, s).unwrap_or(0), s)
                });
                for (i, s) in nets.iter().enumerate() {
                    track_x.insert((col, *s), right + TRACK * (i as i32 + 1));
                }
                right += TRACK * (nets.len() as i32 + 1);
            }
            col_x[col] = right - lo;
            right = col_x[col] + hi;
        }

        // 4. Součástky do obvodu.
        // [en] 4. Components into the circuit.
        let mut c = Circuit::default();
        for (pi, p) in self.parts.iter().enumerate() {
            let id = c.add_component(p.kind, Point::new(col_x[col_of(p)], ys[pi]));
            c.component_mut(id).unwrap().label = p.label.clone();
        }
        let pin = |c: &Circuit, (pi, k, out): PinRef| -> Point {
            let comp = &c.components[pi];
            if out {
                comp.output_pins()[k]
            } else {
                comp.input_pins()[k]
            }
        };

        // 5. Vodiče každé sítě.
        // [en] 5. Wires of every net.
        let mut segs: Vec<(Point, Point)> = Vec::new();
        for (si, list) in sinks.iter().enumerate() {
            if list.is_empty() {
                continue;
            }
            let s = Sig(si);
            let (spi, so) = self.sources[si];
            let src_col = col_of(&self.parts[spi]);
            let sp = pin(&c, (spi, so, true));
            let row = row_of.get(&s).copied();
            // Body na stopě v každém kanálu.
            // [en] Points on the track in each channel.
            let mut on_track: BTreeMap<usize, Vec<i32>> = BTreeMap::new();
            let t1 = src_col + 1;
            let tx1 = track_x[&(t1, s)];
            segs.push((sp, Point::new(tx1, sp.y)));
            on_track.entry(t1).or_default().push(sp.y);
            for &r in list {
                let col = col_of(&self.parts[r.0]);
                let p = pin(&c, r);
                let tx = track_x[&(col, s)];
                segs.push((Point::new(tx, p.y), p));
                on_track.entry(col).or_default().push(p.y);
            }
            if let Some(yr) = row {
                for v in on_track.values_mut() {
                    v.push(yr);
                }
                let xs: Vec<i32> = on_track.keys().map(|&k| track_x[&(k, s)]).collect();
                let (x_lo, x_hi) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap());
                segs.push((Point::new(x_lo, yr), Point::new(x_hi, yr)));
            }
            for (k, v) in &on_track {
                let tx = track_x[&(*k, s)];
                let (lo, hi) = (*v.iter().min().unwrap(), *v.iter().max().unwrap());
                segs.push((Point::new(tx, lo), Point::new(tx, hi)));
            }
        }
        for (a, b) in segs {
            if a != b {
                c.add_wire(a, b);
            }
        }

        // 6. Kontrola: netlist schématu musí odpovídat zamýšleným sítím.
        // [en] 6. Check: the schematic's netlist must match the intended nets.
        let nl = Netlist::build(&c);
        let mut seen: BTreeMap<usize, Sig> = BTreeMap::new();
        for (si, list) in sinks.iter().enumerate() {
            let (spi, so) = self.sources[si];
            let net = nl.output_nets[spi][so];
            if let Some(other) = seen.insert(net, Sig(si)) {
                panic!(
                    "sítě {other:?} a {:?} jsou omylem spojené / nets accidentally joined",
                    Sig(si)
                );
            }
            for &(pi, k, _) in list {
                assert_eq!(
                    nl.input_nets[pi][k], net,
                    "síť {si} nedosáhne na vstup {k} součástky {pi} / net does not reach the input"
                );
            }
        }
        c
    }
}
