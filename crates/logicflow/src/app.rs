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
use crate::settings::{self, Settings, SharedFolder};
use crate::symbols::SymbolStyle;
use crate::theme::{self, colors};
use egui::{Key, RichText};
use egui_tiles::Tree;
use logicflow_core::CircuitInfo;
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

/// Nastavení ze starších verzí (do 0.3) uložené v úložišti eframe. Při prvním
/// spuštění bez souboru s nastavením se převezme, aby uživatel o volby nepřišel.
///
/// [en] Settings from older versions (up to 0.3) stored in the eframe storage. On the
/// first run without a settings file they are taken over so the user keeps them.
#[derive(Clone, Copy, Debug, Default, serde::Deserialize)]
#[serde(default)]
struct LegacySettings {
    /// Jazyk rozhraní.
    ///
    /// [en] Interface language.
    lang: Lang,
    /// Styl značek.
    ///
    /// [en] Symbol style.
    symbols: SymbolStyle,
    /// Povolit úpravy za běhu simulace.
    ///
    /// [en] Allow editing while simulating.
    edit_while_simulating: bool,
}

/// Klíč starého nastavení v úložišti eframe.
///
/// [en] Key of the old settings in the eframe storage.
const SETTINGS_KEY: &str = "settings";

/// K čemu se vybírá složka v okně pro soubory.
///
/// [en] What a folder is being chosen for in the file dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FolderTarget {
    /// Složka osobních knihoven.
    ///
    /// [en] Personal libraries folder.
    PersonalLibraries,
    /// Nová sdílená složka knihoven.
    ///
    /// [en] A new shared library folder.
    SharedLibrary,
}

