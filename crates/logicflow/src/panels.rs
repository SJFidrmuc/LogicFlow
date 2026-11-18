//! Menší editory: vlastnosti, hierarchie, knihovna, pravdivostní tabulka, paměť.

use crate::demo::{DemoCircuit, Item};
use crate::i18n::{Lang, T, tr};
use crate::theme::colors;
use egui::{RichText, Ui};
use logicflow_core::{GateKind, LogicValue};

fn item_name(item: Item) -> &'static str {
    match item {
        Item::SwitchA => "A",
        Item::SwitchB => "B",
        Item::GateAnd => "AND (Y1)",
        Item::GateOr => "OR (Y2)",
        Item::LedY1 => "Y1",
        Item::LedY2 => "Y2",
    }
}

fn padded(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new().inner_margin(8).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        add(ui);
    });
}

pub fn properties(ui: &mut Ui, c: &mut DemoCircuit, lang: Lang) {
    padded(ui, |ui| {
        let Some(item) = c.selected else {
            ui.label(RichText::new(tr(lang, T::NothingSelected)).color(colors::TEXT_WEAK));
            return;
        };
        let (kind, value) = match item {
            Item::SwitchA => (tr(lang, T::Switch), c.va()),
            Item::SwitchB => (tr(lang, T::Switch), c.vb()),
            Item::GateAnd => (tr(lang, T::Gate), c.y1()),
            Item::GateOr => (tr(lang, T::Gate), c.y2()),
            Item::LedY1 => (tr(lang, T::Led), c.y1()),
            Item::LedY2 => (tr(lang, T::Led), c.y2()),
        };
        ui.heading(item_name(item));
        ui.add_space(4.0);
        egui::Grid::new("props")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Typ").color(colors::TEXT_WEAK));
                ui.label(kind);
                ui.end_row();
                ui.label(RichText::new(tr(lang, T::Value)).color(colors::TEXT_WEAK));
                ui.label(RichText::new(value.to_string()).monospace().strong());
                ui.end_row();
                if matches!(item, Item::GateAnd | Item::GateOr) {
                    ui.label(RichText::new("Zpoždění").color(colors::TEXT_WEAK));
                    ui.label("1 ns");
                    ui.end_row();
                }
            });
        if matches!(item, Item::SwitchA | Item::SwitchB) {
            ui.add_space(6.0);
            ui.label(
                RichText::new(tr(lang, T::ClickToToggle))
                    .color(colors::TEXT_WEAK)
                    .small(),
            );
        }
    });
}

pub fn hierarchy(ui: &mut Ui, c: &mut DemoCircuit, lang: Lang) {
    padded(ui, |ui| {
        egui::CollapsingHeader::new(tr(lang, T::MainCircuit))
            .default_open(true)
            .show(ui, |ui| {
                let groups: [(T, &[Item]); 3] = [
                    (T::Inputs, &[Item::SwitchA, Item::SwitchB]),
                    (T::Gates, &[Item::GateAnd, Item::GateOr]),
                    (T::Outputs, &[Item::LedY1, Item::LedY2]),
                ];
                for (title, items) in groups {
                    egui::CollapsingHeader::new(tr(lang, title))
                        .default_open(true)
                        .show(ui, |ui| {
                            for &item in items {
                                let sel = c.selected == Some(item);
                                if ui
                                    .add(egui::Button::selectable(sel, item_name(item)))
                                    .clicked()
                                {
                                    c.selected = Some(item);
                                }
                            }
                        });
                }
            });
    });
}

