//! Vestavěné příklady.

use logicflow_core::{Circuit, CompId, ComponentKind, FlipFlopKind, GateKind, Point, Rotation};

pub struct Example {
    pub name: &'static str,
    pub build: fn() -> Circuit,
}

pub const ALL: [Example; 6] = [
    Example {
        name: "Hradla AND a OR",
        build: and_or,
    },
    Example {
        name: "Poloviční sčítačka",
        build: half_adder,
    },
    Example {
        name: "Klopný obvod RS z hradel NOR",
        build: sr_latch,
    },
    Example {
        name: "Klopný obvod D s hodinami",
        build: d_flip_flop,
    },
    Example {
        name: "Čítač 0–7 z klopných obvodů T",
        build: counter,
    },
    Example {
        name: "Posuvný registr 4 bity",
        build: shift_register,
    },
];

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
    fn sr_latch_example_sets() {
        let mut c = sr_latch();
        let s = c.components[0].id;
        if let Some(k) = c.component_mut(s) {
            k.kind = ComponentKind::Switch { on: true };
        }
        assert_eq!(leds(&c), [LogicValue::Zero, LogicValue::One]);
    }
}
