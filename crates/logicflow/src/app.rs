//! Hlavní okno: horní lišta s nabídkami a pracovními plochami, oblasti, stavový řádek,
//! práce se soubory.
//!
//! [en] Main window: top bar with menus and workspaces, areas, status bar,
//! working with files.

use crate::commands::{self, Command, Scope};
use crate::examples;
use crate::filedialog::{self, FileDialog};
use crate::help::{self, HelpWindow};
use crate::i18n::{Lang, T, tr};
use crate::panels::TruthCache;
use crate::panes::{self, AreaBehavior, Pane, Workspace};
use crate::project::Project;
use crate::symbols::SymbolStyle;
use crate::theme::{self, colors};
use egui::{Key, RichText};
use egui_tiles::Tree;
use logicflow_core::Circuit;
use std::path::PathBuf;
use std::time::Duration;
// Instant z web-time: v nativním programu je to std::time::Instant, ve webové verzi
// (WebAssembly) čas prohlížeče – std::time::Instant::now() tam spadne.
// [en] Instant from web-time: in the native program it is std::time::Instant, in the web
// version (WebAssembly) the browser clock – std::time::Instant::now() panics there.
use web_time::Instant;

/// Akce, která čeká na rozhodnutí o neuložených změnách.
///
/// [en] Action waiting for a decision about unsaved changes.
#[derive(Clone, Copy, Debug)]
enum Pending {
    /// Nový prázdný obvod.
    ///
    /// [en] New empty circuit.
    New,
    /// Otevřít soubor.
    ///
    /// [en] Open a file.
    Open,
    /// Otevřít vestavěný příklad.
    ///
    /// [en] Open a built-in example.
    Example(usize),
    /// Ukončit aplikaci.
    ///
    /// [en] Quit the application.
    Quit,
}

/// Osobní nastavení uživatele (ukládá se mezi spuštěními, ne do souboru obvodu).
///
/// [en] The user's personal settings (kept between runs, not in the circuit file).
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Settings {
    /// Jazyk rozhraní.
    ///
    /// [en] Interface language.
    lang: Lang,
    /// Styl značek.
    ///
    /// [en] Symbol style.
    symbols: SymbolStyle,
    /// Povolit úpravy schématu i v režimu simulace (pro pokročilé, výchozí vypnuto).
    ///
    /// [en] Allow editing the schematic in simulation mode too (advanced, off by default).
    edit_while_simulating: bool,
}

/// Klíč nastavení v úložišti eframe.
///
/// [en] Settings key in the eframe storage.
const SETTINGS_KEY: &str = "settings";

impl Settings {
    /// Výchozí nastavení podle jazyka systému (např. „cs-CZ“, „en_US.UTF-8“):
    /// čeština (i pro slovenštinu), jinak angličtina; v americkém prostředí
    /// americké značky. Uživatel může obojí kdykoli změnit v nabídce Zobrazit.
    ///
    /// [en] Default settings according to the system language (e.g. “cs-CZ”, “en_US.UTF-8”):
    /// Czech (also for Slovak), otherwise English; American symbols in an American
    /// environment. The user can change both at any time in the View menu.
    fn for_locale(locale: Option<&str>) -> Self {
        let tag = locale
            .unwrap_or("")
            .split('.')
            .next()
            .unwrap_or("")
            .replace('_', "-")
            .to_ascii_lowercase();
        let mut parts = tag.split('-');
        let language = parts.next().unwrap_or("");
        let region = parts.find(|p| p.len() == 2).unwrap_or("");
        Self {
            lang: match language {
                "cs" | "sk" => Lang::Cs,
                "" => Lang::Cs,
                _ => Lang::En,
            },
            symbols: if region == "us" {
                SymbolStyle::Ansi
            } else {
                SymbolStyle::Iec
            },
            edit_while_simulating: false,
        }
    }
}

/// Webová stránka projektu.
///
/// [en] Project website.
pub const WEBSITE: &str = "https://www.logicflow-sim.cz";
/// Zdrojový kód a stažení.
///
/// [en] Source code and downloads.
pub const REPOSITORY: &str = "https://github.com/SJFidrmuc/LogicFlow";

/// Nejdelší úsek času simulace za jeden snímek (s).
///
/// [en] Longest stretch of simulation time per frame (s).
const MAX_FRAME_DT: f64 = 0.5;
/// Nejdelší prodleva mezi snímky při běžící simulaci (s).
///
/// [en] Longest pause between frames while the simulation runs (s).
const REPAINT_INTERVAL: f64 = 0.05;

