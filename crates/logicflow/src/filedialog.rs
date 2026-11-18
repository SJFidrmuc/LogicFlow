//! Vlastní prohlížeč souborů ve stylu Blenderu.
//!
//! Funguje stejně na všech systémech včetně Raspberry Pi a nepotřebuje
//! systémové dialogy (GTK, portály…).

use crate::i18n::{Lang, T, tr};
use crate::theme::colors;
use egui::RichText;
use logicflow_core::ProjectFile;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Open,
    Save,
}

pub enum Outcome {
    None,
    Cancel,
    Chosen(PathBuf),
}

struct Entry {
    name: String,
    is_dir: bool,
}

pub struct FileDialog {
    pub mode: Mode,
    dir: PathBuf,
    dir_text: String,
    file_name: String,
    entries: Vec<Entry>,
    error: Option<String>,
}

pub fn home_dir() -> PathBuf {
    std::env::home_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

impl FileDialog {
    pub fn new(mode: Mode, dir: Option<PathBuf>, file_name: &str) -> Self {
        let mut d = Self {
            mode,
            dir: dir.unwrap_or_else(home_dir),
            dir_text: String::new(),
            file_name: file_name.to_owned(),
            entries: Vec::new(),
            error: None,
        };
        let dir = d.dir.clone();
        d.navigate(&dir);
        d
    }

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
                        (is_dir || ours).then_some(Entry { name, is_dir })
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

    pub fn show(&mut self, ctx: &egui::Context, lang: Lang) -> Outcome {
        let mut outcome = Outcome::None;
        let title = tr(
            lang,
            if self.mode == Mode::Open {
                T::OpenFile
            } else {
                T::SaveFile
            },
        );
        let modal = egui::Modal::new(egui::Id::unique("logicflow-file-dialog")).show(ctx, |ui| {
            ui.set_width(560.0);
            ui.heading(title);
            ui.add_space(4.0);

            // Rychlé složky.
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
                                    format!("▸ {}", e.name)
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

            ui.horizontal(|ui| {
                ui.label(tr(lang, T::FileName));
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.file_name).desired_width(f32::INFINITY),
                );
                if r.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && let Some(p) = self.target()
                {
                    outcome = Outcome::Chosen(p);
                }
            });
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
                    if self.mode == Mode::Open {
                        T::Open
                    } else {
                        T::Save
                    },
                );
                let ok = ui.add_enabled(self.target().is_some(), egui::Button::new(ok_text));
                if ok.clicked()
                    && let Some(p) = self.target()
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

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
