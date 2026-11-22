//! Vlastní prohlížeč souborů ve stylu Blenderu.
//!
//! Funguje stejně na všech systémech včetně Raspberry Pi a nepotřebuje
//! systémové dialogy (GTK, portály…).
//!
//! [en] Own file browser in the style of Blender.
//!
//! It works the same on all systems including Raspberry Pi and needs no
//! system dialogs (GTK, portals…).

use crate::i18n::{Lang, T, tr};
use crate::theme::colors;
use egui::RichText;
use logicflow_core::{CircuitInfo, ProjectFile};
use std::path::{Path, PathBuf};

/// Účel okna.
///
/// [en] Purpose of the dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Otevření souboru.
    ///
    /// [en] Opening a file.
    Open,
    /// Uložení souboru.
    ///
    /// [en] Saving a file.
    Save,
    /// Výběr složky (vybere se složka, ve které okno právě je).
    ///
    /// [en] Choosing a folder (the folder the dialog is currently in is chosen).
    Folder,
}

/// Výsledek jednoho snímku okna.
///
/// [en] Result of one frame of the dialog.
pub enum Outcome {
    /// Okno zůstává otevřené.
    ///
    /// [en] The dialog stays open.
    None,
    /// Uživatel okno zavřel bez výběru.
    ///
    /// [en] The user closed the dialog without choosing.
    Cancel,
    /// Vybraný soubor.
    ///
    /// [en] The chosen file.
    Chosen(PathBuf),
}

/// Položka ve výpisu složky.
///
/// [en] Entry in the folder listing.
struct Entry {
    /// Název souboru nebo složky.
    ///
    /// [en] File or folder name.
    name: String,
    /// Je to složka?
    ///
    /// [en] Is it a folder?
    is_dir: bool,
}

/// Stav okna pro otevření a uložení souboru.
///
/// [en] State of the open/save dialog.
pub struct FileDialog {
    pub mode: Mode,
    dir: PathBuf,
    dir_text: String,
    file_name: String,
    entries: Vec<Entry>,
    error: Option<String>,
    /// Název rozepsané nové složky (`None` = pole není otevřené).
    ///
    /// [en] Name of the new folder being typed (`None` = the field is not open).
    new_folder: Option<String>,
    /// Náhled informací o vybraném souboru (při otevírání): soubor a jeho informace
    /// (`None` = soubor nejde přečíst).
    ///
    /// [en] Preview of the information about the chosen file (when opening): the file
    /// and its information (`None` = the file cannot be read).
    preview: Option<(PathBuf, Option<CircuitInfo>)>,
}

