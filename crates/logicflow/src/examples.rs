//! Vestavěné příklady.

use logicflow_core::{
    BlockKind, Circuit, CompId, ComponentKind, FlipFlopKind, GateKind, Point, Rotation,
    SubcircuitDef,
};

pub struct Example {
    pub name: &'static str,
    pub build: fn() -> Circuit,
    /// Podobvody, které příklad používá.
    pub subcircuits: fn() -> Vec<SubcircuitDef>,
}

pub const ALL: [Example; 9] = [
    Example {
        name: "Hradla AND a OR",
        build: and_or,
        subcircuits: no_subs,
    },
    Example {
        name: "Poloviční sčítačka",
        build: half_adder,
        subcircuits: no_subs,
    },
    Example {
        name: "Klopný obvod RS z hradel NOR",
        build: sr_latch,
        subcircuits: no_subs,
    },
    Example {
        name: "Klopný obvod D s hodinami",
        build: d_flip_flop,
        subcircuits: no_subs,
    },
    Example {
        name: "Čítač 0–7 z klopných obvodů T",
        build: counter,
        subcircuits: no_subs,
    },
    Example {
        name: "Posuvný registr 4 bity",
        build: shift_register,
        subcircuits: no_subs,
    },
    Example {
        name: "Multiplexor 4:1",
        build: multiplexer,
        subcircuits: no_subs,
    },
    Example {
        name: "Dekadický čítač se sedmisegmentovkou",
        build: decade_counter,
        subcircuits: no_subs,
    },
    Example {
        name: "Úplná sčítačka ze dvou podobvodů",
        build: full_adder,
        subcircuits: half_adder_subcircuit,
    },
];

fn no_subs() -> Vec<SubcircuitDef> {
    Vec::new()
}

