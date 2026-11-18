//! Menší editory: vlastnosti, hierarchie, knihovna, pravdivostní tabulka, paměť.

use crate::catalog::{self, kind_name};
use crate::i18n::{Lang, T, tr};
use crate::project::{Project, natural_cmp};
use crate::theme::colors;
use egui::{RichText, Ui};
use logicflow_core::{ComponentKind, GateKind, LogicValue};

fn padded(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            egui::Frame::new().inner_margin(8).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                add(ui);
            });
        });
}

fn weak(text: &str) -> RichText {
    RichText::new(text).color(colors::TEXT_WEAK)
}

fn value_text(v: LogicValue) -> RichText {
    RichText::new(v.to_string())
        .monospace()
        .strong()
        .color(crate::theme::signal_color(v))
}

fn delete_selection(project: &mut Project) {
    project.checkpoint();
    for id in project.selection.comp_vec() {
        project.circuit.remove_component(id);
    }
    for id in project.selection.wire_vec() {
        project.circuit.remove_wire(id);
    }
    project.selection.clear();
    project.after_structure_change();
}

pub fn properties(ui: &mut Ui, project: &mut Project, lang: Lang) {
    padded(ui, |ui| {
        if project.selection.is_empty() {
            ui.label(weak(tr(lang, T::NothingSelected)));
            return;
        }
        if let Some(id) = project.selection.single_comp() {
            component_properties(ui, project, id, lang);
            return;
        }
        if project.selection.comps.is_empty() && project.selection.wires.len() == 1 {
            let id = *project.selection.wires.first().expect("jeden vodič");
            if let Some(w) = project.circuit.wire(id).copied() {
                ui.heading(tr(lang, T::Wire));
                egui::Grid::new("wire_props")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(weak(tr(lang, T::Length)));
                        ui.label(w.len().to_string());
                        ui.end_row();
                        ui.label(weak(tr(lang, T::Value)));
                        ui.label(value_text(project.sim.wire_value(id)));
                        ui.end_row();
                    });
            }
        } else {
            ui.label(format!(
                "{}: {} · {}: {}",
                tr(lang, T::Components),
                project.selection.comps.len(),
                tr(lang, T::Wires),
                project.selection.wires.len()
            ));
        }
        ui.add_space(8.0);
        if ui.button(tr(lang, T::Delete)).clicked() {
            delete_selection(project);
        }
    });
}