/// Hlavní okno aplikace a jeho stav.
///
/// [en] Main application window and its state.
pub struct LogicFlowApp {
    /// Jazyk rozhraní.
    ///
    /// [en] Interface language.
    lang: Lang,
    /// Styl značek.
    ///
    /// [en] Symbol style.
    symbols: SymbolStyle,
    /// Povolit úpravy za běhu simulace (osobní nastavení).
    ///
    /// [en] Allow editing while simulating (personal setting).
    edit_while_simulating: bool,
    /// Vybraná pracovní plocha.
    ///
    /// [en] Selected workspace.
    workspace: usize,
    /// Rozložení oblastí každé pracovní plochy.
    ///
    /// [en] Area layout of each workspace.
    trees: Vec<Tree<Pane>>,
    /// Otevřený projekt.
    ///
    /// [en] The open project.
    project: Project,
    /// Mezivýsledek pravdivostní tabulky.
    ///
    /// [en] Cached truth table.
    truth_cache: TruthCache,
    /// Je otevřené okno O aplikaci?
    ///
    /// [en] Is the About window open?
    about_open: bool,
    /// Okno nápovědy (F1).
    ///
    /// [en] Help window (F1).
    help: HelpWindow,
    /// Otevřené okno pro soubory.
    ///
    /// [en] Open file dialog.
    file_dialog: Option<FileDialog>,
    /// Akce, která se provede po úspěšném uložení přes dialog.
    ///
    /// [en] Action carried out after a successful save through the dialog.
    after_save: Option<Pending>,
    /// Akce čekající na potvrzení (neuložené změny).
    ///
    /// [en] Action waiting for confirmation (unsaved changes).
    confirm: Option<Pending>,
    /// Chybová zpráva k zobrazení.
    ///
    /// [en] Error message to show.
    error: Option<String>,
    /// Poslední složka v okně pro soubory.
    ///
    /// [en] Last folder in the file dialog.
    last_dir: Option<PathBuf>,
    /// Naposledy nastavený titulek okna.
    ///
    /// [en] Window title set last time.
    last_title: String,
    /// Zavření okna je potvrzené.
    ///
    /// [en] Closing the window has been confirmed.
    allow_close: bool,
    /// Okamžik předchozího snímku – čas simulace se měří skutečnými hodinami,
    /// ne odhadem egui (`stable_dt` při překreslování „za chvíli“ vrací jen 1/60 s).
    ///
    /// [en] Moment of the previous frame – simulation time is measured by the real
    /// clock, not by the egui estimate (`stable_dt` returns only 1/60 s when
    /// repainting “in a moment”).
    last_frame: Option<Instant>,
}

