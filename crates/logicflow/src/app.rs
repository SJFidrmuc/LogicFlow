//! Hlavní okno: horní lišta s nabídkami a pracovními plochami, oblasti, stavový řádek.

use crate::demo::DemoCircuit;
use crate::i18n::{Lang, T, tr};
use crate::panes::{AreaBehavior, Pane, Workspace};
use crate::theme::{self, colors};
use egui::{Key, KeyboardShortcut, Modifiers, RichText};
use egui_tiles::Tree;

pub struct LogicFlowApp {
    lang: Lang,
    workspace: usize,
    trees: Vec<Tree<Pane>>,
    circuit: DemoCircuit,
    about_open: bool,
}

impl LogicFlowApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        Self {
            lang: Lang::Cs,
            workspace: 0,
            trees: Workspace::ALL.iter().map(|w| w.default_tree()).collect(),
            circuit: DemoCircuit::default(),
            about_open: false,
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (space, next_ws, prev_ws, quit) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::Space),
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::PageDown)),
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::PageUp)),
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Q)),
            )
        });
        // Mezerník nesmí přepínat simulaci, když uživatel píše do pole.
        if space && !ctx.egui_wants_keyboard_input() {
            self.circuit.running = !self.circuit.running;
        }
        let n = self.trees.len();
        if next_ws {
            self.workspace = (self.workspace + 1) % n;
        }
        if prev_ws {
            self.workspace = (self.workspace + n - 1) % n;
        }
        if quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        egui::Panel::top("top_bar")
            .frame(
                egui::Frame::new()
                    .fill(colors::HEADER)
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("LogicFlow")
                            .strong()
                            .color(egui::Color32::from_rgb(0x7e, 0xe0, 0x7e)),
                    );
                    ui.add_space(8.0);
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.menu_button(tr(lang, T::MenuFile), |ui| {
                            ui.add_enabled(
                                false,
                                egui::Button::new(tr(lang, T::New)).shortcut_text("Ctrl+N"),
                            );
                            ui.add_enabled(
                                false,
                                egui::Button::new(tr(lang, T::Open)).shortcut_text("Ctrl+O"),
                            );
                            ui.add_enabled(
                                false,
                                egui::Button::new(tr(lang, T::Save)).shortcut_text("Ctrl+S"),
                            );
                            ui.separator();
                            if ui
                                .add(egui::Button::new(tr(lang, T::Quit)).shortcut_text("Ctrl+Q"))
                                .clicked()
                            {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                        ui.menu_button(tr(lang, T::MenuEdit), |ui| {
                            ui.add_enabled(
                                false,
                                egui::Button::new(tr(lang, T::Undo)).shortcut_text("Ctrl+Z"),
                            );
                            ui.add_enabled(
                                false,
                                egui::Button::new(tr(lang, T::Redo)).shortcut_text("Ctrl+Shift+Z"),
                            );
                        });
                        ui.menu_button(tr(lang, T::MenuView), |ui| {
                            if ui.button(tr(lang, T::ResetLayout)).clicked() {
                                let ws = Workspace::ALL[self.workspace];
                                self.trees[self.workspace] = ws.default_tree();
                            }
                            ui.menu_button(tr(lang, T::Language), |ui| {
                                ui.radio_value(&mut self.lang, Lang::Cs, "Čeština");
                                ui.radio_value(&mut self.lang, Lang::En, "English");
                            });
                        });
                        ui.menu_button(tr(lang, T::MenuSimulation), |ui| {
                            let label = if self.circuit.running {
                                T::Pause
                            } else {
                                T::Run
                            };
                            if ui
                                .add(egui::Button::new(tr(lang, label)).shortcut_text("Mezerník"))
                                .clicked()
                            {
                                self.circuit.running = !self.circuit.running;
                            }
                            if ui
                                .add_enabled(
                                    !self.circuit.running,
                                    egui::Button::new(tr(lang, T::Step)),
                                )
                                .clicked()
                            {
                                self.circuit.running = true;
                                self.circuit.advance(1.0);
                                self.circuit.running = false;
                            }
                        });
                        ui.menu_button(tr(lang, T::MenuHelp), |ui| {
                            if ui.button(tr(lang, T::About)).clicked() {
                                self.about_open = true;
                            }
                        });

                        ui.add_space(16.0);
                        // Pracovní plochy jako záložky, jako v Blenderu.
                        for (i, ws) in Workspace::ALL.iter().enumerate() {
                            if ui
                                .add(egui::Button::selectable(
                                    self.workspace == i,
                                    ws.title(lang),
                                ))
                                .clicked()
                            {
                                self.workspace = i;
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            for (l, code) in [(Lang::En, "EN"), (Lang::Cs, "CS")] {
                                if ui
                                    .add(egui::Button::selectable(self.lang == l, code))
                                    .clicked()
                                {
                                    self.lang = l;
                                }
                            }
                        });
                    });
                });
            });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        egui::Panel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(colors::HEADER)
                    .inner_margin(egui::Margin::symmetric(8, 3)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (text, color) = if self.circuit.running {
                        (tr(lang, T::Running), colors::TEXT)
                    } else {
                        (
                            tr(lang, T::Paused),
                            egui::Color32::from_rgb(0xff, 0xa0, 0x30),
                        )
                    };
                    ui.label(RichText::new(text).color(color).small());
                    ui.label(
                        RichText::new(format!("t = {:.1} s", self.circuit.time))
                            .monospace()
                            .small()
                            .color(colors::TEXT_WEAK),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("LogicFlow {}", env!("CARGO_PKG_VERSION")))
                                .small()
                                .color(colors::TEXT_WEAK),
                        );
                    });
                });
            });
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        let lang = self.lang;
        let mut open = self.about_open;
        egui::Window::new(tr(lang, T::About))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(format!("LogicFlow {}", env!("CARGO_PKG_VERSION")));
                ui.label(tr(lang, T::AboutText));
                ui.add_space(6.0);
                ui.label(format!(
                    "© 2026 Stanislav Jan Fidrmuc · {}",
                    tr(lang, T::License)
                ));
                ui.hyperlink("https://github.com/SJFidrmuc/LogicFlow");
            });
        self.about_open = open;
    }
}

impl eframe::App for LogicFlowApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);

        let dt = ctx.input(|i| i.stable_dt).min(0.1) as f64;
        self.circuit.advance(dt);
        if self.circuit.running {
            // Časový diagram se posouvá, proto překreslujeme průběžně.
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        self.top_bar(ui);
        self.status_bar(ui);
        egui::CentralPanel::no_frame()
            .frame(egui::Frame::new().fill(colors::GAP).inner_margin(3))
            .show(ui, |ui| {
                let mut behavior = AreaBehavior {
                    circuit: &mut self.circuit,
                    lang: self.lang,
                };
                self.trees[self.workspace].ui(&mut behavior, ui);
            });
        self.about_window(&ctx);
    }
}
