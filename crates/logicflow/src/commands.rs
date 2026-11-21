//! Jednotný seznam příkazů: každá akce programu má na jednom místě název (česky
//! i anglicky), klávesovou zkratku, oblast platnosti a podmínku dostupnosti.
//!
//! Klávesy, hlavní nabídky, tlačítka v panelech a později kontextová nabídka
//! a panel nástrojů (i dotyková verze) berou příkazy odsud – zkratka se tak
//! nikde nepíše dvakrát a v nabídkách je vždy vidět.
//!
//! [en] Unified list of commands: every action of the program has its name (Czech
//! and English), keyboard shortcut, scope and availability condition in one place.
//!
//! Keys, main menus, panel buttons and later the context menu and the toolbar
//! (also the touch version) take commands from here – a shortcut is thus never
//! written twice and is always visible in the menus.

use crate::i18n::{Lang, T, tr};
use crate::project::Project;
use egui::{Event, Key, Modifiers, Ui};

/// Klávesová zkratka: modifikátory a klávesa.
///
/// [en] Keyboard shortcut: modifiers and key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    /// Modifikátory (Ctrl / Cmd, Shift, Alt).
    ///
    /// [en] Modifiers (Ctrl / Cmd, Shift, Alt).
    pub mods: Modifiers,
    /// Klávesa (logická – podle rozložení klávesnice).
    ///
    /// [en] Key (logical – according to the keyboard layout).
    pub key: Key,
}

impl Shortcut {
    /// Text zkratky pro nabídky a nápovědu, např. „Ctrl+Shift+Z“ nebo „mezerník“.
    /// Na macOS se místo Ctrl píše Cmd.
    ///
    /// [en] Shortcut text for menus and help, e.g. "Ctrl+Shift+Z" or "Space".
    /// On macOS, Cmd is written instead of Ctrl.
    pub fn text(self, lang: Lang, mac: bool) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.mods.command || self.mods.ctrl || self.mods.mac_cmd {
            parts.push(if mac { "Cmd" } else { "Ctrl" });
        }
        if self.mods.shift {
            parts.push("Shift");
        }
        if self.mods.alt {
            parts.push(if mac { "Option" } else { "Alt" });
        }
        let key = match (self.key, lang) {
            (Key::Space, Lang::Cs) => "mezerník",
            (Key::Space, Lang::En) => "Space",
            (Key::ArrowRight, Lang::Cs) => "šipka vpravo",
            (Key::ArrowRight, Lang::En) => "Right arrow",
            (Key::Escape, _) => "Esc",
            (key, _) => key.name(),
        };
        parts.push(key);
        parts.join("+")
    }
}

/// Kde příkaz z klávesnice platí.
///
/// [en] Where a keyboard command applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// V celém okně (Ctrl+S, mezerník, F1…).
    ///
    /// [en] In the whole window (Ctrl+S, space, F1…).
    Global,
    /// Jen v editoru schématu, nad kterým je myš (Shift+A, G, R, X…).
    ///
    /// [en] Only in the schematic editor under the mouse (Shift+A, G, R, X…).
    Schematic,
}

