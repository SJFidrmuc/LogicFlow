//! Menší editory: vlastnosti, hierarchie, knihovna, pravdivostní tabulka, paměť.
//!
//! [en] Smaller editors: properties, hierarchy, library, truth table, memory.

use crate::catalog::{self, ff_name, kind_name};
use crate::commands::{self, Command};
use crate::i18n::{Lang, T, tr};
use crate::libraries::{self, Libraries, Source};
use crate::project::{Project, natural_cmp};
use crate::theme::{colors, signal_color};
use egui::{RichText, Ui};
use logicflow_core::library::LibraryItem;
use logicflow_core::{BlockKind, ComponentKind, FlipFlopKind, GateKind, LogicValue, PinSide};

/// Obsah panelu s posuvníkem a okraji.
///
/// [en] Panel content with a scroll bar and margins.
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

/// Méně výrazný text (popisky polí).
///
/// [en] Less prominent text (field captions).
fn weak(text: &str) -> RichText {
    RichText::new(text).color(colors::TEXT_WEAK)
}

/// Logická hodnota obarvená jako vodič.
///
/// [en] Logic value coloured like a wire.
fn value_text(v: LogicValue) -> RichText {
    RichText::new(v.to_string())
        .monospace()
        .strong()
        .color(crate::theme::signal_color(v))
}

/// Panel Vlastnosti: vlastnosti vybrané součástky, nebo souhrn výběru.
///
/// [en] Properties panel: properties of the selected component, or a summary of the selection.
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
        if ui
            .add_enabled(
                project.can_edit(),
                egui::Button::new(Command::Delete.button_text(lang, project)),
            )
            .clicked()
        {
            project.delete_selection();
        }
    });
}

