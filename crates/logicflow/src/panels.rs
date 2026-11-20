//! Menší editory: vlastnosti, hierarchie, knihovna, pravdivostní tabulka, paměť.

use crate::catalog::{self, ff_name, kind_name};
use crate::i18n::{Lang, T, tr};
use crate::project::{Project, natural_cmp};
use crate::theme::colors;
use egui::{RichText, Ui};
use logicflow_core::{BlockKind, ComponentKind, FlipFlopKind, GateKind, LogicValue};

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
    let mut open_sub = None;
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
                ComponentKind::FlipFlop {
                    ff,
                    falling,
                    initial,
                } => {
                    ui.label(weak(tr(lang, T::FlipFlop)));
                    let mut new_ff = ff;
                    egui::ComboBox::from_id_salt("ff_kind")
                        .selected_text(ff_name(ff, lang))
                        .show_ui(ui, |ui| {
                            for f in FlipFlopKind::ALL {
                                ui.selectable_value(&mut new_ff, f, ff_name(f, lang));
                            }
                        });
                    ui.end_row();

                    ui.label(weak(tr(lang, T::Trigger)));
                    let mut new_falling = falling;
                    if ff.is_edge_triggered() {
                        ui.vertical(|ui| {
                            ui.radio_value(&mut new_falling, false, tr(lang, T::RisingEdge));
                            ui.radio_value(&mut new_falling, true, tr(lang, T::FallingEdge));
                        });
                    } else if ff == FlipFlopKind::DLatch {
                        ui.label(tr(lang, T::LevelTrigger));
                    } else {
                        ui.label("–");
                    }
                    ui.end_row();

                    ui.label(weak(tr(lang, T::InitialState)));
                    let mut new_initial = initial;
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut new_initial, false, "0");
                        ui.radio_value(&mut new_initial, true, "1");
                    });
                    ui.end_row();

                    if let Some(q) = project.sim.flip_flop_state(id) {
                        ui.label(weak(tr(lang, T::StateQ)));
                        ui.label(value_text(q));
                        ui.end_row();
                    }

                    if (new_ff, new_falling, new_initial) != (ff, falling, initial) {
                        project.checkpoint();
                        if let Some(c) = project.circuit.component_mut(id) {
                            c.kind = ComponentKind::FlipFlop {
                                ff: new_ff,
                                falling: new_falling,
                                initial: new_initial,
                            };
                        }
                        structure_changed = true;
                    }
                }
                ComponentKind::Clock { period_ms } => {
                    ui.label(weak(tr(lang, T::Frequency)));
                    let mut new_period = period_ms;
                    let current = 1000.0 / period_ms.max(1) as f32;
                    egui::ComboBox::from_id_salt("clock_hz")
                        .selected_text(crate::symbols::format_hz(current))
                        .show_ui(ui, |ui| {
                            for hz in [0.2_f32, 0.5, 1.0, 2.0, 5.0, 10.0] {
                                let p = (1000.0 / hz).round() as u32;
                                ui.selectable_value(
                                    &mut new_period,
                                    p,
                                    crate::symbols::format_hz(hz),
                                );
                            }
                        });
                    if new_period != period_ms {
                        project.checkpoint();
                        if let Some(c) = project.circuit.component_mut(id) {
                            c.kind = ComponentKind::Clock {
                                period_ms: new_period,
                            };
                        }
                        structure_changed = true;
                    }
                    ui.end_row();
                }
                ComponentKind::Block { block } => {
                    let mut new_block = block;
                    if let Some(n) = block.param() {
                        let (lo, hi) = block.param_range();
                        let title =
                            if matches!(block, BlockKind::Mux { .. } | BlockKind::Decoder { .. }) {
                                T::SelectBits
                            } else {
                                T::Bits
                            };
                        ui.label(weak(tr(lang, title)));
                        ui.horizontal(|ui| {
                            let mut m = n;
                            if ui.add_enabled(m > lo, egui::Button::new("−")).clicked() {
                                m -= 1;
                            }
                            ui.label(RichText::new(m.to_string()).monospace());
                            if ui.add_enabled(m < hi, egui::Button::new("+")).clicked() {
                                m += 1;
                            }
                            new_block = block.with_param(m);
                        });
                        ui.end_row();
                    }
                    if let BlockKind::Counter { bits, down } = block {
                        ui.label(weak(tr(lang, T::CountDirection)));
                        let mut d = down;
                        ui.vertical(|ui| {
                            ui.radio_value(&mut d, false, tr(lang, T::CountUp));
                            ui.radio_value(&mut d, true, tr(lang, T::CountDown));
                        });
                        if d != down {
                            new_block = BlockKind::Counter { bits, down: d };
                        }
                        ui.end_row();
                    }
                    if let Some(st) = project.sim.block_state(id) {
                        ui.label(weak(tr(lang, T::BlockState)));
                        let bits: String = st.iter().rev().map(|v| v.symbol()).collect();
                        let text = match logicflow_core::block::number(&st) {
                            Some(n) => format!("{bits}  (= {n})"),
                            None => bits,
                        };
                        ui.label(RichText::new(text).monospace());
                        ui.end_row();
                    }
                    ui.label(weak(tr(lang, T::Delay)));
                    ui.label(format!(
                        "{} {}",
                        logicflow_core::sim::GATE_DELAY,
                        tr(lang, T::Ticks)
                    ));
                    ui.end_row();
                    if new_block != block {
                        project.checkpoint();
                        if let Some(c) = project.circuit.component_mut(id) {
                            c.kind = ComponentKind::Block { block: new_block };
                        }
                        structure_changed = true;
                    }
                }
                ComponentKind::Subcircuit {
                    id: def_id,
                    inputs,
                    outputs,
                } => {
                    ui.label(weak(tr(lang, T::Definition)));
                    let name = project
                        .subcircuit(def_id)
                        .map_or_else(|| "?".to_owned(), |d| d.name.clone());
                    ui.label(RichText::new(name).color(colors::ACCENT));
                    ui.end_row();
                    ui.label(weak(tr(lang, T::Pins)));
                    ui.label(format!(
                        "{inputs} {} / {outputs} {}",
                        tr(lang, T::Inputs).to_lowercase(),
                        tr(lang, T::Outputs).to_lowercase()
                    ));
                    ui.end_row();
                    ui.label("");
                    if ui.button(tr(lang, T::OpenSubcircuit)).clicked() {
                        open_sub = Some(def_id);
                    }
                    ui.end_row();
                }
                ComponentKind::Button
                | ComponentKind::Led
                | ComponentKind::SevenSegment
                | ComponentKind::HexDisplay => {}
            }

            let is_led = matches!(comp.kind, ComponentKind::Led);
            let multi = matches!(
                comp.kind,
                ComponentKind::Block { .. }
                    | ComponentKind::SevenSegment
                    | ComponentKind::HexDisplay
                    | ComponentKind::Subcircuit { .. }
            );
            let shown = if is_led {
                project.sim.inputs(id).first().copied()
            } else {
                project.sim.output_net_value(id)
            };
            if let Some(v) = shown.filter(|_| !multi) {
                ui.label(weak(tr(lang, if is_led { T::Input } else { T::Output })));
                ui.label(value_text(v));
                ui.end_row();
            }
            if matches!(comp.kind, ComponentKind::Gate { .. }) || multi {
                let ins = project.sim.inputs(id);
                ui.label(weak(tr(lang, T::Inputs)));
                ui.horizontal_wrapped(|ui| {
                    for v in ins {
                        ui.label(value_text(v));
                    }
                });
                ui.end_row();
            }
            if matches!(
                comp.kind,
                ComponentKind::Block { .. } | ComponentKind::Subcircuit { .. }
            ) {
                let outs = project.sim.output_net_values(id);
                ui.label(weak(tr(lang, T::Outputs)));
                ui.horizontal_wrapped(|ui| {
                    for v in outs {
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
    if let Some(def_id) = open_sub {
        project.enter_subcircuit(def_id);
        return;
    }
    ui.add_space(8.0);
    if ui.button(tr(lang, T::Delete)).clicked() {
        delete_selection(project);
    }
}

/// Listy projektu: hlavní obvod a podobvody; přepínání, přejmenování a mazání.
fn sheets(ui: &mut Ui, project: &mut Project, lang: Lang) {
    egui::CollapsingHeader::new(tr(lang, T::Sheets))
        .id_salt("sheets")
        .default_open(true)
        .show(ui, |ui| {
            let mut open = None;
            if ui
                .add(egui::Button::selectable(
                    project.sheet.is_none(),
                    tr(lang, T::MainCircuit),
                ))
                .clicked()
            {
                open = Some(None);
            }
            let list: Vec<(u32, String)> = project
                .subcircuits
                .iter()
                .map(|d| (d.id, d.name.clone()))
                .collect();
            for (id, name) in list {
                let r = ui.add(egui::Button::selectable(
                    project.sheet == Some(id),
                    format!("   {name}"),
                ));
                if r.clicked() {
                    open = Some(Some(id));
                }
            }
            if ui
                .button(format!("+ {}", tr(lang, T::NewSubcircuit)))
                .clicked()
            {
                project.new_subcircuit(tr(lang, T::Subcircuit));
            }
            if let Some(target) = open {
                project.open_sheet(target);
            }
            // Nastavení otevřeného podobvodu.
            if let Some(id) = project.sheet {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(weak(tr(lang, T::SubcircuitName)));
                    let mut name = project.sheet_name().unwrap_or_default();
                    if ui
                        .add(egui::TextEdit::singleline(&mut name).desired_width(140.0))
                        .changed()
                    {
                        project.rename_subcircuit(id, name);
                    }
                });
                ui.label(
                    RichText::new(tr(lang, T::SubcircuitHint))
                        .small()
                        .color(colors::TEXT_WEAK),
                );
                ui.horizontal_wrapped(|ui| {
                    if ui.button(tr(lang, T::BackToParent)).clicked() {
                        project.leave_subcircuit();
                    }
                    let used = project.is_subcircuit_used(id);
                    let r = ui.add_enabled(!used, egui::Button::new(tr(lang, T::DeleteSubcircuit)));
                    if used {
                        r.on_disabled_hover_text(tr(lang, T::SubcircuitUsed));
                    } else if r.clicked() {
                        project.delete_subcircuit(id);
                    }
                });
            }
        });
    ui.add_space(4.0);
}

pub fn hierarchy(ui: &mut Ui, project: &mut Project, lang: Lang) {
    padded(ui, |ui| {
        sheets(ui, project, lang);
        let name = project
            .sheet_name()
            .unwrap_or_else(|| project.display_name());
        egui::CollapsingHeader::new(name)
            .default_open(true)
            .show(ui, |ui| {
                for (title, group) in [
                    (T::Inputs, 0),
                    (T::Gates, 1),
                    (T::CatFlipFlops, 3),
                    (T::CatBlocks, 4),
                    (T::CatSubcircuits, 5),
                    (T::Outputs, 2),
                ] {
                    let mut items: Vec<(logicflow_core::CompId, String)> = project
                        .circuit
                        .components
                        .iter()
                        .filter(|c| match c.kind {
                            ComponentKind::Switch { .. }
                            | ComponentKind::Button
                            | ComponentKind::Constant { .. }
                            | ComponentKind::Clock { .. } => group == 0,
                            ComponentKind::Gate { .. } => group == 1,
                            ComponentKind::Led
                            | ComponentKind::SevenSegment
                            | ComponentKind::HexDisplay => group == 2,
                            ComponentKind::FlipFlop { .. } => group == 3,
                            ComponentKind::Block { .. } => group == 4,
                            ComponentKind::Subcircuit { .. } => group == 5,
                        })
                        .map(|c| {
                            let kind = match c.kind {
                                ComponentKind::Subcircuit { id, .. } => project
                                    .subcircuit(id)
                                    .map_or_else(|| kind_name(&c.kind, lang), |d| d.name.clone()),
                                _ => kind_name(&c.kind, lang),
                            };
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
        let subs: Vec<_> = catalog::subcircuit_items(project)
            .into_iter()
            .filter(|(n, _)| matches(n))
            .collect();
        egui::CollapsingHeader::new(tr(lang, T::CatSubcircuits))
            .id_salt("lib-subs")
            .default_open(true)
            .show(ui, |ui| {
                if project.subcircuits.is_empty() {
                    ui.label(weak(tr(lang, T::NoSubcircuits)));
                }
                for (name, kind) in subs {
                    let active = project.pending_place == Some(kind);
                    if ui.add(egui::Button::selectable(active, name)).clicked() {
                        project.pending_place = Some(kind);
                    }
                }
                if ui
                    .button(format!("+ {}", tr(lang, T::NewSubcircuit)))
                    .clicked()
                {
                    project.new_subcircuit(tr(lang, T::Subcircuit));
                }
            });
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
        let mut sim = logicflow_core::Simulator::with_subcircuits(&c, &project.subcircuits, None);
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