/// Příkaz programu.
///
/// [en] Program command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    /// Nový prázdný obvod.
    ///
    /// [en] New empty circuit.
    New,
    /// Otevřít soubor.
    ///
    /// [en] Open a file.
    Open,
    /// Uložit.
    ///
    /// [en] Save.
    Save,
    /// Uložit jako.
    ///
    /// [en] Save as.
    SaveAs,
    /// Ukončit program.
    ///
    /// [en] Quit the program.
    Quit,
    /// Zpět.
    ///
    /// [en] Undo.
    Undo,
    /// Znovu.
    ///
    /// [en] Redo.
    Redo,
    /// Nabídka Přidat součástku u ukazatele myši.
    ///
    /// [en] Add component menu at the mouse pointer.
    Add,
    /// Přesunout výběr (končí kliknutím).
    ///
    /// [en] Move the selection (ends with a click).
    Grab,
    /// Otočit výběr (nebo vkládanou součástku) o 90°.
    ///
    /// [en] Rotate the selection (or the component being placed) by 90°.
    Rotate,
    /// Smazat výběr.
    ///
    /// [en] Delete the selection.
    Delete,
    /// Duplikovat výběr a přesouvat kopii.
    ///
    /// [en] Duplicate the selection and move the copy.
    Duplicate,
    /// Vybrat vše.
    ///
    /// [en] Select all.
    SelectAll,
    /// Zrušit výběr.
    ///
    /// [en] Deselect.
    SelectNone,
    /// Zrušit rozdělanou akci (nebo výběr).
    ///
    /// [en] Cancel the action in progress (or the selection).
    Cancel,
    /// Zobrazit celé schéma.
    ///
    /// [en] Fit the whole schematic in view.
    FitView,
    /// Otevřít vybraný podobvod, nebo se vrátit o úroveň výš.
    ///
    /// [en] Open the selected subcircuit, or go one level up.
    Subcircuit,
    /// Spustit nebo pozastavit simulaci.
    ///
    /// [en] Run or pause the simulation.
    RunPause,
    /// Jeden takt simulace (při pozastavení).
    ///
    /// [en] One simulation tick (when paused).
    Step,
    /// Restartovat simulaci.
    ///
    /// [en] Restart the simulation.
    ResetSim,
    /// Další pracovní plocha.
    ///
    /// [en] Next workspace.
    NextWorkspace,
    /// Předchozí pracovní plocha.
    ///
    /// [en] Previous workspace.
    PrevWorkspace,
    /// Nápověda.
    ///
    /// [en] Help.
    Help,
}

/// Ctrl (na macOS Cmd).
///
/// [en] Ctrl (Cmd on macOS).
const CMD: Modifiers = Modifiers::COMMAND;
/// Ctrl+Shift (na macOS Cmd+Shift).
///
/// [en] Ctrl+Shift (Cmd+Shift on macOS).
const CMD_SHIFT: Modifiers = Modifiers {
    shift: true,
    ..Modifiers::COMMAND
};
/// Bez modifikátorů.
///
/// [en] No modifiers.
const NONE: Modifiers = Modifiers::NONE;

impl Command {
    /// Všechny příkazy (pořadí = pořadí v nápovědě).
    ///
    /// [en] All commands (order = order in the help).
    pub const ALL: [Command; 23] = [
        Command::New,
        Command::Open,
        Command::Save,
        Command::SaveAs,
        Command::Quit,
        Command::Undo,
        Command::Redo,
        Command::Add,
        Command::Grab,
        Command::Rotate,
        Command::Delete,
        Command::Duplicate,
        Command::SelectAll,
        Command::SelectNone,
        Command::Cancel,
        Command::FitView,
        Command::Subcircuit,
        Command::RunPause,
        Command::Step,
        Command::ResetSim,
        Command::NextWorkspace,
        Command::PrevWorkspace,
        Command::Help,
    ];