/// Knihovna součástek: (kategorie, součástky, verze, ve které přibudou).
pub fn library(ui: &mut Ui, search: &mut String, lang: Lang) {
    padded(ui, |ui| {
        ui.add(
            egui::TextEdit::singleline(search)
                .hint_text(tr(lang, T::Search))
                .desired_width(f32::INFINITY),
        );
        ui.add_space(4.0);
        let gates: Vec<String> = GateKind::ALL
            .iter()
            .map(|g| format!("{}  [{}]", g.name(), g.iec_label()))
            .collect();
        let gates: Vec<&str> = gates.iter().map(String::as_str).collect();
        let cats: [(T, Vec<&str>, &str); 6] = [
            (
                T::CatInputs,
                vec!["Přepínač", "Tlačítko", "Generátor hodin", "Konstanta 0/1"],
                "0.1",
            ),
            (
                T::CatOutputs,
                vec!["LED", "Sedmisegmentovka", "Hex displej"],
                "0.1",
            ),
            (T::CatGates, gates, "0.1"),
            (T::CatFlipFlops, vec!["RS", "D", "JK", "T"], "0.2"),
            (T::CatMemory, vec!["ROM", "RAM", "EEPROM"], "0.4"),
            (
                T::Cat74xx,
                vec!["7400", "7404", "7408", "7432", "7474", "74138", "74161"],
                "0.6",
            ),
        ];
        let q = search.to_lowercase();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (title, items, ver) in cats {
                let shown: Vec<&&str> = items
                    .iter()
                    .filter(|n| q.is_empty() || n.to_lowercase().contains(&q))
                    .collect();
                if shown.is_empty() {
                    continue;
                }
                egui::CollapsingHeader::new(tr(lang, title))
                    .default_open(true)
                    .show(ui, |ui| {
                        for name in shown {
                            ui.label(*name)
                                .on_hover_text(format!("{} {ver}", tr(lang, T::ComingIn)));
                        }
                    });
            }
        });
    });
}

pub fn truth_table(ui: &mut Ui, c: &DemoCircuit) {
    padded(ui, |ui| {
        ui.label(RichText::new("Y1 = A · B    Y2 = A + B").monospace());
        ui.add_space(6.0);
        egui::Grid::new("truth")
            .striped(true)
            .spacing([18.0, 6.0])
            .show(ui, |ui| {
                for h in ["A", "B", "Y1", "Y2"] {
                    ui.label(RichText::new(h).strong().monospace());
                }
                ui.end_row();
                for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
                    let (va, vb): (LogicValue, LogicValue) = (a.into(), b.into());
                    let current = a == c.a && b == c.b;
                    let row = [
                        va,
                        vb,
                        GateKind::And.eval(&[va, vb]),
                        GateKind::Or.eval(&[va, vb]),
                    ];
                    for v in row {
                        let mut t = RichText::new(v.to_string()).monospace();
                        if current {
                            t = t.strong().background_color(colors::ACCENT);
                        }
                        ui.label(t);
                    }
                    ui.end_row();
                }
            });
        ui.add_space(6.0);
        ui.label(
            RichText::new("Zvýrazněný řádek odpovídá aktuálním vstupům.")
                .color(colors::TEXT_WEAK)
                .small(),
        );
    });
}

/// Ukázkový obsah ROM: dekodér pro sedmisegmentovku (segmenty g…a), číslice 0–F.
const SEVEN_SEG: [u8; 16] = [
    0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F, 0x77, 0x7C, 0x39, 0x5E, 0x79, 0x71,
];

pub fn memory(ui: &mut Ui, lang: Lang) {
    padded(ui, |ui| {
        ui.label(RichText::new("ROM 16 × 8 – dekodér pro sedmisegmentovku").strong());
        ui.label(
            RichText::new(format!("{} 0.4", tr(lang, T::ComingIn)))
                .color(colors::TEXT_WEAK)
                .small(),
        );
        ui.add_space(6.0);
        egui::Grid::new("mem")
            .striped(true)
            .spacing([14.0, 4.0])
            .show(ui, |ui| {
                for h in ["Adresa", "Hex", "Bin (gfedcba)"] {
                    ui.label(RichText::new(h).color(colors::TEXT_WEAK));
                }
                ui.end_row();
                for (addr, byte) in SEVEN_SEG.iter().enumerate() {
                    ui.label(
                        RichText::new(format!("{addr:02X}"))
                            .monospace()
                            .color(colors::TEXT_WEAK),
                    );
                    ui.label(RichText::new(format!("{byte:02X}")).monospace());
                    ui.label(RichText::new(format!("{byte:08b}")).monospace());
                    ui.end_row();
                }
            });
    });
}