/// Licence nabízené v nastavení (vlastní jde napsat do pole).
///
/// [en] Licences offered in the settings (a custom one can be typed into the field).
const LICENSE_PRESETS: [&str; 4] = ["CC BY 4.0", "CC BY-SA 4.0", "CC BY-NC 4.0", "CC0 1.0"];

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
    /// Rychlost simulace (×0,1 … ×10); po spuštění programu vždy ×1.
    ///
    /// [en] Simulation speed (×0.1 … ×10); always ×1 after the program starts.
    speed: f64,
    /// Vybraná pracovní plocha.
    ///
    /// [en] Selected workspace.
    workspace: usize,
    /// Pracovní plocha v minulém snímku (ke zjištění přepnutí).
    ///
    /// [en] Workspace in the previous frame (to detect a switch).
    shown_workspace: usize,
    /// Plocha, na kterou se vrátí zastavení simulace na ploše Simulace.
    ///
    /// [en] Workspace that stopping the simulation on the Simulation workspace returns to.
    return_workspace: usize,
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
    /// Knihovny prvků (vestavěná, osobní a sdílené).
    ///
    /// [en] Element libraries (built-in, personal and shared).
    libraries: crate::libraries::Libraries,
    /// Osobní nastavení (jazyk, značky a úpravy za běhu simulace jsou navíc ve vlastních
    /// polích okna; před uložením se sem přepíšou).
    ///
    /// [en] Personal settings (language, symbols and editing while simulating are also in
    /// the window's own fields; they are copied here before saving).
    settings: Settings,
    /// Cesta k souboru s nastavením (`None` = složku nejde zjistit, neukládá se).
    ///
    /// [en] Path to the settings file (`None` = the folder cannot be found, not saved).
    settings_path: Option<PathBuf>,
    /// Naposledy uložené nastavení (`None` = ještě neuloženo) – ukládá se jen po změně.
    ///
    /// [en] Settings saved last (`None` = not saved yet) – saved only after a change.
    saved_settings: Option<Settings>,
    /// Chyba při posledním ukládání nastavení.
    ///
    /// [en] Error when the settings were saved last.
    settings_error: Option<String>,
    /// Je otevřené okno Nastavení?
    ///
    /// [en] Is the Settings window open?
    settings_open: bool,
    /// Rozpracované informace v okně Vlastnosti obvodu (`None` = okno je zavřené).
    ///
    /// [en] Information being edited in the Circuit properties window (`None` = the window is closed).
    properties: Option<CircuitInfo>,
    /// Čeká obnovení výchozího nastavení na potvrzení?
    ///
    /// [en] Is restoring the default settings waiting for confirmation?
    settings_reset: bool,
    /// K čemu se právě vybírá složka.
    ///
    /// [en] What a folder is being chosen for at the moment.
    folder_target: Option<FolderTarget>,
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
        // Výchozí hodnoty podle systému; soubor s nastavením je přepíše.
        // [en] Defaults according to the system; the settings file overrides them.
        let defaults = Settings::for_locale(sys_locale::get_locale().as_deref());
        let settings_path = settings::folder().map(|d| d.join(settings::FILE_NAME));
        let mut error = None;
        let mut saved_settings = None;
        let settings = match settings_path
            .as_deref()
            .map(|p| settings::load(p, &defaults))
        {
            Some(settings::Loaded::Ok(s)) => {
                saved_settings = Some(s.clone());
                s
            }
            Some(settings::Loaded::Broken(e)) => {
                error = Some(format!("{}\n{e}", tr(defaults.lang, T::SettingsBroken)));
                defaults
            }
            // První spuštění: převezme se nastavení starší verze z úložiště eframe.
            // [en] First run: the settings of an older version are taken from the eframe storage.
            Some(settings::Loaded::Missing) | None => {
                let mut s = defaults;
                if let Some(old) = cc
                    .storage
                    .and_then(|st| eframe::get_value::<LegacySettings>(st, SETTINGS_KEY))
                {
                    s.lang = old.lang;
                    s.symbols = old.symbols;
                    s.edit_while_simulating = old.edit_while_simulating;
                }
                s
            }
        };
        let project = match open.as_deref().map(Project::load) {
            Some(Ok(p)) => p,
            Some(Err(e)) => {
                error = Some(error.map_or(e.clone(), |prev| format!("{prev}\n\n{e}")));
                example_project(0)
            }
            None => example_project(0),
        };
        Self {
            lang: settings.lang,
            symbols: settings.symbols,
            edit_while_simulating: settings.edit_while_simulating,
            speed: 1.0,
            workspace: 0,
            shown_workspace: 0,
            return_workspace: 0,
            trees: Workspace::ALL.iter().map(|w| w.default_tree()).collect(),
            last_dir: project
                .path
                .as_ref()
                .and_then(|p| p.parent().map(PathBuf::from)),
            project,
            truth_cache: None,
            libraries: crate::libraries::Libraries::load(&settings),
            settings_path,
            saved_settings,
            settings_error: None,
            settings_open: false,
            properties: None,
            settings_reset: false,
            folder_target: None,
            settings,
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
                let p = Project::new_empty(&self.settings.author, &self.settings.license);
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
                self.run(ctx, cmd);
            }
        }
    }

    /// Provede příkaz z klávesnice nebo z nabídky. Příkazy editoru schématu, které
    /// nepotřebují pohled (otočit, smazat, vybrat, Tab), jdou spustit i z nabídky Úpravy.
    ///
    /// [en] Carries out a command from the keyboard or a menu. Schematic editor commands
    /// that do not need a view (rotate, delete, select, Tab) can also be run from the Edit menu.
    fn run(&mut self, ctx: &egui::Context, cmd: Command) {
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
            Command::StepEdge => self.project.step_to_edge(false),
            Command::StepCycle => self.project.step_to_edge(true),
            Command::ResetSim => self.project.reset_simulation(),
            Command::NextWorkspace => self.workspace = (self.workspace + 1) % n,
            Command::PrevWorkspace => self.workspace = (self.workspace + n - 1) % n,
            Command::Help => self.help.show_section(0),
            Command::Copy => {
                if let Some(text) = self.project.copy_selection() {
                    commands::set_clipboard(ctx, text);
                }
            }
            Command::Cut => {
                if let Some(text) = self.project.cut_selection() {
                    commands::set_clipboard(ctx, text);
                }
            }
            // Z hlavní nabídky se vkládá 2 body vedle původní polohy.
            // [en] From the main menu, pasting goes 2 points next to the original position.
            Command::Paste => {
                if let Some(text) = commands::take_paste(ctx) {
                    self.project.paste_text(&text, None);
                }
            }
            // Potřebují pohled schématu (polohu myši, kameru) – jen z klávesnice nad schématem.
            // [en] They need a schematic view (mouse position, camera) – only from the keyboard over the schematic.
            Command::Add | Command::Grab | Command::Duplicate | Command::FitView => {}
        }
    }

    /// Pracovní plocha Simulace = režim simulace: přepnutím na ni se simulace spustí
    /// (schéma se zamkne), odchodem z ní se zastaví. Zastavení simulace na ploše Simulace
    /// (Esc, Shift+F5) vrátí na plochu, odkud se přišlo. Jinde jde simulaci spustit
    /// i klávesou F5 bez změny plochy.
    ///
    /// [en] The Simulation workspace = simulation mode: switching to it starts the
    /// simulation (the schematic is locked), leaving it stops it. Stopping the simulation
    /// on the Simulation workspace (Esc, Shift+F5) returns to the workspace it came from.
    /// Elsewhere the simulation can also be started with F5 without changing the workspace.
    fn sync_workspace_mode(&mut self) {
        let sim_ws = Workspace::ALL
            .iter()
            .position(|w| *w == Workspace::Simulation)
            .unwrap_or(1);
        let step = workspace_rule(
            self.shown_workspace,
            self.workspace,
            self.return_workspace,
            sim_ws,
            self.project.is_simulating(),
        );
        match step.mode {
            Some(true) => self.project.start_simulation(),
            Some(false) => self.project.stop_simulation(),
            None => {}
        }
        self.workspace = step.workspace;
        self.return_workspace = step.return_to;
        self.shown_workspace = self.workspace;
    }

    /// Položky nabídky pro seznam příkazů; `None` znamená oddělovač.
    ///
    /// [en] Menu items for a list of commands; `None` means a separator.
    fn menu_commands(&mut self, ui: &mut egui::Ui, items: &[Option<Command>]) {
        for item in items {
            match *item {
                Some(cmd) => {
                    if commands::menu_item(ui, cmd, self.lang, &self.project) {
                        let ctx = ui.ctx().clone();
                        self.run(&ctx, cmd);
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
                            &[None, Some(Command::Save), Some(Command::SaveAs), None],
                        );
                        if ui.button(tr(lang, T::PropsMenu)).clicked() {
                            self.properties = Some(self.project.info.clone());
                        }
                        self.menu_commands(ui, &[None, Some(Command::Quit)]);
                    });
                    ui.menu_button(tr(lang, T::MenuEdit), |ui| {
                        self.menu_commands(
                            ui,
                            &[
                                Some(Command::Undo),
                                Some(Command::Redo),
                                None,
                                Some(Command::Copy),
                                Some(Command::Cut),
                                Some(Command::Paste),
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
                        ui.separator();
                        if ui.button(tr(lang, T::SettingsMenu)).clicked() {
                            self.settings_open = true;
                        }
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
                                Some(Command::StepEdge),
                                Some(Command::StepCycle),
                                Some(Command::Stop),
                                None,
                                Some(Command::ResetSim),
                                None,
                            ],
                        );
                        ui.menu_button(tr(lang, T::SimSpeed), |ui| {
                            for v in crate::project::SPEED_CHOICES {
                                ui.radio_value(&mut self.speed, v, speed_text(v));
                            }
                        });
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
                    if self.project.has_clocks() || self.project.time > 0.0 {
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                tr(lang, T::SimTime),
                                time_text(self.project.time, lang)
                            ))
                            .monospace()
                            .small()
                            .color(colors::TEXT_WEAK),
                        );
                    }
                    // Rychlost simulace – kliknutím jde změnit.
                    // [en] Simulation speed – it can be changed by clicking.
                    egui::ComboBox::from_id_salt("sim_speed")
                        .selected_text(RichText::new(speed_text(self.speed)).small())
                        .width(56.0)
                        .show_ui(ui, |ui| {
                            for v in crate::project::SPEED_CHOICES {
                                ui.selectable_value(&mut self.speed, v, speed_text(v));
                            }
                        })
                        .response
                        .on_hover_text(tr(lang, T::SimSpeed));
                    for (on, key) in [
                        (self.project.sim.is_oscillating(), T::Oscillating),
                        (self.project.sim.has_power_short(), T::PowerShort),
                    ] {
                        if on {
                            ui.label(
                                RichText::new(tr(lang, key))
                                    .small()
                                    .color(egui::Color32::from_rgb(0xff, 0x70, 0x60)),
                            );
                        }
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

    /// Uloží soubor s nastavením, pokud se nastavení od posledního uložení změnilo.
    ///
    /// [en] Saves the settings file if the settings changed since they were saved last.
    fn save_settings_if_changed(&mut self) {
        self.settings.lang = self.lang;
        self.settings.symbols = self.symbols;
        self.settings.edit_while_simulating = self.edit_while_simulating;
        if self.saved_settings.as_ref() == Some(&self.settings) {
            return;
        }
        if let Some(path) = &self.settings_path {
            self.settings_error = settings::save(path, &self.settings).err();
        }
        self.saved_settings = Some(self.settings.clone());
    }

    /// Převezme nastavení (např. po obnovení výchozího) do polí okna a knihoven.
    ///
    /// [en] Applies settings (e.g. after restoring the defaults) to the window fields and libraries.
    fn apply_settings(&mut self, s: Settings) {
        self.lang = s.lang;
        self.symbols = s.symbols;
        self.edit_while_simulating = s.edit_while_simulating;
        self.settings = s;
        self.libraries.configure(&self.settings);
    }

    /// Otevře okno pro výběr složky (pro knihovny).
    ///
    /// [en] Opens the dialog for choosing a folder (for libraries).
    fn choose_folder(&mut self, target: FolderTarget, start: Option<PathBuf>) {
        self.folder_target = Some(target);
        self.file_dialog = Some(FileDialog::new(
            filedialog::Mode::Folder,
            start
                .filter(|p| p.is_dir())
                .or_else(|| self.last_dir.clone()),
            "",
        ));
    }

    /// Vybraná složka z okna pro soubory: osobní nebo nová sdílená složka knihoven.
    ///
    /// [en] Folder chosen in the file dialog: personal or a new shared library folder.
    fn folder_chosen(&mut self, path: PathBuf) {
        match self.folder_target.take() {
            Some(FolderTarget::PersonalLibraries) => {
                self.settings.personal_libraries = Some(path);
            }
            Some(FolderTarget::SharedLibrary) => {
                if !self
                    .settings
                    .shared_libraries
                    .iter()
                    .any(|f| f.path == path)
                {
                    self.settings.shared_libraries.push(SharedFolder {
                        path,
                        read_only: true,
                    });
                }
            }
            None => return,
        }
        self.libraries.configure(&self.settings);
    }

    /// Okno Vlastnosti obvodu: název, autoři, popis, licence, škola a data – ukládají
    /// se do souboru obvodu. Úpravy se projeví až tlačítkem OK.
    ///
    /// [en] Circuit properties window: title, authors, description, licence, school and
    /// dates – saved in the circuit file. Changes take effect only with the OK button.
    fn properties_window(&mut self, ctx: &egui::Context) {
        let Some(mut info) = self.properties.take() else {
            return;
        };
        let lang = self.lang;
        let weak = |t: &str| RichText::new(t).small().color(colors::TEXT_WEAK);
        let me = self.settings.author.trim().to_owned();
        let mut open = true;
        let mut done: Option<bool> = None;
        egui::Window::new(tr(lang, T::PropsTitle))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(480.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Grid::new("circuit_properties")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(tr(lang, T::InfoTitle));
                        ui.add(egui::TextEdit::singleline(&mut info.title).desired_width(320.0));
                        ui.end_row();

                        ui.label(tr(lang, T::InfoAuthors));
                        ui.vertical(|ui| {
                            let mut remove = None;
                            for (i, a) in info.authors.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(a).desired_width(240.0));
                                    if ui.button(tr(lang, T::SettingsRemove)).clicked() {
                                        remove = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = remove {
                                info.authors.remove(i);
                            }
                            ui.horizontal_wrapped(|ui| {
                                if ui.button(tr(lang, T::InfoAddAuthor)).clicked() {
                                    info.authors.push(String::new());
                                }
                                if !me.is_empty() && !info.has_author(&me) {
                                    info.authors.retain(|a| !a.trim().is_empty());
                                    let label = if info.authors.is_empty() {
                                        T::InfoAddMe
                                    } else {
                                        T::InfoAddMeCoauthor
                                    };
                                    if ui.button(format!("{}: {me}", tr(lang, label))).clicked() {
                                        info.add_author(&me);
                                    }
                                }
                            });
                            if me.is_empty() {
                                ui.label(weak(tr(lang, T::InfoSetName)));
                            }
                        });
                        ui.end_row();

                        ui.label(tr(lang, T::InfoDescription));
                        ui.add(
                            egui::TextEdit::multiline(&mut info.description)
                                .desired_rows(3)
                                .desired_width(320.0),
                        );
                        ui.end_row();

                        ui.label(tr(lang, T::InfoLicense));
                        license_picker(ui, "circuit_license", &mut info.license, lang);
                        ui.end_row();

                        ui.label(tr(lang, T::InfoSchool));
                        ui.add(
                            egui::TextEdit::singleline(&mut info.school)
                                .hint_text(tr(lang, T::InfoSchoolHint))
                                .desired_width(320.0),
                        );
                        ui.end_row();

                        let or_not_yet = |d: &str| {
                            if d.is_empty() {
                                tr(lang, T::InfoNotYet).to_owned()
                            } else {
                                d.to_owned()
                            }
                        };
                        ui.label(tr(lang, T::InfoCreated));
                        let mut created = or_not_yet(&info.created);
                        if !info.app_version.is_empty() {
                            created = format!(
                                "{created}, {} {}",
                                tr(lang, T::InfoInVersion),
                                info.app_version
                            );
                        }
                        ui.label(weak(&created));
                        ui.end_row();
                        ui.label(tr(lang, T::InfoModified));
                        ui.label(weak(&or_not_yet(&info.modified)));
                        ui.end_row();
                    });
                ui.add_space(4.0);
                ui.label(weak(tr(lang, T::InfoNote)));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button(tr(lang, T::Ok)).clicked() {
                        done = Some(true);
                    }
                    if ui.button(tr(lang, T::Cancel)).clicked() {
                        done = Some(false);
                    }
                });
            });
        match (open, done) {
            (_, Some(true)) => {
                tidy_info(&mut info);
                if info != self.project.info {
                    self.project.info = info;
                    self.project.modified = true;
                }
            }
            (false, _) | (_, Some(false)) => {}
            (true, None) => self.properties = Some(info),
        }
    }

    /// Okno Nastavení: obecné volby, autor a licence, složky knihoven, soubor s nastavením.
    ///
    /// [en] Settings window: general options, author and licence, library folders, settings file.
    fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let lang = self.lang;
        let weak = |t: &str| RichText::new(t).small().color(colors::TEXT_WEAK);
        let before_libs = (
            self.settings.personal_libraries.clone(),
            self.settings.shared_libraries.clone(),
        );
        let mut open = true;
        let mut choose: Option<(FolderTarget, Option<PathBuf>)> = None;
        let mut reset = false;
        egui::Window::new(tr(lang, T::SettingsTitle))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(tr(lang, T::SettingsGeneral));
                egui::Grid::new("settings_general")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(tr(lang, T::Language));
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.lang, Lang::Cs, "Čeština");
                            ui.radio_value(&mut self.lang, Lang::En, "English");
                        });
                        ui.end_row();
                        ui.label(tr(lang, T::Symbols));
                        ui.horizontal(|ui| {
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
                        ui.end_row();
                    });
                ui.checkbox(
                    &mut self.edit_while_simulating,
                    tr(lang, T::EditWhileSimulating),
                );

                ui.add_space(8.0);
                ui.heading(tr(lang, T::SettingsAuthorSection));
                egui::Grid::new("settings_author")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(tr(lang, T::SettingsAuthor));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.settings.author)
                                .hint_text(tr(lang, T::SettingsAuthorHint))
                                .desired_width(280.0),
                        );
                        ui.end_row();
                        ui.label(tr(lang, T::SettingsLicense));
                        license_picker(ui, "settings_license", &mut self.settings.license, lang);
                        ui.end_row();
                    });
                ui.label(weak(tr(lang, T::SettingsAuthorNote)));

                ui.add_space(8.0);
                ui.heading(tr(lang, T::SettingsLibraries));
                ui.label(tr(lang, T::SettingsPersonalFolder));
                let personal = self.settings.personal_folder();
                ui.label(weak(&personal.as_ref().map_or_else(
                    || tr(lang, T::LibNoFolder).to_owned(),
                    |p| p.display().to_string(),
                )));
                ui.horizontal(|ui| {
                    if ui.button(tr(lang, T::SettingsChange)).clicked() {
                        choose = Some((FolderTarget::PersonalLibraries, personal.clone()));
                    }
                    if ui
                        .add_enabled(
                            self.settings.personal_libraries.is_some(),
                            egui::Button::new(tr(lang, T::SettingsDefault)),
                        )
                        .clicked()
                    {
                        self.settings.personal_libraries = None;
                    }
                });
                ui.add_space(4.0);
                ui.label(tr(lang, T::SettingsShared));
                ui.label(weak(tr(lang, T::SettingsSharedNote)));
                let mut remove = None;
                for (i, f) in self.settings.shared_libraries.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut f.read_only, tr(lang, T::SettingsReadOnly));
                        if ui.button(tr(lang, T::SettingsRemove)).clicked() {
                            remove = Some(i);
                        }
                        ui.label(f.path.display().to_string());
                    });
                }
                if let Some(i) = remove {
                    self.settings.shared_libraries.remove(i);
                }
                if self.settings.shared_libraries.is_empty() {
                    ui.label(weak(tr(lang, T::SettingsNoShared)));
                }
                if ui.button(tr(lang, T::SettingsAddShared)).clicked() {
                    choose = Some((FolderTarget::SharedLibrary, None));
                }

                ui.add_space(8.0);
                ui.separator();
                ui.label(tr(lang, T::SettingsFile));
                match &self.settings_path {
                    Some(p) => ui.label(weak(&p.display().to_string())),
                    None => ui.label(weak(tr(lang, T::SettingsNoPath))),
                };
                if let Some(e) = &self.settings_error {
                    ui.label(
                        RichText::new(format!("{}: {e}", tr(lang, T::SettingsSaveFailed)))
                            .small()
                            .color(theme::signal_color(logicflow_core::LogicValue::X)),
                    );
                }
                ui.horizontal(|ui| {
                    if let Some(dir) = self.settings_path.as_ref().and_then(|p| p.parent())
                        && ui.button(tr(lang, T::SettingsOpenFolder)).clicked()
                        && let Err(e) = crate::libraries::open_folder(dir)
                    {
                        self.settings_error = Some(e);
                    }
                    if !self.settings_reset && ui.button(tr(lang, T::SettingsReset)).clicked() {
                        self.settings_reset = true;
                    }
                });
                if self.settings_reset {
                    ui.label(tr(lang, T::SettingsResetConfirm));
                    ui.horizontal(|ui| {
                        if ui.button(tr(lang, T::SettingsResetYes)).clicked() {
                            reset = true;
                            self.settings_reset = false;
                        }
                        if ui.button(tr(lang, T::Cancel)).clicked() {
                            self.settings_reset = false;
                        }
                    });
                }
            });
        if !open {
            self.settings_open = false;
            self.settings_reset = false;
        }
        if reset {
            self.apply_settings(Settings::for_locale(sys_locale::get_locale().as_deref()));
        } else if (
            self.settings.personal_libraries.clone(),
            self.settings.shared_libraries.clone(),
        ) != before_libs
        {
            self.libraries.configure(&self.settings);
        }
        if let Some((target, start)) = choose {
            self.choose_folder(target, start);
        }
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
                    self.folder_target = None;
                }
                filedialog::Outcome::Chosen(path) => {
                    self.last_dir = Some(dialog.dir().to_path_buf());
                    let mode = dialog.mode;
                    self.file_dialog = None;
                    match mode {
                        filedialog::Mode::Folder => self.folder_chosen(path),
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

        self.settings_window(ctx);
        if std::mem::take(&mut self.project.properties_request) {
            self.properties = Some(self.project.info.clone());
        }
        self.properties_window(ctx);

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
                    "© 2026 Mgr. Stanislav Jan Fidrmuc · {}",
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

/// Text rychlosti simulace, např. „×0,25“.
///
/// [en] Simulation speed text, e.g. "×0.25".
fn speed_text(v: f64) -> String {
    format!("×{v}")
}

/// Simulovaný čas pro stavový řádek, např. „12,25 s“ (v angličtině s tečkou).
///
/// [en] Simulated time for the status bar, e.g. "12.25 s" (with a decimal point in English).
fn time_text(t: f64, lang: Lang) -> String {
    let text = format!("{t:.2} s");
    match lang {
        Lang::Cs => text.replace('.', ","),
        Lang::En => text,
    }
}

/// Výsledek pravidla „plocha Simulace = režim simulace“.
///
/// [en] Result of the rule "Simulation workspace = simulation mode".
#[derive(Debug, PartialEq, Eq)]
struct WorkspaceStep {
    /// Plocha, která má být vybraná.
    ///
    /// [en] Workspace that should be selected.
    workspace: usize,
    /// Změna režimu: `Some(true)` spustit simulaci, `Some(false)` zastavit.
    ///
    /// [en] Mode change: `Some(true)` start the simulation, `Some(false)` stop it.
    mode: Option<bool>,
    /// Plocha, na kterou se vrátí zastavení simulace na ploše Simulace.
    ///
    /// [en] Workspace that stopping the simulation on the Simulation workspace returns to.
    return_to: usize,
}

/// Pravidlo „plocha Simulace = režim simulace“ (viz `sync_workspace_mode`).
/// `shown` je plocha v minulém snímku, `current` teď vybraná, `sim_ws` index
/// plochy Simulace.
///
/// [en] The rule "Simulation workspace = simulation mode" (see `sync_workspace_mode`).
/// `shown` is the workspace in the previous frame, `current` the one selected now,
/// `sim_ws` the index of the Simulation workspace.
fn workspace_rule(
    shown: usize,
    current: usize,
    return_to: usize,
    sim_ws: usize,
    simulating: bool,
) -> WorkspaceStep {
    let mut step = WorkspaceStep {
        workspace: current,
        mode: None,
        return_to,
    };
    if current != shown {
        if current == sim_ws {
            // Příchod na plochu Simulace: spustit.
            // [en] Arriving at the Simulation workspace: start.
            step.return_to = shown;
            step.mode = Some(true);
        } else if shown == sim_ws && simulating {
            // Odchod z plochy Simulace: zastavit (stav obvodu zůstává).
            // [en] Leaving the Simulation workspace: stop (the circuit state stays).
            step.mode = Some(false);
        }
    } else if current == sim_ws && !simulating {
        // Simulace zastavená na ploše Simulace (Esc, Shift+F5): zpět na předchozí plochu.
        // [en] Simulation stopped on the Simulation workspace (Esc, Shift+F5): back to the previous workspace.
        step.workspace = if return_to == sim_ws { 0 } else { return_to };
    }
    step
}

/// Pole pro licenci s rozbalovací nabídkou obvyklých licencí (CC BY 4.0…, neuvedeno).
///
/// [en] Licence field with a drop-down of common licences (CC BY 4.0…, not specified).
fn license_picker(ui: &mut egui::Ui, id: &str, license: &mut String, lang: Lang) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(license).desired_width(160.0));
        egui::ComboBox::from_id_salt(id)
            .selected_text("")
            .width(24.0)
            .show_ui(ui, |ui| {
                for l in LICENSE_PRESETS
                    .into_iter()
                    .chain([tr(lang, T::LicenseNone)])
                {
                    if ui.selectable_label(*license == l, l).clicked() {
                        *license = l.to_owned();
                    }
                }
            });
    });
}