fn component_properties(
    ui: &mut Ui,
    project: &mut Project,
    id: logicflow_core::CompId,
    lang: Lang,
) {
    let Some(comp) = project.circuit.component(id).cloned() else {
        return;
    };
    let title = if comp.label.is_empty() {
        kind_name(&comp.kind, lang)
    } else {
        comp.label.clone()
    };
    ui.heading(title);
    ui.add_space(4.0);

    let mut structure_changed = false;
    egui::Grid::new("props")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label(weak(tr(lang, T::Type)));
            ui.label(kind_name(&comp.kind, lang));
            ui.end_row();

            ui.label(weak(tr(lang, T::Label)));
            let mut label = comp.label.clone();
            let r = ui.add(egui::TextEdit::singleline(&mut label).desired_width(120.0));
            if r.gained_focus() {
                project.checkpoint();
            }
            if r.changed()
                && let Some(c) = project.circuit.component_mut(id)
            {
                c.label = label;
                project.touch();
            }
            ui.end_row();

            match comp.kind {
                ComponentKind::Gate { gate, inputs } => {
                    ui.label(weak(tr(lang, T::Gate)));
                    let mut new_gate = gate;
                    egui::ComboBox::from_id_salt("gate_kind")
                        .selected_text(gate.name())
                        .show_ui(ui, |ui| {
                            for g in GateKind::ALL {
                                ui.selectable_value(
                                    &mut new_gate,
                                    g,
                                    format!("{}  [{}]", g.name(), g.iec_label()),
                                );
                            }
                        });
                    if new_gate != gate {
                        project.checkpoint();
                        let inputs = match (new_gate.is_unary(), gate.is_unary()) {
                            (true, _) => 1,
                            (false, true) => 2,
                            (false, false) => inputs,
                        };
                        if let Some(c) = project.circuit.component_mut(id) {
                            c.kind = ComponentKind::Gate {
                                gate: new_gate,
                                inputs,
                            };
                        }
                        structure_changed = true;
                    }
                    ui.end_row();

                    if !gate.is_unary() {
                        ui.label(weak(tr(lang, T::InputCount)));
                        ui.horizontal(|ui| {
                            let mut n = inputs;
                            if ui
                                .add_enabled(n > ComponentKind::MIN_INPUTS, egui::Button::new("−"))
                                .clicked()
                            {
                                n -= 1;
                            }
                            ui.label(RichText::new(n.to_string()).monospace());
                            if ui
                                .add_enabled(n < ComponentKind::MAX_INPUTS, egui::Button::new("+"))
                                .clicked()
                            {
                                n += 1;
                            }
                            if n != inputs {
                                project.checkpoint();
                                if let Some(c) = project.circuit.component_mut(id) {
                                    c.kind = ComponentKind::Gate { gate, inputs: n };
                                }
                                structure_changed = true;
                            }
                        });
                        ui.end_row();
                    }
                    ui.label(weak(tr(lang, T::Delay)));
                    ui.label(format!(
                        "{} {}",
                        logicflow_core::sim::GATE_DELAY,
                        tr(lang, T::Ticks)
                    ));
                    ui.end_row();
                }
                ComponentKind::Switch { on } => {
                    ui.label(weak(tr(lang, T::Value)));
                    let mut v = on;
                    if ui.checkbox(&mut v, if on { "1" } else { "0" }).changed() {
                        project.toggle_switch(id);
                    }
                    ui.end_row();
                }
                ComponentKind::Constant { value } => {
                    ui.label(weak(tr(lang, T::Value)));
                    ui.horizontal(|ui| {
                        let mut v = value;
                        ui.radio_value(&mut v, false, "0");
                        ui.radio_value(&mut v, true, "1");
                        if v != value {
                            project.checkpoint();
                            if let Some(c) = project.circuit.component_mut(id) {
                                c.kind = ComponentKind::Constant { value: v };
                            }
                            structure_changed = true;
                        }
                    });
                    ui.end_row();
                }
                ComponentKind::Button | ComponentKind::Led => {}
            }

            let is_led = matches!(comp.kind, ComponentKind::Led);
            let shown = if is_led {
                project.sim.inputs(id).first().copied()
            } else {
                project.sim.output_net_value(id)
            };
            if let Some(v) = shown {
                ui.label(weak(tr(lang, if is_led { T::Input } else { T::Output })));
                ui.label(value_text(v));
                ui.end_row();
            }
            if matches!(comp.kind, ComponentKind::Gate { .. }) {
                let ins = project.sim.inputs(id);
                ui.label(weak(tr(lang, T::Inputs)));
                ui.horizontal(|ui| {
                    for v in ins {
                        ui.label(value_text(v));
                    }
                });
                ui.end_row();
            }

            ui.label(weak(tr(lang, T::Rotation)));
            ui.horizontal(|ui| {
                let deg = match comp.rotation {
                    logicflow_core::Rotation::R0 => 0,
                    logicflow_core::Rotation::R90 => 90,
                    logicflow_core::Rotation::R180 => 180,
                    logicflow_core::Rotation::R270 => 270,
                };
                ui.label(format!("{deg}°"));
                if ui.button(tr(lang, T::RotateBtn)).clicked() {
                    project.checkpoint();
                    if let Some(c) = project.circuit.component_mut(id) {
                        c.rotation = c.rotation.next();
                    }
                    structure_changed = true;
                }
            });
            ui.end_row();

            ui.label(weak(tr(lang, T::Position)));
            ui.label(format!("{}, {}", comp.pos.x, comp.pos.y));
            ui.end_row();
        });
    if structure_changed {
        project.after_structure_change();
    }
    ui.add_space(8.0);
    if ui.button(tr(lang, T::Delete)).clicked() {
        delete_selection(project);
    }
}