/// Vlastnosti jedné součástky podle jejího druhu (popisek, otočení, vstupy, bity, frekvence…).
///
/// [en] Properties of one component according to its kind (label, rotation, inputs, bits, frequency…).
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
    // V režimu simulace jde měnit jen přepínač a frekvence hodin.
    // [en] In simulation mode only the switch and the clock frequency can be changed.
    let editable = project.can_edit();
    if !editable {
        ui.label(weak(tr(lang, T::PropsLocked)));
        ui.add_space(4.0);
    }

    let mut structure_changed = false;
    let mut rotate = false;
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
            let r = ui.add_enabled(
                editable,
                egui::TextEdit::singleline(&mut label).desired_width(120.0),
            );
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
                    ui.add_enabled_ui(editable, |ui| {
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
                            if !editable {
                                ui.disable();
                            }
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
                        if !editable {
                            ui.disable();
                        }
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
                    ui.add_enabled_ui(editable, |ui| {
                        egui::ComboBox::from_id_salt("ff_kind")
                            .selected_text(ff_name(ff, lang))
                            .show_ui(ui, |ui| {
                                for f in FlipFlopKind::ALL {
                                    ui.selectable_value(&mut new_ff, f, ff_name(f, lang));
                                }
                            });
                    });
                    ui.end_row();

                    ui.label(weak(tr(lang, T::Trigger)));
                    let mut new_falling = falling;
                    if ff.is_edge_triggered() {
                        ui.vertical(|ui| {
                            if !editable {
                                ui.disable();
                            }
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
                        if !editable {
                            ui.disable();
                        }
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
                    // Frekvence: rychlé volby, posuvník 0,1–100 Hz (logaritmicky, s polem
                    // pro zadání čísla) a perioda v ms. Jde měnit i za běhu simulace,
                    // hodiny navážou bez skoku fáze.
                    // [en] Frequency: quick choices, a slider 0.1–100 Hz (logarithmic, with
                    // a field for typing a number) and the period in ms. It can be changed
                    // while simulating too, the clock continues without a phase jump.
                    ui.label(weak(tr(lang, T::Frequency)));
                    let current = 1000.0 / period_ms.max(1) as f32;
                    let mut chosen = None;
                    egui::ComboBox::from_id_salt("clock_hz")
                        .selected_text(crate::symbols::format_hz(current))
                        .show_ui(ui, |ui| {
                            for hz in crate::project::CLOCK_PRESETS_HZ {
                                let p = crate::project::period_for_hz(hz);
                                if ui
                                    .selectable_label(period_ms == p, crate::symbols::format_hz(hz))
                                    .clicked()
                                {
                                    chosen = Some((p, true));
                                }
                            }
                        });
                    ui.end_row();
                    ui.label("");
                    let mut hz = current;
                    let r = ui.add(
                        egui::Slider::new(&mut hz, 0.1..=100.0)
                            .logarithmic(true)
                            .suffix(" Hz")
                            .max_decimals(2),
                    );
                    if r.changed() {
                        // Zpět jen jednou na začátek tažení, ne při každém pohybu.
                        // [en] Undo only once at the start of a drag, not on every movement.
                        let undo = r.drag_started() || !r.dragged();
                        chosen = Some((crate::project::period_for_hz(hz), undo));
                    }
                    ui.end_row();
                    ui.label(weak(tr(lang, T::Period)));
                    let mut ms = period_ms;
                    let r = ui.add(
                        egui::DragValue::new(&mut ms)
                            .range(
                                crate::project::MIN_CLOCK_PERIOD_MS
                                    ..=crate::project::MAX_CLOCK_PERIOD_MS,
                            )
                            .speed(5)
                            .suffix(" ms"),
                    );
                    if r.changed() {
                        chosen = Some((ms, r.drag_started() || !r.dragged()));
                    }
                    ui.end_row();
                    if let Some((p, undo)) = chosen {
                        project.set_clock_period(id, p, undo);
                    }
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
                            if !editable {
                                ui.disable();
                            }
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
                            if !editable {
                                ui.disable();
                            }
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
                    if ui
                        .button(Command::Subcircuit.button_text(lang, project))
                        .clicked()
                    {
                        open_sub = Some(def_id);
                    }
                    ui.end_row();
                }
                ComponentKind::SevenSegment { pins } | ComponentKind::HexDisplay { pins } => {
                    ui.label(weak(tr(lang, T::PinSideLabel)));
                    let side_name = |p: PinSide| {
                        tr(
                            lang,
                            match p {
                                PinSide::Left => T::SideLeft,
                                PinSide::Right => T::SideRight,
                                PinSide::Top => T::SideTop,
                                PinSide::Bottom => T::SideBottom,
                            },
                        )
                    };
                    let mut new_pins = pins;
                    ui.add_enabled_ui(editable, |ui| {
                        egui::ComboBox::from_id_salt("pin_side")
                            .selected_text(side_name(pins))
                            .show_ui(ui, |ui| {
                                for p in PinSide::ALL {
                                    ui.selectable_value(&mut new_pins, p, side_name(p));
                                }
                            });
                    });
                    if new_pins != pins {
                        project.checkpoint();
                        if let Some(c) = project.circuit.component_mut(id) {
                            c.kind = match c.kind {
                                ComponentKind::SevenSegment { .. } => {
                                    ComponentKind::SevenSegment { pins: new_pins }
                                }
                                _ => ComponentKind::HexDisplay { pins: new_pins },
                            };
                        }
                        structure_changed = true;
                    }
                    ui.end_row();
                }
                ComponentKind::Led { color } | ComponentKind::LedAk { color } => {
                    // Barva (jen vzhled) a počet vývodů: 1 = vstup, 2 = anoda a katoda.
                    // [en] Colour (appearance only) and number of pins: 1 = input,
                    // 2 = anode and cathode.
                    ui.label(weak(tr(lang, T::LedColorLabel)));
                    let mut new_color = color;
                    ui.add_enabled_ui(editable, |ui| {
                        egui::ComboBox::from_id_salt("led_color")
                            .selected_text(catalog::led_color_name(color, lang))
                            .show_ui(ui, |ui| {
                                for c in logicflow_core::LedColor::ALL {
                                    ui.selectable_value(
                                        &mut new_color,
                                        c,
                                        catalog::led_color_name(c, lang),
                                    );
                                }
                            });
                    });
                    ui.end_row();
                    ui.label(weak(tr(lang, T::PinSideLabel)));
                    let two = matches!(comp.kind, ComponentKind::LedAk { .. });
                    let mut new_two = two;
                    ui.vertical(|ui| {
                        if !editable {
                            ui.disable();
                        }
                        ui.radio_value(&mut new_two, false, tr(lang, T::LedPinsOne));
                        ui.radio_value(&mut new_two, true, tr(lang, T::LedPinsTwo));
                    });
                    ui.end_row();
                    if (new_color, new_two) != (color, two) {
                        project.checkpoint();
                        if let Some(c) = project.circuit.component_mut(id) {
                            c.kind = c.kind.with_led_color(new_color).with_led_two_pins(new_two);
                        }
                        structure_changed = true;
                    }
                }
                ComponentKind::Button | ComponentKind::Vcc | ComponentKind::Gnd => {}
            }

            let is_led = comp.kind.is_led();
            let multi = matches!(
                comp.kind,
                ComponentKind::Block { .. }
                    | ComponentKind::SevenSegment { .. }
                    | ComponentKind::HexDisplay { .. }
                    | ComponentKind::Subcircuit { .. }
            );
            let shown = if is_led {
                comp.kind.led_state(&project.sim.inputs(id))
            } else {
                project.sim.output_net_value(id)
            };
            let two_pin_led = matches!(comp.kind, ComponentKind::LedAk { .. });
            if let Some(v) = shown.filter(|_| !multi) {
                // U LED se dvěma vývody je to stav svitu, ne hodnota jednoho vstupu.
                // [en] For a two-pin LED it is the light state, not the value of one input.
                let title = if two_pin_led {
                    T::BlockState
                } else if is_led {
                    T::Input
                } else {
                    T::Output
                };
                ui.label(weak(tr(lang, title)));
                ui.label(value_text(v));
                ui.end_row();
            }
            if matches!(comp.kind, ComponentKind::Gate { .. }) || multi || two_pin_led {
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
                if ui
                    .add_enabled(
                        editable,
                        egui::Button::new(Command::Rotate.button_text(lang, project)),
                    )
                    .clicked()
                {
                    rotate = true;
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
    if rotate {
        project.rotate_selection();
    }
    if let Some(def_id) = open_sub {
        project.enter_subcircuit(def_id);
        return;
    }
    ui.add_space(8.0);
    if ui
        .add_enabled(
            editable,
            egui::Button::new(Command::Delete.button_text(lang, project)),
        )
        .clicked()
    {
        project.delete_selection();
    }
}

/// Listy projektu: hlavní obvod a podobvody; přepínání, přejmenování a mazání.
///
/// [en] Project sheets: main circuit and subcircuits; switching, renaming and deleting.
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
                // Pravé tlačítko na listu: otevřít, smazat nepoužitý podobvod.
                // [en] Right button on a sheet: open, delete an unused subcircuit.
                r.context_menu(|ui| {
                    if ui.button(tr(lang, T::CtxOpen)).clicked() {
                        open = Some(Some(id));
                        ui.close();
                    }
                    let used = project.is_subcircuit_used(id);
                    let del = ui.add_enabled(
                        !used && project.can_edit(),
                        egui::Button::new(tr(lang, T::DeleteSubcircuit)),
                    );
                    let del = if used {
                        del.on_disabled_hover_text(tr(lang, T::SubcircuitUsed))
                    } else {
                        del.on_disabled_hover_text(tr(lang, T::LockedHint))
                    };
                    if del.clicked() {
                        project.delete_subcircuit(id);
                        ui.close();
                    }
                });
            }
            if ui
                .add_enabled(
                    project.can_edit(),
                    egui::Button::new(format!("+ {}", tr(lang, T::NewSubcircuit))),
                )
                .clicked()
            {
                project.new_subcircuit(tr(lang, T::Subcircuit));
            }
            if let Some(target) = open {
                project.open_sheet(target);
            }
            // Nastavení otevřeného podobvodu.
            // [en] Settings of the open subcircuit.
            if let Some(id) = project.sheet {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(weak(tr(lang, T::SubcircuitName)));
                    let mut name = project.sheet_name().unwrap_or_default();
                    if ui
                        .add_enabled(
                            project.can_edit(),
                            egui::TextEdit::singleline(&mut name).desired_width(140.0),
                        )
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
                    if ui
                        .button(commands::with_shortcut(
                            tr(lang, T::CmdBackToParent),
                            Command::Subcircuit,
                            lang,
                        ))
                        .clicked()
                    {
                        project.leave_subcircuit();
                    }
                    let used = project.is_subcircuit_used(id);
                    let r = ui.add_enabled(
                        !used && project.can_edit(),
                        egui::Button::new(tr(lang, T::DeleteSubcircuit)),
                    );
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

/// Informace o obvodu nahoře v Hierarchii: název, autoři a licence, popis v tooltipu.
/// Kliknutím (i pravým tlačítkem) se otevře okno Vlastnosti obvodu.
///
/// [en] Circuit information at the top of the Hierarchy: title, authors and licence,
/// description in the tooltip. A click (also with the right button) opens the
/// Circuit properties window.
fn circuit_info(ui: &mut Ui, project: &mut Project, lang: Lang) {
    let info = &project.info;
    let byline = [info.authors_text(), info.license.clone()]
        .into_iter()
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("  ·  ");
    let mut text = egui::text::LayoutJob::default();
    let font = egui::TextStyle::Body.resolve(ui.style());
    let small = egui::TextStyle::Small.resolve(ui.style());
    if info.title.is_empty() && byline.is_empty() {
        text.append(
            tr(lang, T::InfoNone),
            0.0,
            egui::TextFormat::simple(small, colors::TEXT_WEAK),
        );
    } else {
        if !info.title.is_empty() {
            text.append(
                &info.title,
                0.0,
                egui::TextFormat::simple(font, ui.visuals().strong_text_color()),
            );
        }
        if !byline.is_empty() {
            let sep = if info.title.is_empty() { "" } else { "\n" };
            text.append(
                &format!("{sep}{byline}"),
                0.0,
                egui::TextFormat::simple(small, colors::TEXT_WEAK),
            );
        }
    }
    let mut r = ui
        .add(egui::Button::new(text).frame(false).wrap())
        .on_hover_text(tr(lang, T::PropsMenu));
    if !info.description.is_empty() {
        r = r.on_hover_text(info.description.clone());
    }
    let mut open = r.clicked();
    r.context_menu(|ui| {
        if ui.button(tr(lang, T::PropsMenu)).clicked() {
            open = true;
            ui.close();
        }
    });
    if open {
        project.properties_request = true;
    }
    ui.add_space(4.0);
}

/// Panel Hierarchie: listy projektu a součástky právě upravovaného listu.
///
/// [en] Hierarchy panel: project sheets and components of the sheet being edited.
pub fn hierarchy(ui: &mut Ui, project: &mut Project, lang: Lang) {
    padded(ui, |ui| {
        circuit_info(ui, project, lang);
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
                            | ComponentKind::Vcc
                            | ComponentKind::Gnd
                            | ComponentKind::Clock { .. } => group == 0,
                            ComponentKind::Gate { .. } => group == 1,
                            ComponentKind::Led { .. }
                            | ComponentKind::LedAk { .. }
                            | ComponentKind::SevenSegment { .. }
                            | ComponentKind::HexDisplay { .. } => group == 2,
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
                                let r = ui.add(egui::Button::selectable(sel, text));
                                if r.clicked() {
                                    project.selection.clear();
                                    project.selection.comps.insert(id);
                                }
                                component_context_menu(&r, project, id, lang);
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

/// Kontextová nabídka součástky v Hierarchii: najít ve schématu, přepnout, otevřít
/// podobvod, otočit, smazat.
///
/// [en] Context menu of a component in the Hierarchy: find in schematic, toggle, open
/// subcircuit, rotate, delete.
fn component_context_menu(
    r: &egui::Response,
    project: &mut Project,
    id: logicflow_core::CompId,
    lang: Lang,
) {
    r.context_menu(|ui| {
        project.selection.clear();
        project.selection.comps.insert(id);
        if ui.button(tr(lang, T::CtxFind)).clicked() {
            project.focus_request = Some(id);
            ui.close();
        }
        if matches!(
            project.circuit.component(id).map(|c| c.kind),
            Some(ComponentKind::Switch { .. })
        ) && ui.button(tr(lang, T::CtxToggle)).clicked()
        {
            project.toggle_switch(id);
            ui.close();
        }
        if project.selected_subcircuit().is_some()
            && commands::menu_item(ui, Command::Subcircuit, lang, project)
        {
            project.toggle_subcircuit();
            ui.close();
        }
        ui.separator();
        if commands::menu_item(ui, Command::Rotate, lang, project) {
            project.rotate_selection();
            ui.close();
        }
        if commands::menu_item(ui, Command::Delete, lang, project) {
            project.delete_selection();
            ui.close();
        }
    });
}

/// Knihovna součástek s vyhledáváním; kliknutí vybere součástku k umístění.
/// Pod součástkami a podobvody projektu jsou knihovny prvků.
///
/// [en] Component library with search; a click chooses a component to place.
/// Below the components and the project's subcircuits are the element libraries.
pub fn library(
    ui: &mut Ui,
    search: &mut String,
    project: &mut Project,
    libs: &mut Libraries,
    lang: Lang,
) {
    padded(ui, |ui| {
        // V režimu simulace se součástky nepřidávají.
        // [en] Components are not added in simulation mode.
        if !project.can_edit() {
            ui.label(weak(tr(lang, T::LibraryLocked)));
            ui.add_space(4.0);
            ui.disable();
        }
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
        element_libraries(ui, &q, project, libs, lang);
    });
}

/// Popis prvku knihovny: popis, vývody, autor, licence a verze.
///
/// [en] Description of a library element: description, pins, author, licence and version.
fn item_details(item: &LibraryItem, lang: Lang) -> String {
    let mut lines = Vec::new();
    let desc = item.display_description(lang.code());
    if !desc.is_empty() {
        lines.push(desc.to_owned());
    }
    let (ins, outs) = item.def().pin_names();
    lines.push(format!(
        "{}: {}  →  {}: {}",
        tr(lang, T::LibInputs),
        ins.join(", "),
        tr(lang, T::LibOutputs),
        outs.join(", ")
    ));
    lines.push(format!(
        "{}: {} · {}: {} · {} {}",
        tr(lang, T::LibAuthor),
        item.author,
        tr(lang, T::LibLicense),
        item.license,
        tr(lang, T::LibVersion),
        item.version
    ));
    lines.join("\n")
}

/// Knihovny prvků: vestavěná a osobní, prvky podle kategorií. Kliknutí připraví
/// prvek k vložení (vloží se jako podobvod i s prvky, které používá); pod
/// vybraným prvkem je jeho popis (i pro dotyk, ne jen v tooltipu).
///
/// [en] Element libraries: built-in and personal, elements by category. A click
/// prepares the element for insertion (inserted as a subcircuit together with the
/// elements it uses); below the chosen element is its description (also for touch,
/// not only in a tooltip).
fn element_libraries(
    ui: &mut Ui,
    q: &str,
    project: &mut Project,
    libs: &mut Libraries,
    lang: Lang,
) {
    let code = lang.code();
    let matches = |item: &LibraryItem| {
        q.is_empty()
            || [
                item.display_name(code),
                &item.name,
                &item.category_name(code),
            ]
            .iter()
            .any(|n| n.to_lowercase().contains(q))
    };
    let chosen_id = egui::Id::unique("logicflow-lib-chosen");
    let chosen: Option<String> = ui.data(|d| d.get_temp(chosen_id));
    ui.add_space(6.0);
    ui.separator();
    ui.label(RichText::new(tr(lang, T::LibSection)).strong());
    ui.label(
        RichText::new(tr(lang, T::LibHint))
            .small()
            .color(colors::TEXT_WEAK),
    );
    let mut any = false;
    for loaded in &libs.list {
        let lib = &loaded.library;
        let groups: Vec<_> = libraries::by_category(lib, lang)
            .into_iter()
            .map(|(name, items)| {
                (
                    name,
                    items.into_iter().filter(|i| matches(i)).collect::<Vec<_>>(),
                )
            })
            .filter(|(_, items)| !items.is_empty())
            .collect();
        if groups.is_empty() {
            continue;
        }
        any = true;
        let source = match &loaded.source {
            Source::Builtin => tr(lang, T::LibBuiltin).to_owned(),
            Source::Personal(path) => format!(
                "{} · {}",
                tr(lang, T::LibPersonal),
                path.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default()
            ),
            Source::Shared { file, read_only } => {
                let mut t = format!("{} · {}", tr(lang, T::LibShared), file.display());
                if *read_only {
                    t.push_str(&format!(" · {}", tr(lang, T::SettingsReadOnly)));
                }
                t
            }
        };
        let title = format!("{} ({})", lib.info.display_name(code), lib.items.len());
        egui::CollapsingHeader::new(title)
            .id_salt(("lib-ext", &lib.info.uid, &loaded.source))
            .default_open(false)
            .open((!q.is_empty()).then_some(true))
            .show(ui, |ui| {
                ui.label(weak(&source));
                ui.label(weak(&format!("{} · {}", lib.info.author, lib.info.license)));
                for (cat, items) in groups {
                    egui::CollapsingHeader::new(cat.as_str())
                        .id_salt(("lib-ext-cat", &lib.info.uid, &loaded.source, &cat))
                        .default_open(false)
                        .open((!q.is_empty()).then_some(true))
                        .show(ui, |ui| {
                            for item in items {
                                let active = project
                                    .pending_paste
                                    .as_ref()
                                    .is_some_and(|(uid, _)| uid == &item.uid);
                                let resp = ui
                                    .add(egui::Button::selectable(active, item.display_name(code)))
                                    .on_hover_text(item_details(item, lang));
                                if resp.clicked()
                                    && let Some(text) = lib.insertion_text(&item.uid, code)
                                {
                                    project.pending_place = None;
                                    project.pending_paste = Some((item.uid.clone(), text));
                                    ui.data_mut(|d| d.insert_temp(chosen_id, item.uid.clone()));
                                }
                                if chosen.as_deref() == Some(item.uid.as_str()) {
                                    ui.label(
                                        RichText::new(item_details(item, lang))
                                            .small()
                                            .color(colors::TEXT_WEAK),
                                    );
                                }
                            }
                        });
                }
            });
    }
    if !any && !q.is_empty() {
        ui.label(weak(tr(lang, T::LibNoMatch)));
    }

    // Osobní složka: cesta, otevření ve správci souborů, opětovné načtení, chyby.
    // [en] Personal folder: path, opening in the file manager, reloading, errors.
    ui.add_space(4.0);
    ui.label(RichText::new(tr(lang, T::LibFolder)).small());
    match libs.folder.clone() {
        Some(dir) => {
            ui.label(weak(&dir.display().to_string()));
            ui.horizontal_wrapped(|ui| {
                if ui.button(tr(lang, T::LibOpenFolder)).clicked()
                    && let Err(e) = libraries::open_folder(&dir)
                {
                    libs.problems
                        .push((dir.display().to_string(), libraries::Problem::Io(e)));
                }
                if ui.button(tr(lang, T::LibReload)).clicked() {
                    libs.reload();
                }
                if ui.button(tr(lang, T::LibSettingsButton)).clicked() {
                    libraries::request_settings(ui.ctx());
                }
            });
            if !libs
                .list
                .iter()
                .any(|l| matches!(l.source, Source::Personal(_)))
            {
                ui.label(
                    RichText::new(tr(lang, T::LibEmpty))
                        .small()
                        .color(colors::TEXT_WEAK),
                );
            }
        }
        None => {
            ui.label(weak(tr(lang, T::LibNoFolder)));
        }
    }
    for (file, problem) in &libs.problems {
        ui.label(
            RichText::new(format!(
                "{} {file}: {}",
                tr(lang, T::LibProblem),
                problem.text(lang)
            ))
            .small()
            .color(signal_color(LogicValue::X)),
        );
    }
}

/// Pravdivostní tabulka spočítaná simulací všech kombinací vstupů.
///
/// [en] Truth table computed by simulating all input combinations.
#[derive(Clone, Debug)]
pub struct TruthTable {
    /// Názvy vstupů (sloupce vlevo).
    ///
    /// [en] Input names (columns on the left).
    pub inputs: Vec<String>,
    /// Názvy výstupů (sloupce vpravo).
    ///
    /// [en] Output names (columns on the right).
    pub outputs: Vec<String>,
    /// Řádky: kombinace vstupů a výsledné výstupy.
    ///
    /// [en] Rows: input combinations and the resulting outputs.
    pub rows: Vec<(Vec<bool>, Vec<LogicValue>)>,
}

impl TruthTable {
    /// Tabulka jako text oddělený tabulátory (první řádek názvy sloupců) – po vložení
    /// do tabulkového procesoru nebo Wordu vzniknou sloupce.
    ///
    /// [en] The table as tab-separated text (the first row has column names) – pasted
    /// into a spreadsheet or Word it becomes columns.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let head: Vec<&str> = self
            .inputs
            .iter()
            .chain(&self.outputs)
            .map(String::as_str)
            .collect();
        out.push_str(&head.join("\t"));
        out.push('\n');
        for (bits, values) in &self.rows {
            let row: Vec<String> = bits
                .iter()
                .map(|b| if *b { "1".to_owned() } else { "0".to_owned() })
                .chain(values.iter().map(ToString::to_string))
                .collect();
            out.push_str(&row.join("\t"));
            out.push('\n');
        }
        out
    }
}

/// Uložená tabulka s číslem revize obvodu, pro kterou platí (nebo důvod, proč ji nejde spočítat).
///
/// [en] Stored table with the circuit revision it belongs to (or the reason why it cannot be computed).
pub type TruthCache = Option<(u64, Result<TruthTable, T>)>;

/// Nejvyšší počet vstupů pro pravdivostní tabulku (2^8 = 256 řádků).
///
/// [en] Maximum number of inputs for the truth table (2^8 = 256 rows).
const MAX_TABLE_INPUTS: usize = 8;

/// Spočítá tabulku: pro každou kombinaci vstupů nová simulace až do ustálení.
///
/// [en] Computes the table: a new simulation until settled for each input combination.
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

/// Panel Pravdivostní tabulka (přepočítá se jen při změně obvodu).
///
/// [en] Truth table panel (recomputed only when the circuit changes).
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
        // [en] Current input state for highlighting the row.
        let (ins, _) = project.io_components();
        let current: Vec<bool> = ins
            .iter()
            .map(|id| project.sim.output(*id) == Some(LogicValue::One))
            .collect();
        let grid = egui::Grid::new("truth")
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
        // Pravé tlačítko na tabulce: zkopírovat ji jako text (do Wordu, sešitu, e-mailu).
        // [en] Right button on the table: copy it as text (to Word, a workbook, an e-mail).
        grid.response.container_context_menu(|ui| {
            if ui.button(tr(lang, T::CopyAsText)).clicked() {
                ui.ctx().copy_text(table.to_text());
                ui.close();
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
///
/// [en] Sample ROM content: decoder for a seven-segment display (segments g…a), digits 0–F.
const SEVEN_SEG: [u8; 16] = [
    0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F, 0x77, 0x7C, 0x39, 0x5E, 0x79, 0x71,
];

/// Panel Paměť – ukázka budoucího hex editoru.
///
/// [en] Memory panel – a preview of the future hex editor.
pub fn memory(ui: &mut Ui, lang: Lang) {
    padded(ui, |ui| {
        ui.label(RichText::new("ROM 16 × 8 – dekodér pro sedmisegmentovku").strong());
        ui.label(weak(&format!(
            "{} {}",
            tr(lang, T::ComingIn),
            crate::catalog::MEMORY_VERSION
        )));
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
    fn truth_table_as_text() {
        let p = Project::new(crate::examples::and_or());
        let t = compute_truth_table(&p).unwrap();
        assert_eq!(
            t.to_text(),
            "A\tB\tY1\tY2\n0\t0\t0\t0\n0\t1\t0\t1\n1\t0\t0\t1\n1\t1\t1\t1\n"
        );
    }

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