    /// Klávesové zkratky; první se ukazuje v nabídkách, další jsou alternativy.
    ///
    /// [en] Keyboard shortcuts; the first is shown in menus, the others are alternatives.
    pub fn shortcuts(self) -> &'static [Shortcut] {
        use Command::*;
        // Struktura zapsaná přímo (ne voláním funkce), aby šlo vrátit odkaz na konstantu.
        // [en] The struct is written directly (not by a function call) so that a reference to a constant can be returned.
        macro_rules! s {
            ($mods:expr, $key:ident) => {
                Shortcut {
                    mods: $mods,
                    key: Key::$key,
                }
            };
        }
        match self {
            New => &[s!(CMD, N)],
            Open => &[s!(CMD, O)],
            Save => &[s!(CMD, S)],
            SaveAs => &[s!(CMD_SHIFT, S)],
            Quit => &[s!(CMD, Q)],
            Undo => &[s!(CMD, Z)],
            Redo => &[s!(CMD_SHIFT, Z), s!(CMD, Y)],
            Add => &[s!(Modifiers::SHIFT, A)],
            Grab => &[s!(NONE, G)],
            Rotate => &[s!(NONE, R)],
            Delete => &[s!(NONE, X), s!(NONE, Delete)],
            Duplicate => &[s!(Modifiers::SHIFT, D)],
            SelectAll => &[s!(NONE, A)],
            SelectNone => &[s!(Modifiers::ALT, A)],
            Cancel => &[s!(NONE, Escape)],
            FitView => &[s!(NONE, Home)],
            Subcircuit => &[s!(NONE, Tab)],
            RunPause => &[s!(NONE, Space)],
            Step => &[s!(NONE, ArrowRight)],
            ResetSim => &[s!(CMD, R)],
            NextWorkspace => &[s!(CMD, PageDown)],
            PrevWorkspace => &[s!(CMD, PageUp)],
            Help => &[s!(NONE, F1)],
        }
    }

    /// Hlavní zkratka (ukazuje se v nabídkách a tooltipech).
    ///
    /// [en] Main shortcut (shown in menus and tooltips).
    pub fn shortcut(self) -> Shortcut {
        self.shortcuts()[0]
    }

    /// Kde příkaz z klávesnice platí.
    ///
    /// [en] Where the keyboard command applies.
    pub fn scope(self) -> Scope {
        use Command::*;
        match self {
            Add | Grab | Rotate | Delete | Duplicate | SelectAll | SelectNone | Cancel
            | FitView | Subcircuit => Scope::Schematic,
            _ => Scope::Global,
        }
    }

    /// Platí zkratka i při psaní do textového pole? (Ctrl+S ano, Ctrl+Z nebo
    /// mezerník ne – ty patří textovému poli.)
    ///
    /// [en] Does the shortcut work while typing in a text field? (Ctrl+S yes, Ctrl+Z or
    /// space no – those belong to the text field.)
    pub fn while_typing(self) -> bool {
        use Command::*;
        matches!(
            self,
            New | Open | Save | SaveAs | Quit | ResetSim | NextWorkspace | PrevWorkspace | Help
        )
    }

    /// Název příkazu (bez zkratky).
    ///
    /// [en] Command name (without the shortcut).
    pub fn label(self, lang: Lang) -> &'static str {
        use Command::*;
        tr(
            lang,
            match self {
                New => T::New,
                Open => T::Open,
                Save => T::Save,
                SaveAs => T::SaveAs,
                Quit => T::Quit,
                Undo => T::Undo,
                Redo => T::Redo,
                Add => T::CmdAdd,
                Grab => T::CmdGrab,
                Rotate => T::CmdRotate,
                Delete => T::CmdDelete,
                Duplicate => T::CmdDuplicate,
                SelectAll => T::CmdSelectAll,
                SelectNone => T::CmdSelectNone,
                Cancel => T::CmdCancel,
                FitView => T::FitView,
                Subcircuit => T::CmdSubcircuit,
                RunPause => T::CmdRunPause,
                Step => T::Step,
                ResetSim => T::ResetSim,
                NextWorkspace => T::CmdNextWorkspace,
                PrevWorkspace => T::CmdPrevWorkspace,
                Help => T::Manual,
            },
        )
    }

    /// Název podle aktuálního stavu (Spustit / Pozastavit, Otevřít podobvod / Zpět
    /// o úroveň výš) – pro nabídky a tlačítka.
    ///
    /// [en] Name according to the current state (Run / Pause, Open subcircuit / Back one
    /// level) – for menus and buttons.
    pub fn label_for(self, lang: Lang, project: &Project) -> &'static str {
        match self {
            Command::RunPause => tr(lang, if project.running { T::Pause } else { T::Run }),
            Command::Subcircuit if project.selected_subcircuit().is_none() => {
                tr(lang, T::CmdBackToParent)
            }
            Command::Subcircuit => tr(lang, T::CmdOpenSubcircuit),
            _ => self.label(lang),
        }
    }

    /// Název se zkratkou v závorce, např. „Smazat (X)“ – pro tlačítka v panelech.
    ///
    /// [en] Name with the shortcut in brackets, e.g. "Delete (X)" – for panel buttons.
    pub fn button_text(self, lang: Lang, project: &Project) -> String {
        with_shortcut(self.label_for(lang, project), self, lang)
    }

    /// Dá se příkaz právě teď provést? (Nedostupné položky nabídek jsou zašedlé.)
    ///
    /// [en] Can the command be carried out right now? (Unavailable menu items are greyed out.)
    pub fn available(self, project: &Project) -> bool {
        use Command::*;
        match self {
            Undo => project.can_undo(),
            Redo => project.can_redo(),
            Grab | Rotate | Delete | Duplicate | SelectNone => !project.selection.is_empty(),
            SelectAll => !project.circuit.components.is_empty(),
            Subcircuit => project.selected_subcircuit().is_some() || project.sheet.is_some(),
            Step => !project.running,
            _ => true,
        }
    }
}