/// Uklidí informace z okna Vlastnosti obvodu: mezery na okrajích, prázdní autoři.
///
/// [en] Tidies the information from the Circuit properties window: surrounding spaces,
/// empty authors.
fn tidy_info(info: &mut CircuitInfo) {
    for s in [&mut info.title, &mut info.license, &mut info.school] {
        *s = s.trim().to_owned();
    }
    info.description = info.description.trim().to_owned();
    for a in &mut info.authors {
        *a = a.trim().to_owned();
    }
    info.authors.retain(|a| !a.is_empty());
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
        self.project.speed = self.speed;
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
                .map_or(REPAINT_INTERVAL, |t| {
                    (t / self.speed.max(0.01)).clamp(0.0, REPAINT_INTERVAL)
                });
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
                    libraries: &mut self.libraries,
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
        self.sync_workspace_mode();
        if crate::libraries::take_settings_request(&ctx) {
            self.settings_open = true;
        }
        self.dialogs(&ctx);
        self.update_title(&ctx);
        self.save_settings_if_changed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_workspace_is_simulation_mode() {
        // Plochy: 0 Návrh, 1 Simulace, 2 Analýza.
        // [en] Workspaces: 0 Design, 1 Simulation, 2 Analysis.
        let sim = 1;
        // Přepnutí z Analýzy na Simulaci spustí a zapamatuje si Analýzu.
        // [en] Switching from Analysis to Simulation starts and remembers Analysis.
        let s = workspace_rule(2, sim, 0, sim, false);
        assert_eq!((s.workspace, s.mode, s.return_to), (sim, Some(true), 2));
        // Odchod ze Simulace zastaví.
        // [en] Leaving Simulation stops.
        let s = workspace_rule(sim, 0, 2, sim, true);
        assert_eq!((s.workspace, s.mode), (0, Some(false)));
        // Esc na ploše Simulace: zpět na Analýzu, režim už je zastavený.
        // [en] Esc on the Simulation workspace: back to Analysis, the mode is already stopped.
        let s = workspace_rule(sim, sim, 2, sim, false);
        assert_eq!((s.workspace, s.mode), (2, None));
        // Na ploše Simulace pozastavená simulace (mezerník) plochu nemění.
        // [en] On the Simulation workspace a paused simulation (space) keeps the workspace.
        let s = workspace_rule(sim, sim, 0, sim, true);
        assert_eq!((s.workspace, s.mode), (sim, None));
        // F5 na ploše Návrh: plocha i režim beze změny pravidlem.
        // [en] F5 on the Design workspace: the rule changes neither the workspace nor the mode.
        let s = workspace_rule(0, 0, 0, sim, true);
        assert_eq!((s.workspace, s.mode), (0, None));
    }
}
