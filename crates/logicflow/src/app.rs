//! Hlavní okno: horní lišta s nabídkami a pracovními plochami, oblasti, stavový řádek,
//! práce se soubory.

use crate::examples;
use crate::filedialog::{self, FileDialog};
use crate::help::{self, HelpWindow};
use crate::i18n::{Lang, T, tr};
use crate::panels::TruthCache;
use crate::panes::{self, AreaBehavior, Pane, Workspace};
use crate::project::Project;
use crate::symbols::SymbolStyle;
use crate::theme::{self, colors};
use egui::{Key, KeyboardShortcut, Modifiers, RichText};
use egui_tiles::Tree;
use logicflow_core::Circuit;
use std::path::PathBuf;

/// Akce, která čeká na rozhodnutí o neuložených změnách.
#[derive(Clone, Copy, Debug)]
enum Pending {
    New,
    Open,
    Example(usize),
    Quit,
}

/// Osobní nastavení uživatele (ukládá se mezi spuštěními, ne do souboru obvodu).
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Settings {
    lang: Lang,
    symbols: SymbolStyle,
}

const SETTINGS_KEY: &str = "settings";

pub struct LogicFlowApp {
    lang: Lang,
    symbols: SymbolStyle,
    workspace: usize,
    trees: Vec<Tree<Pane>>,
    project: Project,
    truth_cache: TruthCache,
    about_open: bool,
    help: HelpWindow,
    file_dialog: Option<FileDialog>,
    /// Akce, která se provede po úspěšném uložení přes dialog.
    after_save: Option<Pending>,
    confirm: Option<Pending>,
    error: Option<String>,
    last_dir: Option<PathBuf>,
    last_title: String,
    allow_close: bool,
}