/// Text se zkratkou příkazu v závorce, např. „Zpět o úroveň výš (Tab)“.
///
/// [en] Text with the command shortcut in brackets, e.g. "Back one level (Tab)".
pub fn with_shortcut(text: &str, cmd: Command, lang: Lang) -> String {
    format!("{text} ({})", cmd.shortcut().text(lang, false))
}

/// Je to stisk klávesy odpovídající zkratce? Modifikátory se porovnávají přesně
/// (Shift+X tedy není X), Ctrl a Cmd podle systému.
///
/// [en] Is this a key press matching the shortcut? Modifiers are compared exactly
/// (so Shift+X is not X), Ctrl and Cmd according to the system.
fn matches(event: &Event, sc: Shortcut) -> bool {
    matches!(
        event,
        Event::Key { key, modifiers, pressed: true, .. }
            if *key == sc.key && modifiers.matches_exact(sc.mods)
    )
}

/// Vybere ze vstupu stisky zkratek příkazů dané oblasti a odebere je, aby je nikdo
/// jiný nezpracoval. Při psaní do textového pole se berou jen příkazy, které tam
/// platí. Vrací příkazy v pořadí stisků.
///
/// [en] Takes the presses of command shortcuts of the given scope from the input and
/// removes them so that nobody else handles them. While typing in a text field, only
/// commands that apply there are taken. Returns the commands in the order of presses.
pub fn take_pressed(ctx: &egui::Context, scope: Scope) -> Vec<Command> {
    let typing = ctx.egui_wants_keyboard_input();
    let candidates: Vec<Command> = Command::ALL
        .into_iter()
        .filter(|c| c.scope() == scope && (!typing || c.while_typing()))
        .collect();
    let mut found = Vec::new();
    ctx.input_mut(|i| {
        i.events.retain(|e| {
            let hit = candidates
                .iter()
                .find(|c| c.shortcuts().iter().any(|&sc| matches(e, sc)));
            if let Some(&c) = hit {
                found.push(c);
            }
            hit.is_none()
        });
    });
    found
}