/// Domovská složka uživatele (záložně aktuální složka).
///
/// [en] The user's home folder (falling back to the current folder).
pub fn home_dir() -> PathBuf {
    std::env::home_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

impl FileDialog {
    /// Otevře okno ve složce `dir` (jinak v domovské) s předvyplněným názvem souboru.
    ///
    /// [en] Opens the dialog in folder `dir` (otherwise the home folder) with a prefilled file name.
    pub fn new(mode: Mode, dir: Option<PathBuf>, file_name: &str) -> Self {
        let mut d = Self {
            mode,
            dir: dir.unwrap_or_else(home_dir),
            dir_text: String::new(),
            file_name: file_name.to_owned(),
            entries: Vec::new(),
            error: None,
            new_folder: None,
            preview: None,
        };
        let dir = d.dir.clone();
        d.navigate(&dir);
        d
    }

    /// Přejde do složky a načte její obsah (složky a soubory .logicflow).
    ///
    /// [en] Goes to a folder and reads its content (folders and .logicflow files).
    fn navigate(&mut self, dir: &Path) {
        match std::fs::read_dir(dir) {
            Ok(rd) => {
                let mut entries: Vec<Entry> = rd
                    .filter_map(Result::ok)
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().into_owned();
                        if name.starts_with('.') {
                            return None;
                        }
                        let is_dir = e.file_type().ok()?.is_dir();
                        let ours = Path::new(&name)
                            .extension()
                            .is_some_and(|x| x.eq_ignore_ascii_case(ProjectFile::EXTENSION));
                        let shown = is_dir || (ours && self.mode != Mode::Folder);
                        shown.then_some(Entry { name, is_dir })
                    })
                    .collect();
                entries.sort_by(|a, b| {
                    b.is_dir
                        .cmp(&a.is_dir)
                        .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
                self.entries = entries;
                self.dir = dir.to_path_buf();
                self.dir_text = self.dir.display().to_string();
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    /// Vytvoří v aktuální složce podsložku a přejde do ní.
    ///
    /// [en] Creates a subfolder in the current folder and goes into it.
    fn create_folder(&mut self, name: &str, lang: Lang) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
            return Err(tr(lang, T::BadFolderName).to_owned());
        }
        let path = self.dir.join(name);
        if path.exists() {
            return Err(tr(lang, T::FolderExists).to_owned());
        }
        std::fs::create_dir(&path).map_err(|e| e.to_string())?;
        self.navigate(&path);
        Ok(())
    }

    /// Cesta vybraného souboru; při ukládání doplní příponu .logicflow.
    ///
    /// [en] Path of the chosen file; when saving, adds the .logicflow extension.
    fn target(&self) -> Option<PathBuf> {
        let name = self.file_name.trim();
        if name.is_empty() {
            return None;
        }
        let mut p = self.dir.join(name);
        if self.mode == Mode::Save
            && !p
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case(ProjectFile::EXTENSION))
        {
            p.set_extension(ProjectFile::EXTENSION);
        }
        Some(p)
    }

    /// Vykreslí okno a vrátí, co uživatel udělal.
    ///
    /// [en] Draws the dialog and returns what the user did.
    pub fn show(&mut self, ctx: &egui::Context, lang: Lang) -> Outcome {
        let mut outcome = Outcome::None;
        let title = tr(
            lang,
            match self.mode {
                Mode::Open => T::OpenFile,
                Mode::Save => T::SaveFile,
                Mode::Folder => T::ChooseFolder,
            },
        );
        let modal = egui::Modal::new(egui::Id::unique("logicflow-file-dialog")).show(ctx, |ui| {
            ui.set_width(560.0);
            ui.heading(title);
            ui.add_space(4.0);

            // Rychlé složky.
            // [en] Quick folders.
            ui.horizontal(|ui| {
                let home = home_dir();
                if ui.button(tr(lang, T::Home)).clicked() {
                    self.navigate(&home);
                }
                for sub in ["Desktop", "Plocha", "Documents", "Dokumenty"] {
                    let p = home.join(sub);
                    if p.is_dir() && ui.button(sub).clicked() {
                        self.navigate(&p);
                    }
                }
                #[cfg(windows)]
                for letter in b'C'..=b'Z' {
                    let p = PathBuf::from(format!("{}:\\", letter as char));
                    if p.exists() && ui.button(format!("{}:", letter as char)).clicked() {
                        self.navigate(&p);
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("..").on_hover_text(tr(lang, T::Parent)).clicked()
                    && let Some(parent) = self.dir.parent().map(Path::to_path_buf)
                {
                    self.navigate(&parent);
                }
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.dir_text).desired_width(f32::INFINITY),
                );
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let p = PathBuf::from(self.dir_text.clone());
                    self.navigate(&p);
                }
            });

            // Nová složka: tlačítko otevře pole pro název, Enter nebo Vytvořit ji založí.
            // [en] New folder: the button opens a name field, Enter or Create makes the folder.
            ui.horizontal(|ui| match self.new_folder.take() {
                None => {
                    if ui.button(format!("+ {}", tr(lang, T::NewFolder))).clicked() {
                        self.new_folder = Some(String::new());
                    }
                }
                Some(mut name) => {
                    ui.label(tr(lang, T::NewFolder));
                    let r = ui.add(egui::TextEdit::singleline(&mut name).desired_width(220.0));
                    if name.is_empty() && !r.has_focus() && !r.lost_focus() {
                        r.request_focus();
                    }
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let create = ui.button(tr(lang, T::CreateFolder)).clicked();
                    let cancel = ui.button(tr(lang, T::Cancel)).clicked();
                    if enter || create {
                        match self.create_folder(&name, lang) {
                            Ok(()) => {}
                            Err(e) => {
                                self.error = Some(e);
                                self.new_folder = Some(name);
                            }
                        }
                    } else if !cancel {
                        self.new_folder = Some(name);
                    }
                }
            });

            egui::Frame::new()
                .fill(colors::PANEL)
                .corner_radius(4)
                .inner_margin(4)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .min_scrolled_height(300.0)
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            let mut go: Option<PathBuf> = None;
                            for e in &self.entries {
                                let label = if e.is_dir {
                                    format!("{}/", e.name)
                                } else {
                                    e.name.clone()
                                };
                                let selected = !e.is_dir && e.name == self.file_name;
                                let r = ui.add(egui::Button::selectable(selected, label));
                                if e.is_dir {
                                    if r.clicked() {
                                        go = Some(self.dir.join(&e.name));
                                    }
                                } else if r.double_clicked() {
                                    self.file_name = e.name.clone();
                                    outcome = Outcome::Chosen(self.dir.join(&e.name));
                                } else if r.clicked() {
                                    self.file_name = e.name.clone();
                                }
                            }
                            if self.entries.is_empty() {
                                ui.label(
                                    RichText::new(tr(lang, T::EmptyFolder))
                                        .color(colors::TEXT_WEAK),
                                );
                            }
                            if let Some(p) = go {
                                self.navigate(&p);
                            }
                        });
                });

            if self.mode != Mode::Folder {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, T::FileName));
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.file_name)
                            .desired_width(f32::INFINITY),
                    );
                    if r.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && let Some(p) = self.target()
                    {
                        outcome = Outcome::Chosen(p);
                    }
                });
            }
            if self.mode == Mode::Open {
                self.show_preview(ui, lang);
            }
            if self.mode == Mode::Save
                && let Some(p) = self.target()
                && p.exists()
            {
                ui.label(
                    RichText::new(tr(lang, T::Overwrite))
                        .color(egui::Color32::from_rgb(0xff, 0x90, 0x60)),
                );
            }
            if let Some(err) = &self.error {
                ui.label(RichText::new(err).color(egui::Color32::from_rgb(0xff, 0x70, 0x60)));
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let ok_text = tr(
                    lang,
                    match self.mode {
                        Mode::Open => T::Open,
                        Mode::Save => T::Save,
                        Mode::Folder => T::ChooseThisFolder,
                    },
                );
                // Ve výběru složky se vybere složka, ve které okno právě je.
                // [en] When choosing a folder, the folder the dialog is in is chosen.
                let target = if self.mode == Mode::Folder {
                    Some(self.dir.clone())
                } else {
                    self.target()
                };
                let ok = ui.add_enabled(target.is_some(), egui::Button::new(ok_text));
                if ok.clicked()
                    && let Some(p) = target
                {
                    outcome = Outcome::Chosen(p);
                }
                if ui.button(tr(lang, T::Cancel)).clicked() {
                    outcome = Outcome::Cancel;
                }
            });
        });
        if modal.should_close() && matches!(outcome, Outcome::None) {
            outcome = Outcome::Cancel;
        }
        outcome
    }

    /// Náhled informací o vybraném obvodu (název, autoři, licence, popis, data);
    /// soubor se přečte jen při změně výběru.
    ///
    /// [en] Preview of the information about the chosen circuit (title, authors,
    /// licence, description, dates); the file is read only when the choice changes.
    fn show_preview(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let Some(path) = self.target().filter(|p| p.is_file()) else {
            self.preview = None;
            return;
        };
        if self.preview.as_ref().is_none_or(|(p, _)| *p != path) {
            let info = std::fs::read_to_string(&path)
                .ok()
                .and_then(|t| ProjectFile::read_info(&t));
            self.preview = Some((path, info));
        }
        let Some((_, info)) = &self.preview else {
            return;
        };
        let weak = |t: &str| RichText::new(t).small().color(colors::TEXT_WEAK);
        egui::Frame::new()
            .fill(colors::PANEL)
            .corner_radius(4)
            .inner_margin(6)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                match info {
                    Some(i) if !i.is_empty() => {
                        if !i.title.is_empty() {
                            ui.label(RichText::new(&i.title).strong());
                        }
                        let byline: Vec<String> =
                            [i.authors_text(), i.license.clone(), i.school.clone()]
                                .into_iter()
                                .filter(|t| !t.is_empty())
                                .collect();
                        if !byline.is_empty() {
                            ui.label(weak(&byline.join("  ·  ")));
                        }
                        if !i.description.is_empty() {
                            ui.label(&i.description);
                        }
                        let dates: Vec<String> =
                            [(T::InfoCreated, &i.created), (T::InfoModified, &i.modified)]
                                .into_iter()
                                .filter(|(_, d)| !d.is_empty())
                                .map(|(k, d)| format!("{}: {d}", tr(lang, k)))
                                .collect();
                        if !dates.is_empty() {
                            ui.label(weak(&dates.join("  ·  ")));
                        }
                    }
                    _ => {
                        ui.label(weak(tr(lang, T::InfoNone)));
                    }
                }
            });
    }

    /// Aktuální složka.
    ///
    /// [en] Current folder.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_folder_and_enters_it() {
        let base = std::env::temp_dir().join(format!("lf-dialog-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let mut d = FileDialog::new(Mode::Save, Some(base.clone()), "obvod");
        d.create_folder("Úlohy 3.A", Lang::Cs).unwrap();
        assert_eq!(d.dir(), base.join("Úlohy 3.A"));
        assert!(base.join("Úlohy 3.A").is_dir());
        assert_eq!(
            d.target(),
            Some(base.join("Úlohy 3.A").join("obvod.logicflow"))
        );
        d.navigate(&base);
        assert!(
            d.create_folder("Úlohy 3.A", Lang::Cs).is_err(),
            "už existuje"
        );
        assert!(d.create_folder("a/b", Lang::Cs).is_err());
        assert!(d.create_folder("  ", Lang::Cs).is_err());
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn folder_mode_lists_only_folders() {
        let base = std::env::temp_dir().join(format!("lf-dialog-slozky-{}", std::process::id()));
        std::fs::create_dir_all(base.join("Knihovny")).unwrap();
        std::fs::write(base.join("obvod.logicflow"), "{}").unwrap();
        let d = FileDialog::new(Mode::Folder, Some(base.clone()), "");
        let names: Vec<&str> = d.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Knihovny"]);
        let d = FileDialog::new(Mode::Open, Some(base.clone()), "");
        assert_eq!(d.entries.len(), 2);
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn save_adds_extension() {
        let tmp = std::env::temp_dir();
        let mut d = FileDialog::new(Mode::Save, Some(tmp.clone()), "obvod");
        assert_eq!(d.target(), Some(tmp.join("obvod.logicflow")));
        d.file_name = "x.LOGICFLOW".into();
        assert_eq!(d.target(), Some(tmp.join("x.LOGICFLOW")));
        d.file_name = "  ".into();
        assert_eq!(d.target(), None);
    }
}