impl LogicFlowApp {
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> Self {
        theme::apply(&cc.egui_ctx);
        let settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, SETTINGS_KEY))
            .unwrap_or_default();
        let mut error = None;
        let project = match open.as_deref().map(Project::load) {
            Some(Ok(p)) => p,
            Some(Err(e)) => {
                error = Some(e);
                example_project(0)
            }
            None => example_project(0),
        };
        Self {
            lang: settings.lang,
            symbols: settings.symbols,
            workspace: 0,
            trees: Workspace::ALL.iter().map(|w| w.default_tree()).collect(),
            last_dir: project
                .path
                .as_ref()
                .and_then(|p| p.parent().map(PathBuf::from)),
            project,
            truth_cache: None,
            about_open: false,
            help: HelpWindow::new(),
            file_dialog: None,
            after_save: None,
            confirm: None,
            error,
            last_title: String::new(),
            allow_close: false,
        }
    }

    fn replace_project(&mut self, project: Project) {
        self.project = project;
        self.truth_cache = None;
        for tree in &mut self.trees {
            panes::reset_views(tree);
        }
    }

    /// Spustí akci; při neuložených změnách se nejdřív zeptá.
    fn request(&mut self, action: Pending) {
        if self.project.modified {
            self.confirm = Some(action);
        } else {
            self.perform(action);
        }
    }

    fn perform(&mut self, action: Pending) {
        match action {
            Pending::New => {
                let mut p = Project::new(Circuit::default());
                p.title = None;
                self.replace_project(p);
            }
            Pending::Open => {
                self.file_dialog = Some(FileDialog::new(
                    filedialog::Mode::Open,
                    self.last_dir.clone(),
                    "",
                ));
            }
            Pending::Example(i) => self.replace_project(example_project(i)),
            Pending::Quit => {
                self.allow_close = true;
                // Zavření se dokončí v příštím snímku (viz `ui`).
            }
        }
    }

    fn save_file(&mut self, save_as: bool) {
        match self.project.path.clone() {
            Some(path) if !save_as => {
                if let Err(e) = self.project.save(&path) {
                    self.error = Some(e);
                }
            }
            _ => {
                let name = format!(
                    "{}.logicflow",
                    self.project.display_name().trim_end_matches(".logicflow")
                );
                self.file_dialog = Some(FileDialog::new(
                    filedialog::Mode::Save,
                    self.last_dir.clone(),
                    &name,
                ));
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let typing = ctx.egui_wants_keyboard_input();
        let sc = |m, k| KeyboardShortcut::new(m, k);
        let ctrl_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        let (save_as, save, open, new, redo, undo, redo_y, next_ws, prev_ws, quit) =
            ctx.input_mut(|i| {
                (
                    i.consume_shortcut(&sc(ctrl_shift, Key::S)),
                    i.consume_shortcut(&sc(Modifiers::COMMAND, Key::S)),
                    i.consume_shortcut(&sc(Modifiers::COMMAND, Key::O)),
                    i.consume_shortcut(&sc(Modifiers::COMMAND, Key::N)),
                    !typing && i.consume_shortcut(&sc(ctrl_shift, Key::Z)),
                    !typing && i.consume_shortcut(&sc(Modifiers::COMMAND, Key::Z)),
                    !typing && i.consume_shortcut(&sc(Modifiers::COMMAND, Key::Y)),
                    i.consume_shortcut(&sc(Modifiers::COMMAND, Key::PageDown)),
                    i.consume_shortcut(&sc(Modifiers::COMMAND, Key::PageUp)),
                    i.consume_shortcut(&sc(Modifiers::COMMAND, Key::Q)),
                )
            });
        let (space, step) = if typing {
            (false, false)
        } else {
            ctx.input_mut(|i| {
                (
                    i.consume_key(Modifiers::NONE, Key::Space),
                    i.consume_key(Modifiers::NONE, Key::ArrowRight),
                )
            })
        };
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F1)) {
            self.help.open = !self.help.open;
        }
        if save_as {
            self.save_file(true);
        } else if save {
            self.save_file(false);
        }
        if open {
            self.request(Pending::Open);
        }
        if new {
            self.request(Pending::New);
        }
        if redo || redo_y {
            self.project.redo();
        } else if undo {
            self.project.undo();
        }
        if space {
            self.project.running = !self.project.running;
        }
        if step && !self.project.running {
            self.project.step();
        }
        let n = self.trees.len();
        if next_ws {
            self.workspace = (self.workspace + 1) % n;
        }
        if prev_ws {
            self.workspace = (self.workspace + n - 1) % n;
        }
        if quit {
            self.request(Pending::Quit);
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
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.label(
                        RichText::new("LogicFlow")
                            .strong()
                            .color(egui::Color32::from_rgb(0x7e, 0xe0, 0x7e)),
                    );
                    ui.add_space(8.0);
                    ui.menu_button(tr(lang, T::MenuFile), |ui| {
                        if ui
                            .add(egui::Button::new(tr(lang, T::New)).shortcut_text("Ctrl+N"))
                            .clicked()
                        {
                            self.request(Pending::New);
                        }
                        if ui
                            .add(egui::Button::new(tr(lang, T::Open)).shortcut_text("Ctrl+O"))
                            .clicked()
                        {
                            self.request(Pending::Open);
                        }
                        ui.menu_button(tr(lang, T::Examples), |ui| {
                            for (i, ex) in examples::ALL.iter().enumerate() {
                                if ui.button(ex.name).clicked() {
                                    self.request(Pending::Example(i));
                                }
                            }
                        });
                        ui.separator();
                        if ui
                            .add(egui::Button::new(tr(lang, T::Save)).shortcut_text("Ctrl+S"))
                            .clicked()
                        {
                            self.save_file(false);
                        }
                        if ui
                            .add(
                                egui::Button::new(tr(lang, T::SaveAs))
                                    .shortcut_text("Ctrl+Shift+S"),
                            )
                            .clicked()
                        {
                            self.save_file(true);
                        }
                        ui.separator();
                        if ui
                            .add(egui::Button::new(tr(lang, T::Quit)).shortcut_text("Ctrl+Q"))
                            .clicked()
                        {
                            self.request(Pending::Quit);
                        }
                    });
                    ui.menu_button(tr(lang, T::MenuEdit), |ui| {
                        let undo = egui::Button::new(tr(lang, T::Undo)).shortcut_text("Ctrl+Z");
                        if ui.add_enabled(self.project.can_undo(), undo).clicked() {
                            self.project.undo();
                        }
                        let redo =
                            egui::Button::new(tr(lang, T::Redo)).shortcut_text("Ctrl+Shift+Z");
                        if ui.add_enabled(self.project.can_redo(), redo).clicked() {
                            self.project.redo();
                        }
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
                        ui.menu_button(tr(lang, T::Symbols), |ui| {
                            ui.radio_value(
                                &mut self.symbols,
                                SymbolStyle::Iec,
                                tr(lang, T::SymbolsIec),
                            );
                            ui.radio_value(
                                &mut self.symbols,
                                SymbolStyle::Ansi,
                                tr(lang, T::SymbolsAnsi),
                            );
                        });
                    });
                    ui.menu_button(tr(lang, T::MenuSimulation), |ui| {
                        let label = if self.project.running {
                            T::Pause
                        } else {
                            T::Run
                        };
                        if ui
                            .add(egui::Button::new(tr(lang, label)).shortcut_text(
                                if lang == Lang::Cs {
                                    "mezerník"
                                } else {
                                    "Space"
                                },
                            ))
                            .clicked()
                        {
                            self.project.running = !self.project.running;
                        }
                        let arrow = if lang == Lang::Cs {
                            "šipka vpravo"
                        } else {
                            "Right arrow"
                        };
                        let step = egui::Button::new(tr(lang, T::Step)).shortcut_text(arrow);
                        if ui.add_enabled(!self.project.running, step).clicked() {
                            self.project.step();
                        }
                        ui.separator();
                        let reset =
                            egui::Button::new(tr(lang, T::ResetSim)).shortcut_text("Ctrl+R");
                        if ui.add(reset).clicked() {
                            self.project.reset_simulation();
                        }
                    });
                    ui.menu_button(tr(lang, T::MenuHelp), |ui| {
                        if ui
                            .add(egui::Button::new(tr(lang, T::Manual)).shortcut_text("F1"))
                            .clicked()
                        {
                            self.help.show_section(0);
                        }
                        if ui.button(tr(lang, T::Shortcuts)).clicked() {
                            self.help.show_section(help::SECTION_SHORTCUTS);
                        }
                        if ui.button(tr(lang, T::WindowsWarning)).clicked() {
                            self.help.show_section(help::SECTION_WINDOWS);
                        }
                        ui.separator();
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
                    let (text, color) = if self.project.running {
                        (tr(lang, T::Running), colors::TEXT)
                    } else {
                        (
                            tr(lang, T::Paused),
                            egui::Color32::from_rgb(0xff, 0xa0, 0x30),
                        )
                    };
                    ui.label(RichText::new(text).color(color).small());
                    ui.label(
                        RichText::new(format!("{} {}", tr(lang, T::Tick), self.project.sim.time()))
                            .monospace()
                            .small()
                            .color(colors::TEXT_WEAK),
                    );
                    if self.project.sim.is_oscillating() {
                        ui.label(
                            RichText::new(tr(lang, T::Oscillating))
                                .small()
                                .color(egui::Color32::from_rgb(0xff, 0x70, 0x60)),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("LogicFlow {}", env!("CARGO_PKG_VERSION")))
                                .small()
                                .color(colors::TEXT_WEAK),
                        );
                        ui.separator();
                        let mut name = self.project.display_name();
                        if self.project.modified {
                            name = format!("{name} ({})", tr(lang, T::Unsaved));
                        }
                        ui.label(RichText::new(name).small().color(colors::TEXT_WEAK));
                    });
                });
            });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        let lang = self.lang;
        self.help.ui(ctx, lang);

        if let Some(dialog) = &mut self.file_dialog {
            match dialog.show(ctx, lang) {
                filedialog::Outcome::None => {}
                filedialog::Outcome::Cancel => {
                    self.file_dialog = None;
                    self.after_save = None;
                }
                filedialog::Outcome::Chosen(path) => {
                    self.last_dir = Some(dialog.dir().to_path_buf());
                    let mode = dialog.mode;
                    self.file_dialog = None;
                    match mode {
                        filedialog::Mode::Open => match Project::load(&path) {
                            Ok(p) => self.replace_project(p),
                            Err(e) => self.error = Some(format!("{}\n{e}", path.display())),
                        },
                        filedialog::Mode::Save => match self.project.save(&path) {
                            Ok(()) => {
                                if let Some(next) = self.after_save.take() {
                                    self.perform(next);
                                }
                            }
                            Err(e) => self.error = Some(format!("{}\n{e}", path.display())),
                        },
                    }
                }
            }
        }

        if let Some(action) = self.confirm {
            let mut choice = None;
            egui::Modal::new(egui::Id::unique("logicflow-unsaved")).show(ctx, |ui| {
                ui.set_width(360.0);
                ui.heading(tr(lang, T::UnsavedTitle));
                ui.label(tr(lang, T::UnsavedText));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(tr(lang, T::Save)).clicked() {
                        choice = Some(0);
                    }
                    if ui.button(tr(lang, T::Discard)).clicked() {
                        choice = Some(1);
                    }
                    if ui.button(tr(lang, T::Cancel)).clicked() {
                        choice = Some(2);
                    }
                });
            });
            match choice {
                Some(0) => {
                    self.confirm = None;
                    if self.project.path.is_some() {
                        self.save_file(false);
                        if !self.project.modified {
                            self.perform(action);
                        }
                    } else {
                        self.after_save = Some(action);
                        self.save_file(true);
                    }
                }
                Some(1) => {
                    self.confirm = None;
                    self.perform(action);
                }
                Some(_) => self.confirm = None,
                None => {}
            }
        }

        if let Some(err) = self.error.clone() {
            egui::Modal::new(egui::Id::unique("logicflow-error")).show(ctx, |ui| {
                ui.set_width(400.0);
                ui.heading(tr(lang, T::ErrorTitle));
                ui.label(err);
                ui.add_space(8.0);
                if ui.button(tr(lang, T::Close)).clicked() {
                    self.error = None;
                }
            });
        }

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

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = format!(
            "LogicFlow – {}{}",
            self.project.display_name(),
            if self.project.modified { " *" } else { "" }
        );
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }
}