/// Položka nabídky pro příkaz: název podle stavu, zkratka vpravo, zašedlá, když
/// příkaz nejde provést. Vrací `true`, když na ni uživatel klikl.
///
/// [en] Menu item for a command: name according to the state, shortcut on the right,
/// greyed out when the command cannot be carried out. Returns `true` when clicked.
pub fn menu_item(ui: &mut Ui, cmd: Command, lang: Lang, project: &Project) -> bool {
    let mac = ui.ctx().os() == egui::os::OperatingSystem::Mac;
    let button = egui::Button::new(cmd.label_for(lang, project))
        .shortcut_text(cmd.shortcut().text(lang, mac));
    ui.add_enabled(cmd.available(project), button).clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Každá zkratka patří jen jednomu příkazu (v rámci stejné oblasti i napříč –
    /// globální zkratky platí i nad schématem).
    ///
    /// [en] Every shortcut belongs to one command only (within the same scope and
    /// across – global shortcuts also work over the schematic).
    #[test]
    fn shortcuts_are_unique() {
        let mut seen: Vec<(Shortcut, Command)> = Vec::new();
        for cmd in Command::ALL {
            for &sc in cmd.shortcuts() {
                if let Some((_, other)) = seen.iter().find(|(s, _)| *s == sc) {
                    panic!("{cmd:?} a {other:?} mají stejnou zkratku {sc:?}");
                }
                seen.push((sc, cmd));
            }
        }
    }

    /// Seznam ALL obsahuje každý příkaz právě jednou a každý má oba texty.
    ///
    /// [en] The ALL list contains every command exactly once and each has both texts.
    #[test]
    fn all_commands_have_texts() {
        for (i, cmd) in Command::ALL.iter().enumerate() {
            assert!(!Command::ALL[..i].contains(cmd), "{cmd:?} je v ALL dvakrát");
            for lang in [Lang::Cs, Lang::En] {
                assert!(!cmd.label(lang).is_empty(), "{cmd:?} nemá text");
            }
            assert_ne!(
                cmd.label(Lang::Cs),
                cmd.label(Lang::En),
                "{cmd:?} nemá překlad"
            );
        }
    }

    /// Texty zkratek.
    ///
    /// [en] Shortcut texts.
    #[test]
    fn shortcut_texts() {
        assert_eq!(
            Command::Redo.shortcut().text(Lang::Cs, false),
            "Ctrl+Shift+Z"
        );
        assert_eq!(Command::Redo.shortcut().text(Lang::En, true), "Cmd+Shift+Z");
        assert_eq!(
            Command::RunPause.shortcut().text(Lang::Cs, false),
            "mezerník"
        );
        assert_eq!(
            Command::Step.shortcut().text(Lang::En, false),
            "Right arrow"
        );
        assert_eq!(
            Command::SelectNone.shortcut().text(Lang::Cs, false),
            "Alt+A"
        );
        assert_eq!(Command::Cancel.shortcut().text(Lang::Cs, false), "Esc");
    }

    /// Nápověda (kapitola Klávesové zkratky) uvádí všechny zkratky příkazů v obou jazycích.
    ///
    /// [en] The help (Keyboard shortcuts chapter) lists all command shortcuts in both languages.
    #[test]
    fn help_lists_all_shortcuts() {
        for (lang, dir) in [(Lang::Cs, "cs"), (Lang::En, "en")] {
            let file = format!(
                "{}/../../docs/napoveda/{dir}/src/klavesove-zkratky.md",
                env!("CARGO_MANIFEST_DIR")
            );
            let text = std::fs::read_to_string(&file).expect("kapitola nápovědy");
            for cmd in Command::ALL {
                for sc in cmd.shortcuts() {
                    let key = sc.text(lang, false);
                    assert!(
                        text.split(['|', ',']).any(|cell| cell.trim() == key),
                        "{file}: chybí zkratka {key} ({cmd:?})"
                    );
                }
            }
        }
    }

    /// Přesné porovnání modifikátorů: Shift+X není X, Ctrl+S není Ctrl+Shift+S.
    ///
    /// [en] Exact modifier matching: Shift+X is not X, Ctrl+S is not Ctrl+Shift+S.
    #[test]
    fn exact_modifiers() {
        let press = |key, modifiers| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let x = Command::Delete.shortcut();
        assert!(matches(&press(Key::X, Modifiers::NONE), x));
        assert!(!matches(&press(Key::X, Modifiers::SHIFT), x));
        let ctrl = Modifiers {
            ctrl: true,
            command: true,
            ..Modifiers::NONE
        };
        let ctrl_shift = Modifiers {
            shift: true,
            ..ctrl
        };
        assert!(matches(&press(Key::S, ctrl), Command::Save.shortcut()));
        assert!(!matches(
            &press(Key::S, ctrl_shift),
            Command::Save.shortcut()
        ));
        assert!(matches(
            &press(Key::S, ctrl_shift),
            Command::SaveAs.shortcut()
        ));
    }
}