pub fn hierarchy(ui: &mut Ui, project: &mut Project, lang: Lang) {
    padded(ui, |ui| {
        let name = project.display_name();
        egui::CollapsingHeader::new(name)
            .default_open(true)
            .show(ui, |ui| {
                for (title, group) in [(T::Inputs, 0), (T::Gates, 1), (T::Outputs, 2)] {
                    let mut items: Vec<(logicflow_core::CompId, String)> = project
                        .circuit
                        .components
                        .iter()
                        .filter(|c| match c.kind {
                            ComponentKind::Switch { .. }
                            | ComponentKind::Button
                            | ComponentKind::Constant { .. } => group == 0,
                            ComponentKind::Gate { .. } => group == 1,
                            ComponentKind::Led => group == 2,
                        })
                        .map(|c| {
                            let kind = kind_name(&c.kind, lang);
                            let text = if c.label.is_empty() {
                                kind
                            } else {
                                format!("{}  ·  {kind}", c.label)
                            };
                            (c.id, text)
                        })
                        .collect();
                    if items.is_empty() {
                        continue;
                    }
                    items.sort_by(|a, b| natural_cmp(&a.1, &b.1));
                    egui::CollapsingHeader::new(format!("{} ({})", tr(lang, title), items.len()))
                        .id_salt(("hier", group))
                        .default_open(true)
                        .show(ui, |ui| {
                            for (id, text) in items {
                                let sel = project.selection.comps.contains(&id);
                                if ui.add(egui::Button::selectable(sel, text)).clicked() {
                                    project.selection.clear();
                                    project.selection.comps.insert(id);
                                }
                            }
                        });
                }
                ui.label(weak(&format!(
                    "{}: {}",
                    tr(lang, T::Wires),
                    project.circuit.wires.len()
                )));
            });
    });
}

pub fn library(ui: &mut Ui, search: &mut String, project: &mut Project, lang: Lang) {
    padded(ui, |ui| {
        ui.add(
            egui::TextEdit::singleline(search)
                .hint_text(tr(lang, T::Search))
                .desired_width(f32::INFINITY),
        );
        ui.label(
            RichText::new(tr(lang, T::LibraryHint))
                .small()
                .color(colors::TEXT_WEAK),
        );
        ui.add_space(4.0);
        let q = search.to_lowercase();
        let matches = |n: &str| q.is_empty() || n.to_lowercase().contains(&q);
        for cat in catalog::catalog(lang) {
            let items: Vec<_> = cat.items.into_iter().filter(|(n, _)| matches(n)).collect();
            if items.is_empty() {
                continue;
            }
            egui::CollapsingHeader::new(tr(lang, cat.title))
                .id_salt(("lib", cat.title as u8))
                .default_open(true)
                .show(ui, |ui| {
                    for (name, kind) in items {
                        let active = project.pending_place == Some(kind);
                        if ui.add(egui::Button::selectable(active, name)).clicked() {
                            project.pending_place = Some(kind);
                        }
                    }
                });
        }
        for (title, items, ver) in catalog::upcoming() {
            let items: Vec<_> = items.iter().filter(|n| matches(n)).collect();
            if items.is_empty() {
                continue;
            }
            egui::CollapsingHeader::new(weak(&format!("{} ({ver})", tr(lang, title))))
                .id_salt(("lib-next", title as u8))
                .default_open(false)
                .show(ui, |ui| {
                    for name in items {
                        ui.label(weak(name))
                            .on_hover_text(format!("{} {ver}", tr(lang, T::ComingIn)));
                    }
                });
        }
    });
}

/// Pravdivostní tabulka spočítaná simulací všech kombinací vstupů.
#[derive(Clone, Debug)]
pub struct TruthTable {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub rows: Vec<(Vec<bool>, Vec<LogicValue>)>,
}

pub type TruthCache = Option<(u64, Result<TruthTable, T>)>;

const MAX_TABLE_INPUTS: usize = 8;