fn p(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

fn wire_path(c: &mut Circuit, points: &[(i32, i32)]) {
    for w in points.windows(2) {
        c.add_wire(p(w[0].0, w[0].1), p(w[1].0, w[1].1));
    }
}

fn rotate(c: &mut Circuit, id: CompId, r: Rotation) {
    if let Some(k) = c.component_mut(id) {
        k.rotation = r;
    }
}

fn label(c: &mut Circuit, id: CompId, text: &str) {
    if let Some(k) = c.component_mut(id) {
        k.label = text.to_owned();
    }
}

/// Dva vstupy do dvou hradel; společné zapojení pro AND/OR a sčítačku.
fn two_gates(top: GateKind, bottom: GateKind, out_top: &str, out_bottom: &str) -> Circuit {
    let mut c = Circuit::default();
    c.add_component(ComponentKind::Switch { on: true }, p(0, 0));
    c.add_component(ComponentKind::Switch { on: false }, p(0, 10));
    c.add_component(ComponentKind::gate(top), p(14, 1));
    c.add_component(ComponentKind::gate(bottom), p(14, 9));
    let y1 = c.add_component(ComponentKind::Led, p(22, 1));
    let y2 = c.add_component(ComponentKind::Led, p(22, 9));
    label(&mut c, y1, out_top);
    label(&mut c, y2, out_bottom);
    // A → horní vstup obou hradel
    wire_path(&mut c, &[(2, 0), (11, 0)]);
    wire_path(&mut c, &[(6, 0), (6, 8), (11, 8)]);
    // B → dolní vstup obou hradel
    wire_path(&mut c, &[(2, 10), (11, 10)]);
    wire_path(&mut c, &[(8, 10), (8, 2), (11, 2)]);
    // výstupy
    wire_path(&mut c, &[(17, 1), (20, 1)]);
    wire_path(&mut c, &[(17, 9), (20, 9)]);
    c
}

pub fn and_or() -> Circuit {
    two_gates(GateKind::And, GateKind::Or, "Y1", "Y2")
}

fn half_adder() -> Circuit {
    two_gates(GateKind::Xor, GateKind::And, "S", "C")
}

fn sr_latch() -> Circuit {
    let mut c = Circuit::default();
    let s = c.add_component(ComponentKind::Switch { on: false }, p(0, 0));
    let r = c.add_component(ComponentKind::Switch { on: false }, p(0, 10));
    label(&mut c, s, "S");
    label(&mut c, r, "R");
    c.add_component(ComponentKind::gate(GateKind::Nor), p(10, 0));
    c.add_component(ComponentKind::gate(GateKind::Nor), p(10, 10));
    let qn = c.add_component(ComponentKind::Led, p(21, 0));
    let q = c.add_component(ComponentKind::Led, p(21, 10));
    label(&mut c, qn, "Qn");
    label(&mut c, q, "Q");
    wire_path(&mut c, &[(2, 0), (4, 0), (4, -1), (7, -1)]);
    wire_path(&mut c, &[(2, 10), (4, 10), (4, 11), (7, 11)]);
    // výstupy do LED
    wire_path(&mut c, &[(13, 0), (19, 0)]);
    wire_path(&mut c, &[(13, 10), (19, 10)]);
    // zpětné vazby (křížem)
    wire_path(&mut c, &[(15, 0), (15, 5), (6, 5), (6, 9), (7, 9)]);
    wire_path(&mut c, &[(17, 10), (17, 3), (5, 3), (5, 1), (7, 1)]);
    c
}

fn d_flip_flop() -> Circuit {
    let mut c = Circuit::default();
    let d = c.add_component(ComponentKind::Switch { on: true }, p(0, -1));
    label(&mut c, d, "D");
    c.add_component(ComponentKind::Clock { period_ms: 2000 }, p(0, 5));
    c.add_component(ComponentKind::flip_flop(FlipFlopKind::D), p(12, 0));
    let q = c.add_component(ComponentKind::Led, p(20, -1));
    let qn = c.add_component(ComponentKind::Led, p(20, 3));
    label(&mut c, q, "Q");
    label(&mut c, qn, "Qn");
    wire_path(&mut c, &[(2, -1), (9, -1)]);
    wire_path(&mut c, &[(3, 5), (6, 5), (6, 1), (9, 1)]);
    wire_path(&mut c, &[(15, -1), (18, -1)]);
    wire_path(&mut c, &[(15, 1), (16, 1), (16, 3), (18, 3)]);
    c
}

/// LED nad vodičem: vstup míří dolů na bod (x, y + 2).
fn led_above(c: &mut Circuit, x: i32, y: i32, text: &str) {
    let id = c.add_component(ComponentKind::Led, p(x, y));
    rotate(c, id, Rotation::R270);
    label(c, id, text);
}

/// Asynchronní čítač nahoru: každý další stupeň je hodinován výstupem Q̄ předchozího.
pub fn counter() -> Circuit {
    let mut c = Circuit::default();
    c.add_component(ComponentKind::Clock { period_ms: 500 }, p(0, 5));
    c.add_component(ComponentKind::Constant { value: true }, p(0, -5));
    for k in 0..3 {
        c.add_component(ComponentKind::flip_flop(FlipFlopKind::T), p(12 + 12 * k, 0));
    }
    // hodiny → C prvního stupně
    wire_path(&mut c, &[(3, 5), (6, 5), (6, 1), (9, 1)]);
    // konstanta 1 → všechny vstupy T
    wire_path(&mut c, &[(2, -5), (31, -5)]);
    for k in 0..3 {
        let x = 7 + 12 * k;
        wire_path(&mut c, &[(x, -5), (x, -1), (x + 2, -1)]);
    }
    // Q̄ → C dalšího stupně
    wire_path(&mut c, &[(15, 1), (21, 1)]);
    wire_path(&mut c, &[(27, 1), (33, 1)]);
    // výstupy Q do LED
    for k in 0..3 {
        let x = 15 + 12 * k;
        wire_path(&mut c, &[(x, -1), (x + 1, -1), (x + 1, -10)]);
        led_above(&mut c, x + 1, -12, &format!("Q{k}"));
    }
    c
}

pub fn shift_register() -> Circuit {
    let mut c = Circuit::default();
    let d = c.add_component(ComponentKind::Switch { on: true }, p(0, -1));
    label(&mut c, d, "D");
    c.add_component(ComponentKind::Clock { period_ms: 1000 }, p(0, 6));
    for k in 0..4 {
        c.add_component(ComponentKind::flip_flop(FlipFlopKind::D), p(12 + 12 * k, 0));
    }
    wire_path(&mut c, &[(2, -1), (9, -1)]);
    // hodinová sběrnice a přípojky k vstupům C
    wire_path(&mut c, &[(3, 6), (43, 6)]);
    for k in 0..4 {
        let x = 7 + 12 * k;
        wire_path(&mut c, &[(x, 6), (x, 1), (x + 2, 1)]);
    }
    // Q → D dalšího stupně, odbočka do LED
    for k in 0..4 {
        let x = 15 + 12 * k;
        let end = if k < 3 { x + 6 } else { x + 2 };
        wire_path(&mut c, &[(x, -1), (end, -1)]);
        wire_path(&mut c, &[(x + 2, -1), (x + 2, -8)]);
        led_above(&mut c, x + 2, -10, &format!("Q{k}"));
    }
    c
}

/// Multiplexor 4:1 – adresa S1 S0 vybírá jeden ze vstupů D0–D3.
pub fn multiplexer() -> Circuit {
    let mut c = Circuit::default();
    let names = ["S0", "S1", "D0", "D1", "D2", "D3"];
    for (k, name) in names.iter().enumerate() {
        let y = -5 + 2 * k as i32;
        let sw = c.add_component(ComponentKind::Switch { on: k == 3 }, p(0, y));
        label(&mut c, sw, name);
        wire_path(&mut c, &[(2, y), (10, y)]);
    }
    c.add_component(
        ComponentKind::Block {
            block: BlockKind::Mux { select: 2 },
        },
        p(14, 0),
    );
    let y = c.add_component(ComponentKind::Led, p(22, -5));
    label(&mut c, y, "Y");
    wire_path(&mut c, &[(18, -5), (20, -5)]);
    c
}

/// Čítač 0–9: 4bitový čítač nulovaný hradlem AND při stavu 10 (Q3 a Q1),
/// převodník BCD na 7 segmentů a sedmisegmentovka.
pub fn decade_counter() -> Circuit {
    let mut c = Circuit::default();
    c.add_component(ComponentKind::Clock { period_ms: 500 }, p(0, 1));
    c.add_component(
        ComponentKind::Block {
            block: BlockKind::Counter {
                bits: 4,
                down: false,
            },
        },
        p(12, 0),
    );
    // hodiny → C
    wire_path(&mut c, &[(3, 1), (8, 1)]);
    // Q0–Q3 → D0–D3 převodníku
    for k in 0..4 {
        let (y, x) = (-3 + 2 * k, 20 + k);
        wire_path(&mut c, &[(16, y), (x, y), (x, y - 2), (26, y - 2)]);
    }
    c.add_component(
        ComponentKind::Block {
            block: BlockKind::BcdTo7Seg,
        },
        p(30, 0),
    );
    // segmenty a–g
    for y in (-5..=7).step_by(2) {
        wire_path(&mut c, &[(34, y), (38, y)]);
    }
    let disp = c.add_component(
        ComponentKind::SevenSegment {
            pins: logicflow_core::PinSide::Left,
        },
        p(42, 2),
    );
    label(&mut c, disp, "Y1");
    // Nulování při 10 (1010): AND(Q1, Q3) → R
    c.add_component(ComponentKind::gate(GateKind::And), p(4, -8));
    wire_path(&mut c, &[(7, -8), (7, -3), (8, -3)]);
    wire_path(&mut c, &[(18, -1), (18, -12), (-1, -12), (-1, -9), (1, -9)]);
    wire_path(&mut c, &[(19, 3), (19, -14), (-2, -14), (-2, -7), (1, -7)]);
    c
}

/// Podobvod „Poloviční sčítačka“ (id 1): vstupy A, B; výstupy C (přenos), S (součet).
pub fn half_adder_subcircuit() -> Vec<SubcircuitDef> {
    let mut c = half_adder();
    // Výstupy pojmenované podle významu: C = přenos (AND), S = součet (XOR).
    let names: Vec<CompId> = c
        .components
        .iter()
        .filter(|k| matches!(k.kind, ComponentKind::Led))
        .map(|k| k.id)
        .collect();
    label(&mut c, names[0], "S");
    label(&mut c, names[1], "C");
    vec![SubcircuitDef {
        id: 1,
        name: "Poloviční sčítačka".to_owned(),
        circuit: c,
    }]
}

/// Úplná sčítačka: dvě instance podobvodu „Poloviční sčítačka“ a hradlo OR.
pub fn full_adder() -> Circuit {
    let mut c = Circuit::default();
    let half = ComponentKind::Subcircuit {
        id: 1,
        inputs: 2,
        outputs: 2,
    };
    for (y, on, name) in [(0, true, "A"), (2, true, "B"), (6, false, "CI")] {
        let sw = c.add_component(ComponentKind::Switch { on }, p(0, y));
        label(&mut c, sw, name);
    }
    // HA1: vstupy (10,0), (10,2); výstupy C1 (18,0), S1 (18,2)
    c.add_component(half, p(14, 0));
    // HA2: vstupy (24,2), (24,4); výstupy C2 (32,2), S (32,4)
    c.add_component(half, p(28, 2));
    // OR: vstupy (37,-6), (37,-4); výstup (43,-5)
    c.add_component(ComponentKind::gate(GateKind::Or), p(40, -5));
    wire_path(&mut c, &[(2, 0), (10, 0)]);
    wire_path(&mut c, &[(2, 2), (10, 2)]);
    wire_path(&mut c, &[(2, 6), (22, 6), (22, 4), (24, 4)]);
    wire_path(&mut c, &[(18, 2), (24, 2)]);
    wire_path(&mut c, &[(18, 0), (20, 0), (20, -6), (37, -6)]);
    wire_path(&mut c, &[(32, 2), (34, 2), (34, -4), (37, -4)]);
    let s = c.add_component(ComponentKind::Led, p(36, 4));
    label(&mut c, s, "S");
    wire_path(&mut c, &[(32, 4), (34, 4)]);
    let co = c.add_component(ComponentKind::Led, p(47, -5));
    label(&mut c, co, "CO");
    wire_path(&mut c, &[(43, -5), (45, -5)]);
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use logicflow_core::{LogicValue, Simulator};

    fn leds(c: &Circuit) -> Vec<LogicValue> {
        let mut sim = Simulator::new(c, None);
        assert!(sim.run(10_000));
        c.components
            .iter()
            .filter(|k| matches!(k.kind, ComponentKind::Led))
            .map(|k| sim.inputs(k.id)[0])
            .collect()
    }

    #[test]
    fn examples_compute_expected_values() {
        // A = 1, B = 0
        assert_eq!(leds(&and_or()), [LogicValue::Zero, LogicValue::One]);
        assert_eq!(leds(&half_adder()), [LogicValue::One, LogicValue::Zero]);
        // RS bez nastavení: neurčitý stav.
        assert_eq!(leds(&sr_latch()), [LogicValue::X, LogicValue::X]);
    }

    /// Hodnota čítače z LED Q0..Q2.
    fn count(p: &crate::project::Project) -> u32 {
        (0..3)
            .map(|k| {
                let id = p
                    .circuit
                    .components
                    .iter()
                    .find(|c| c.label == format!("Q{k}"))
                    .unwrap()
                    .id;
                u32::from(p.signal_value(id) == LogicValue::One) << k
            })
            .sum()
    }

    #[test]
    fn counter_counts_up() {
        let mut p = crate::project::Project::new(counter());
        // Hodiny 2 Hz: náběžná hrana v čase 0,25; 0,75; 1,25; …
        let mut seen = Vec::new();
        for _ in 0..9 {
            p.update(0.5);
            seen.push(count(&p));
        }
        assert_eq!(seen, [1, 2, 3, 4, 5, 6, 7, 0, 1]);
    }

    #[test]
    fn shift_register_shifts() {
        let mut p = crate::project::Project::new(shift_register());
        let q = |p: &crate::project::Project, k: usize| {
            let id = p
                .circuit
                .components
                .iter()
                .find(|c| c.label == format!("Q{k}"))
                .unwrap()
                .id;
            p.signal_value(id)
        };
        // Hodiny 1 Hz: náběžné hrany v 0,5 s, 1,5 s, …
        p.update(0.75);
        assert_eq!(q(&p, 0), LogicValue::One);
        assert_eq!(q(&p, 1), LogicValue::Zero);
        p.update(1.0);
        p.update(1.0);
        assert_eq!(q(&p, 2), LogicValue::One);
        assert_eq!(q(&p, 3), LogicValue::Zero);
    }

    #[test]
    fn d_example_builds() {
        let mut p = crate::project::Project::new(d_flip_flop());
        p.update(1.5);
        let qid = p
            .circuit
            .components
            .iter()
            .find(|c| c.label == "Q")
            .unwrap()
            .id;
        assert_eq!(p.signal_value(qid), LogicValue::One);
    }

    #[test]
    fn multiplexer_selects_d1() {
        // S1 S0 = 01 → vybraný D1, který je jediný zapnutý.
        let mut c = multiplexer();
        assert_eq!(leds(&c), [LogicValue::Zero]);
        c.components[0].kind = ComponentKind::Switch { on: true };
        assert_eq!(leds(&c), [LogicValue::One]);
    }

    #[test]
    fn decade_counter_counts_to_nine() {
        let mut p = crate::project::Project::new(decade_counter());
        let ctr = p
            .circuit
            .components
            .iter()
            .find(|c| matches!(c.kind, ComponentKind::Block { block } if block.is_sequential()))
            .unwrap()
            .id;
        let disp = p
            .circuit
            .components
            .iter()
            .find(|c| matches!(c.kind, ComponentKind::SevenSegment { .. }))
            .unwrap()
            .id;
        let mut seen = Vec::new();
        for _ in 0..12 {
            p.update(0.5);
            let st = p.sim.block_state(ctr).unwrap();
            seen.push(logicflow_core::block::number(&st).unwrap());
        }
        assert_eq!(seen, [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);
        // Na displeji je „2“: segmenty a, b, d, e, g.
        let segs: Vec<bool> = p.sim.inputs(disp)[..7]
            .iter()
            .map(|v| *v == LogicValue::One)
            .collect();
        assert_eq!(segs, [true, true, false, true, true, false, true]);
    }

    #[test]
    fn full_adder_from_subcircuits() {
        let subs = half_adder_subcircuit();
        assert_eq!(subs[0].pin_names().1, ["C", "S"]);
        let p = crate::project::Project::with_subcircuits(full_adder(), subs);
        let table = crate::panels::compute_truth_table(&p).unwrap();
        assert_eq!(table.inputs, ["A", "B", "CI"]);
        assert_eq!(table.outputs, ["CO", "S"]);
        for (bits, outs) in &table.rows {
            let n = bits.iter().filter(|b| **b).count();
            let expect = [n >= 2, n % 2 == 1].map(LogicValue::from_bool);
            assert_eq!(outs[..], expect, "{bits:?}");
        }
    }

    /// Rozdělení vývodů do sítí: pro každý vývod (součástka, index) číslo
    /// nejmenšího vývodu ve stejné síti.
    fn partition(c: &Circuit) -> std::collections::BTreeMap<(u32, usize), (u32, usize)> {
        let nl = logicflow_core::sim::Netlist::build(c);
        let mut first: std::collections::HashMap<usize, (u32, usize)> = Default::default();
        let mut out = std::collections::BTreeMap::new();
        for (i, k) in c.components.iter().enumerate() {
            let nets = nl.input_nets[i].iter().chain(&nl.output_nets[i]);
            for (j, &n) in nets.enumerate() {
                let key = (k.id.0, j);
                let rep = *first.entry(n).or_insert(key);
                out.insert(key, rep);
            }
        }
        out
    }

    /// Nespojil posun vývody, které předtím byly v různých sítích?
    fn assert_no_new_connections(before: &Circuit, after: &Circuit, what: &str) {
        let (b, a) = (partition(before), partition(after));
        for (pin, rep) in &a {
            assert!(b[pin] == b[rep], "{what}: spojil {pin:?} s {rep:?}");
        }
    }

    #[test]
    fn moving_converter_with_display_keeps_all_connections() {
        let c = decade_counter();
        let ids: Vec<CompId> = c
            .components
            .iter()
            .filter(|k| {
                matches!(
                    k.kind,
                    ComponentKind::SevenSegment { .. }
                        | ComponentKind::Block {
                            block: BlockKind::BcdTo7Seg
                        }
                )
            })
            .map(|k| k.id)
            .collect();
        for dy in [-4, -2, -1, 1, 2, 4] {
            let m = c.moved(&ids, &[], 0, dy);
            assert_eq!(partition(&c), partition(&m), "posun o {dy}");
        }
    }

    #[test]
    fn moving_never_creates_new_connections() {
        let offsets: Vec<(i32, i32)> = (-6..=6)
            .flat_map(|dy| [(0, dy), (1, dy), (-2, dy), (3, dy)])
            .collect();
        for build in [
            decade_counter as fn() -> Circuit,
            full_adder,
            multiplexer,
            counter,
            shift_register,
        ] {
            let c = build();
            // Jednotlivé součástky i dvojice sousedních (např. převodník + displej).
            let mut groups: Vec<Vec<CompId>> = c.components.iter().map(|k| vec![k.id]).collect();
            groups.extend(c.components.windows(2).map(|w| vec![w[0].id, w[1].id]));
            for group in groups {
                for &(dx, dy) in &offsets {
                    // Součástka položená vývodem přímo na cizí vývod nebo vodič se
                    // spojí záměrně – to není chyba přepojení vodičů.
                    let lands = c
                        .components
                        .iter()
                        .filter(|k| group.contains(&k.id))
                        .any(|k| {
                            k.pins().iter().any(|p| {
                                let q = p.offset(dx, dy);
                                c.wires.iter().any(|w| w.contains(q))
                                    || c.components
                                        .iter()
                                        .any(|o| !group.contains(&o.id) && o.pins().contains(&q))
                            })
                        });
                    if lands {
                        continue;
                    }
                    let m = c.moved(&group, &[], dx, dy);
                    assert_no_new_connections(&c, &m, &format!("{group:?} o ({dx},{dy})"));
                }
            }
        }
    }

    #[test]
    fn sr_latch_example_sets() {
        let mut c = sr_latch();
        let s = c.components[0].id;
        if let Some(k) = c.component_mut(s) {
            k.kind = ComponentKind::Switch { on: true };
        }
        assert_eq!(leds(&c), [LogicValue::Zero, LogicValue::One]);
    }
}