impl LogicFlowApp {
    /// Vytvoří okno: načte nastavení (při prvním spuštění podle systému) a otevře soubor nebo první příklad.
    ///
    /// [en] Creates the window: loads the settings (from the system on the first run) and opens a file or the first example.
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> Self {
        theme::apply(&cc.egui_ctx);
        let settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, SETTINGS_KEY))
            // První spuštění: jazyk a značky podle nastavení systému.
            // [en] First run: language and symbols according to the system settings.
            .unwrap_or_else(|| Settings::for_locale(sys_locale::get_locale().as_deref()));
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
            edit_while_simulating: settings.edit_while_simulating,
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
            last_frame: None,
        }
    }

    /// Nahradí otevřený projekt a přizpůsobí pohledy.
    ///
    /// [en] Replaces the open project and adapts the views.
    fn replace_project(&mut self, project: Project) {
        self.project = project;
        self.truth_cache = None;
        for tree in &mut self.trees {
            panes::reset_views(tree);
        }
    }

    /// Spustí akci; při neuložených změnách se nejdřív zeptá.
    ///
    /// [en] Runs an action; asks first if there are unsaved changes.
    fn request(&mut self, action: Pending) {
        if self.project.modified {
            self.confirm = Some(action);
        } else {
            self.perform(action);
        }
    }

    /// Provede akci (nový, otevřít, příklad, konec) bez dalšího ptaní.
    ///
    /// [en] Carries out an action (new, open, example, quit) without asking again.
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
                // [en] Closing completes in the next frame (see `ui`).
            }
        }
    }

    /// Uloží projekt; bez cesty (nebo s `save_as`) otevře okno pro soubory.
    ///
    /// [en] Saves the project; without a path (or with `save_as`) opens the file dialog.
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

    /// Globální klávesové zkratky (Ctrl+N/O/S, Ctrl+Z/Y, mezerník, F1…) – seznam
    /// a zkratky jsou v `commands.rs`.
    ///
    /// [en] Global keyboard shortcuts (Ctrl+N/O/S, Ctrl+Z/Y, space, F1…) – the list
    /// and the shortcuts are in `commands.rs`.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        for cmd in commands::take_pressed(ctx, Scope::Global) {
            if cmd == Command::Help {
                self.help.open = !self.help.open;
            } else {
                self.run(cmd);
            }
        }
    }

    /// Provede příkaz z klávesnice nebo z nabídky. Příkazy editoru schématu, které
    /// nepotřebují pohled (otočit, smazat, vybrat, Tab), jdou spustit i z nabídky Úpravy.
    ///
    /// [en] Carries out a command from the keyboard or a menu. Schematic editor commands
    /// that do not need a view (rotate, delete, select, Tab) can also be run from the Edit menu.
    fn run(&mut self, cmd: Command) {
        if !cmd.available(&self.project) {
            return;
        }
        let n = self.trees.len();
        match cmd {
            Command::New => self.request(Pending::New),
            Command::Open => self.request(Pending::Open),
            Command::Save => self.save_file(false),
            Command::SaveAs => self.save_file(true),
            Command::Quit => self.request(Pending::Quit),
            Command::Undo => self.project.undo(),
            Command::Redo => self.project.redo(),
            Command::Rotate => self.project.rotate_selection(),
            Command::Delete => self.project.delete_selection(),
            Command::SelectAll => self.project.select_all(),
            Command::SelectNone => self.project.selection.clear(),
            Command::Cancel if self.project.is_simulating() => self.project.stop_simulation(),
            Command::Cancel => self.project.selection.clear(),
            Command::Subcircuit => self.project.toggle_subcircuit(),
            Command::RunPause => self.project.toggle_run(),
            Command::Start => self.project.start_simulation(),
            Command::Stop => self.project.stop_simulation(),
            Command::Step => self.project.step(),
            Command::ResetSim => self.project.reset_simulation(),
            Command::NextWorkspace => self.workspace = (self.workspace + 1) % n,
            Command::PrevWorkspace => self.workspace = (self.workspace + n - 1) % n,
            Command::Help => self.help.show_section(0),
            // Potřebují pohled schématu (polohu myši, kameru) – jen z klávesnice nad schématem.
            // [en] They need a schematic view (mouse position, camera) – only from the keyboard over the schematic.
            Command::Add | Command::Grab | Command::Duplicate | Command::FitView => {}
        }
    }

    /// Položky nabídky pro seznam příkazů; `None` znamená oddělovač.
    ///
    /// [en] Menu items for a list of commands; `None` means a separator.
    fn menu_commands(&mut self, ui: &mut egui::Ui, items: &[Option<Command>]) {
        for item in items {
            match *item {
                Some(cmd) => {
                    if commands::menu_item(ui, cmd, self.lang, &self.project) {
                        self.run(cmd);
                    }
                }
                None => {
                    ui.separator();
                }
            }
        }
    }

    /// Horní lišta: nabídky, pracovní plochy, ovládání simulace, jazyk.
    ///
    /// [en] Top bar: menus, workspaces, simulation controls, language.
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
                        self.menu_commands(ui, &[Some(Command::New), Some(Command::Open)]);
                        ui.menu_button(tr(lang, T::Examples), |ui| {
                            for (i, ex) in examples::ALL.iter().enumerate() {
                                if ui.button(ex.name).clicked() {
                                    self.request(Pending::Example(i));
                                }
                            }
                        });
                        self.menu_commands(
                            ui,
                            &[
                                None,
                                Some(Command::Save),
                                Some(Command::SaveAs),
                                None,
                                Some(Command::Quit),
                            ],
                        );
                    });
                    ui.menu_button(tr(lang, T::MenuEdit), |ui| {
                        self.menu_commands(
                            ui,
                            &[
                                Some(Command::Undo),
                                Some(Command::Redo),
                                None,
                                Some(Command::SelectAll),
                                Some(Command::SelectNone),
                                None,
                                Some(Command::Rotate),
                                Some(Command::Delete),
                                None,
                                Some(Command::Subcircuit),
                            ],
                        );
                    });
                    ui.menu_button(tr(lang, T::MenuView), |ui| {
                        if ui.button(tr(lang, T::ResetLayout)).clicked() {
                            let ws = Workspace::ALL[self.workspace];
                            self.trees[self.workspace] = ws.default_tree();
                        }
                        self.menu_commands(
                            ui,
                            &[
                                Some(Command::NextWorkspace),
                                Some(Command::PrevWorkspace),
                                None,
                            ],
                        );
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
                        // V režimu úprav: Spustit (F5); v simulaci: Pozastavit / Pokračovat
                        // (mezerník) a Zastavit (Shift+F5, Esc).
                        // [en] In edit mode: Run (F5); in simulation: Pause / Resume (space)
                        // and Stop (Shift+F5, Esc).
                        let first = if self.project.is_simulating() {
                            Command::RunPause
                        } else {
                            Command::Start
                        };
                        self.menu_commands(
                            ui,
                            &[
                                Some(first),
                                Some(Command::Step),
                                Some(Command::Stop),
                                None,
                                Some(Command::ResetSim),
                                None,
                            ],
                        );
                        ui.checkbox(
                            &mut self.edit_while_simulating,
                            tr(lang, T::EditWhileSimulating),
                        );
                    });
                    ui.menu_button(tr(lang, T::MenuHelp), |ui| {
                        self.menu_commands(ui, &[Some(Command::Help)]);
                        if ui.button(tr(lang, T::Shortcuts)).clicked() {
                            self.help.show_section(help::SECTION_SHORTCUTS);
                        }
                        if ui.button(tr(lang, T::WindowsWarning)).clicked() {
                            self.help.show_section(help::SECTION_WINDOWS);
                        }
                        ui.separator();
                        if ui
                            .button(tr(lang, T::Website))
                            .on_hover_text(WEBSITE)
                            .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(WEBSITE));
                        }
                        if ui.button(tr(lang, T::About)).clicked() {
                            self.about_open = true;
                        }
                    });

                    ui.add_space(16.0);
                    // Pracovní plochy jako záložky, jako v Blenderu.
                    // [en] Workspaces as tabs, as in Blender.
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

    /// Stavový řádek: stav simulace, nápověda ke zkratkám, soubor, verze.
    ///
    /// [en] Status bar: simulation state, shortcut hints, file, version.
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
                    let (text, color) = match (self.project.is_simulating(), self.project.running) {
                        (false, _) => (tr(lang, T::EditMode), colors::TEXT),
                        (true, true) => (tr(lang, T::Running), colors::SIMULATION),
                        (true, false) => (
                            tr(lang, T::Paused),
                            egui::Color32::from_rgb(0xff, 0xa0, 0x30),
                        ),
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

    /// Okna a dialogy: soubory, potvrzení neuložených změn, chyby, O aplikaci, nápověda.
    ///
    /// [en] Windows and dialogs: files, confirming unsaved changes, errors, About, help.
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
                ui.add_space(6.0);
                egui::Grid::new("about_links")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(tr(lang, T::Website));
                        ui.hyperlink_to("www.logicflow-sim.cz", WEBSITE);
                        ui.end_row();
                        ui.label(tr(lang, T::SourceCode));
                        ui.hyperlink_to("github.com/SJFidrmuc/LogicFlow", REPOSITORY);
                        ui.end_row();
                    });
            });
        self.about_open = open;
    }

    /// Nastaví titulek okna podle souboru a neuložených změn.
    ///
    /// [en] Sets the window title according to the file and unsaved changes.
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