pub fn compute_truth_table(project: &Project) -> Result<TruthTable, T> {
    let (ins, outs) = project.io_components();
    if ins.is_empty() || outs.is_empty() {
        return Err(T::TruthNeedsIo);
    }
    if ins.len() > MAX_TABLE_INPUTS {
        return Err(T::TruthTooMany);
    }
    let label = |id| {
        project
            .circuit
            .component(id)
            .map(|c| c.label.clone())
            .unwrap_or_default()
    };
    let mut rows = Vec::new();
    for combo in 0..(1u32 << ins.len()) {
        let bits: Vec<bool> = (0..ins.len())
            .map(|i| (combo >> (ins.len() - 1 - i)) & 1 == 1)
            .collect();
        let mut c = project.circuit.clone();
        for (id, &b) in ins.iter().zip(&bits) {
            if let Some(k) = c.component_mut(*id) {
                match &mut k.kind {
                    ComponentKind::Switch { on } => *on = b,
                    ComponentKind::Button => k.pressed = b,
                    _ => {}
                }
            }
        }
        let mut sim = logicflow_core::Simulator::new(&c, None);
        let stable = sim.run(50_000);
        let values = outs
            .iter()
            .map(|id| {
                if stable {
                    sim.inputs(*id).first().copied().unwrap_or(LogicValue::Z)
                } else {
                    LogicValue::X
                }
            })
            .collect();
        rows.push((bits, values));
    }
    Ok(TruthTable {
        inputs: ins.into_iter().map(label).collect(),
        outputs: outs.into_iter().map(label).collect(),
        rows,
    })
}

pub fn truth_table(ui: &mut Ui, project: &Project, cache: &mut TruthCache, lang: Lang) {
    if cache
        .as_ref()
        .is_none_or(|(rev, _)| *rev != project.revision)
    {
        *cache = Some((project.revision, compute_truth_table(project)));
    }
    let Some((_, table)) = cache.as_ref() else {
        return;
    };
    padded(ui, |ui| {
        let table = match table {
            Ok(t) => t,
            Err(msg) => {
                ui.label(weak(tr(lang, *msg)));
                return;
            }
        };
        // Aktuální stav vstupů pro zvýraznění řádku.
        let (ins, _) = project.io_components();
        let current: Vec<bool> = ins
            .iter()
            .map(|id| project.sim.output(*id) == Some(LogicValue::One))
            .collect();
        egui::Grid::new("truth")
            .striped(true)
            .spacing([16.0, 5.0])
            .show(ui, |ui| {
                for h in &table.inputs {
                    ui.label(RichText::new(h).strong().monospace());
                }
                ui.label(weak("|"));
                for h in &table.outputs {
                    ui.label(RichText::new(h).strong().monospace());
                }
                ui.end_row();
                for (bits, values) in &table.rows {
                    let hl = *bits == current;
                    let cell = |t: String, color| {
                        let mut r = RichText::new(t).monospace().color(color);
                        if hl {
                            r = r.strong().background_color(colors::ACCENT);
                        }
                        r
                    };
                    for b in bits {
                        ui.label(cell(if *b { "1".into() } else { "0".into() }, colors::TEXT));
                    }
                    ui.label(weak("|"));
                    for v in values {
                        let color = if *v == LogicValue::Zero {
                            colors::TEXT
                        } else {
                            crate::theme::signal_color(*v)
                        };
                        ui.label(cell(v.to_string(), color));
                    }
                    ui.end_row();
                }
            });
        ui.add_space(6.0);
        ui.label(
            RichText::new(tr(lang, T::TruthNote))
                .small()
                .color(colors::TEXT_WEAK),
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
        ui.label(weak(&format!("{} 0.4", tr(lang, T::ComingIn))));
        ui.add_space(6.0);
        egui::Grid::new("mem")
            .striped(true)
            .spacing([14.0, 4.0])
            .show(ui, |ui| {
                for h in ["Adresa", "Hex", "Bin (gfedcba)"] {
                    ui.label(weak(h));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truth_table_of_and_or() {
        let p = Project::new(crate::examples::and_or());
        let t = compute_truth_table(&p).unwrap();
        assert_eq!(t.inputs, ["A", "B"]);
        assert_eq!(t.outputs, ["Y1", "Y2"]);
        use LogicValue::*;
        let outs: Vec<_> = t.rows.iter().map(|r| r.1.clone()).collect();
        assert_eq!(
            outs,
            vec![
                vec![Zero, Zero],
                vec![Zero, One],
                vec![Zero, One],
                vec![One, One]
            ]
        );
    }

    #[test]
    fn truth_table_needs_inputs() {
        let p = Project::new(logicflow_core::Circuit::default());
        assert!(matches!(compute_truth_table(&p), Err(T::TruthNeedsIo)));
    }
}