fn example_project(i: usize) -> Project {
    let ex = &examples::ALL[i.min(examples::ALL.len() - 1)];
    let mut p = Project::with_subcircuits((ex.build)(), (ex.subcircuits)());
    p.title = Some(ex.name.to_owned());
    p
}

impl eframe::App for LogicFlowApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let settings = Settings {
            lang: self.lang,
            symbols: self.symbols,
        };
        eframe::set_value(storage, SETTINGS_KEY, &settings);
    }

    /// Tab slouží ke vstupu do podobvodu a zpět (jako v Blenderu). Mimo textová pole
    /// ho proto egui nedostane – jinak by jím přesouvalo fokus mezi tlačítky.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Nevyužitý stisk z minulého snímku (myš nebyla nad schématem) se zahodí.
        crate::schematic::take_tab(ctx);
        if ctx.text_edit_focused() {
            return;
        }
        let mut pressed = false;
        raw_input.events.retain(|e| match e {
            egui::Event::Key {
                key: Key::Tab,
                modifiers,
                pressed: p,
                ..
            } if !modifiers.any() => {
                pressed |= *p;
                false
            }
            _ => true,
        });
        if pressed {
            crate::schematic::request_tab(ctx);
        }
    }

    /// Paměť egui (rozbalené skupiny apod.) se neukládá – jen naše nastavení a okno.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::symbols::set_style(ui.ctx(), self.symbols);
        let ctx = ui.ctx().clone();

        // Zavírání okna s neuloženými změnami.
        if ctx.input(|i| i.viewport().close_requested())
            && self.project.modified
            && !self.allow_close
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm = Some(Pending::Quit);
        }
        if self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if self.file_dialog.is_none() && self.confirm.is_none() && self.error.is_none() {
            self.handle_shortcuts(&ctx);
        }

        let dt = ctx.input(|i| i.stable_dt).min(0.1) as f64;
        self.project.update(dt);
        if self.project.running {
            // Časový diagram se posouvá, proto překreslujeme průběžně.
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        self.top_bar(ui);
        self.status_bar(ui);
        egui::CentralPanel::no_frame()
            .frame(egui::Frame::new().fill(colors::GAP).inner_margin(3))
            .show(ui, |ui| {
                let tree = &mut self.trees[self.workspace];
                let mut behavior = AreaBehavior {
                    project: &mut self.project,
                    truth_cache: &mut self.truth_cache,
                    lang: self.lang,
                    requests: Vec::new(),
                    can_close: panes::count_panes(tree) > 1,
                };
                tree.ui(&mut behavior, ui);
                let requests = std::mem::take(&mut behavior.requests);
                panes::apply_requests(tree, requests);
            });
        self.dialogs(&ctx);
        self.update_title(&ctx);
    }
}