/// Projekt z vestavěného příkladu.
///
/// [en] Project from a built-in example.
fn example_project(i: usize) -> Project {
    let ex = &examples::ALL[i.min(examples::ALL.len() - 1)];
    let mut p = Project::with_subcircuits((ex.build)(), (ex.subcircuits)());
    p.title = Some(ex.name.to_owned());
    p
}

impl eframe::App for LogicFlowApp {
    /// Uloží nastavení (jazyk, značky) do úložiště eframe.
    ///
    /// [en] Saves the settings (language, symbols) to the eframe storage.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let settings = Settings {
            lang: self.lang,
            symbols: self.symbols,
            edit_while_simulating: self.edit_while_simulating,
        };
        eframe::set_value(storage, SETTINGS_KEY, &settings);
    }

    /// Tab slouží ke vstupu do podobvodu a zpět (jako v Blenderu). Mimo textová pole
    /// ho proto egui nedostane – jinak by jím přesouvalo fokus mezi tlačítky.
    ///
    /// [en] Tab is used to enter a subcircuit and go back (as in Blender). Outside text
    /// fields egui therefore does not get it – otherwise it would move the focus
    /// between buttons.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Nevyužitý stisk z minulého snímku (myš nebyla nad schématem) se zahodí.
        // [en] An unused press from the previous frame (the mouse was not over the schematic) is dropped.
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
    ///
    /// [en] egui memory (expanded groups etc.) is not saved – only our settings and the window.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    /// Jeden snímek: čas simulace, zkratky, lišty, oblasti a dialogy.
    ///
    /// [en] One frame: simulation time, shortcuts, bars, areas and dialogs.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::symbols::set_style(ui.ctx(), self.symbols);
        let ctx = ui.ctx().clone();

        // Zavírání okna s neuloženými změnami.
        // [en] Closing the window with unsaved changes.
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

        self.project.edit_while_simulating = self.edit_while_simulating;
        let no_dialog =
            self.file_dialog.is_none() && self.confirm.is_none() && self.error.is_none();
        if no_dialog {
            self.handle_shortcuts(&ctx);
        }

        let now = Instant::now();
        let dt = self
            .last_frame
            .map_or(0.0, |t| now.duration_since(t).as_secs_f64())
            // Po uspání nebo minimalizaci okna simulace nepřeskakuje dlouhý úsek.
            // [en] After sleep or minimising the window, the simulation does not skip a long stretch.
            .min(MAX_FRAME_DT);
        self.last_frame = Some(now);
        self.project.update(dt);
        if self.project.running {
            // Časový diagram se posouvá, proto překreslujeme průběžně – a nejpozději
            // v okamžiku příští hrany hodin, aby LED blikaly přesně v rytmu.
            // [en] The timing diagram scrolls, so we repaint continuously – and at the latest
            // at the moment of the next clock edge, so that LEDs blink exactly in rhythm.
            let wait = self
                .project
                .time_to_next_clock_edge()
                .map_or(REPAINT_INTERVAL, |t| t.clamp(0.0, REPAINT_INTERVAL));
            ctx.request_repaint_after(Duration::from_secs_f64(wait));
        } else if !self.project.is_simulating() && !self.project.sim.is_stable() {
            // Režim úprav: obvod se ještě ustaluje (nebo kmitá).
            // [en] Edit mode: the circuit is still settling (or oscillating).
            ctx.request_repaint();
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
        // Esc, který nezpracoval žádný editor schématu (myš je jinde): v simulaci
        // zastaví simulaci.
        // [en] Esc not handled by any schematic editor (the mouse is elsewhere): in
        // simulation it stops the simulation.
        if no_dialog
            && self.project.is_simulating()
            && !ctx.egui_wants_keyboard_input()
            && commands::take(&ctx, Command::Cancel)
        {
            self.project.stop_simulation();
        }
        self.dialogs(&ctx);
        self.update_title(&ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_system_locale() {
        let s = Settings::for_locale(Some("cs-CZ"));
        assert_eq!((s.lang, s.symbols), (Lang::Cs, SymbolStyle::Iec));
        let s = Settings::for_locale(Some("sk_SK.UTF-8"));
        assert_eq!(s.lang, Lang::Cs);
        let s = Settings::for_locale(Some("en-US"));
        assert_eq!((s.lang, s.symbols), (Lang::En, SymbolStyle::Ansi));
        let s = Settings::for_locale(Some("en_GB.UTF-8"));
        assert_eq!((s.lang, s.symbols), (Lang::En, SymbolStyle::Iec));
        let s = Settings::for_locale(Some("de-DE"));
        assert_eq!((s.lang, s.symbols), (Lang::En, SymbolStyle::Iec));
        let s = Settings::for_locale(Some("zh-Hans-US"));
        assert_eq!(s.symbols, SymbolStyle::Ansi);
        // Neznámé prostředí: čeština a evropské značky.
        // [en] Unknown environment: Czech and European symbols.
        let s = Settings::for_locale(None);
        assert_eq!((s.lang, s.symbols), (Lang::Cs, SymbolStyle::Iec));
    }
}
