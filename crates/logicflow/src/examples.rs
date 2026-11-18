//! Vestavěné příklady.

use logicflow_core::{Circuit, ComponentKind, GateKind, Point};

pub struct Example {
    pub name: &'static str,
    pub build: fn() -> Circuit,
}

pub const ALL: [Example; 3] = [
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
];

fn p(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

fn wire_path(c: &mut Circuit, points: &[(i32, i32)]) {
    for w in points.windows(2) {
        c.add_wire(p(w[0].0, w[0].1), p(w[1].0, w[1].1));
    }
}

fn label(c: &mut Circuit, id: logicflow_core::CompId, text: &str) {
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
