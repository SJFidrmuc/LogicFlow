//! Soubor s nastavením `logicflow.toml` v domovské složce uživatele.
//!
//! Umístění podle systému (část profilu, která se u cestovních profilů přenáší):
//! Windows `%APPDATA%\LogicFlow`, macOS `~/Library/Application Support/LogicFlow`,
//! Linux a Raspberry Pi `~/.config/logicflow`. Soubor je čitelný text s komentáři,
//! jde upravit i ručně. Chybějící položky se doplní výchozími hodnotami; nečitelný
//! soubor se odloží jako `.bak` a program pokračuje s výchozími hodnotami. Zápis je
//! bezpečný: nový obsah se zapíše vedle a pak se soubor přejmenuje, takže nikdy
//! nezůstane zapsaný napůl.
//!
//! Převod z textu a do textu je bez práce se soubory ([`Settings::from_toml`],
//! [`Settings::to_toml`]) – webová verze ho použije s úložištěm prohlížeče.
//!
//! [en]
//!
//! The settings file `logicflow.toml` in the user's home folder.
//!
//! Location by system (the part of the profile that roams with roaming profiles):
//! Windows `%APPDATA%\LogicFlow`, macOS `~/Library/Application Support/LogicFlow`,
//! Linux and Raspberry Pi `~/.config/logicflow`. The file is readable text with
//! comments and can be edited by hand. Missing items get default values; an
//! unreadable file is set aside as `.bak` and the program continues with default
//! values. Writing is safe: the new content is written next to it and the file is
//! then renamed, so it is never left half-written.
//!
//! Conversion from and to text involves no files ([`Settings::from_toml`],
//! [`Settings::to_toml`]) – the web version will use it with the browser storage.

use std::path::{Path, PathBuf};

use logicflow_core::library::DEFAULT_LICENSE;
use serde::Deserialize;

use crate::i18n::Lang;
use crate::symbols::SymbolStyle;

/// Název souboru s nastavením.
///
/// [en] Name of the settings file.
pub const FILE_NAME: &str = "logicflow.toml";

/// Sdílená složka knihoven (síť, OneDrive, kroužek).
///
/// [en] A shared library folder (network, OneDrive, club).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedFolder {
    /// Cesta ke složce.
    ///
    /// [en] Path to the folder.
    pub path: PathBuf,
    /// Program do složky nezapisuje (knihovny v ní jsou jen pro čtení).
    ///
    /// [en] The program does not write into the folder (its libraries are read-only).
    pub read_only: bool,
}

/// Osobní nastavení uživatele.
///
/// [en] The user's personal settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Jazyk rozhraní.
    ///
    /// [en] Interface language.
    pub lang: Lang,
    /// Styl značek.
    ///
    /// [en] Symbol style.
    pub symbols: SymbolStyle,
    /// Povolit úpravy schématu i v režimu simulace (pro pokročilé, výchozí vypnuto).
    ///
    /// [en] Allow editing the schematic in simulation mode too (advanced, off by default).
    pub edit_while_simulating: bool,
    /// Jméno autora pro prvky knihoven a informace o obvodu (prázdné = nevyplněno).
    ///
    /// [en] Author name for library elements and circuit information (empty = not set).
    pub author: String,
    /// Výchozí licence nových prvků a obvodů.
    ///
    /// [en] Default licence of new elements and circuits.
    pub license: String,
    /// Složka osobních knihoven (`None` = `Dokumenty/LogicFlow/Knihovny`).
    ///
    /// [en] Personal libraries folder (`None` = `Documents/LogicFlow/Knihovny`).
    pub personal_libraries: Option<PathBuf>,
    /// Sdílené složky knihoven.
    ///
    /// [en] Shared library folders.
    pub shared_libraries: Vec<SharedFolder>,
}

impl Settings {
    /// Výchozí nastavení podle jazyka systému (např. „cs-CZ“, „en_US.UTF-8“):
    /// čeština (i pro slovenštinu), jinak angličtina; v americkém prostředí
    /// americké značky. Uživatel může obojí kdykoli změnit.
    ///
    /// [en] Default settings according to the system language (e.g. “cs-CZ”, “en_US.UTF-8”):
    /// Czech (also for Slovak), otherwise English; American symbols in an American
    /// environment. The user can change both at any time.
    pub fn for_locale(locale: Option<&str>) -> Self {
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
                "cs" | "sk" | "" => Lang::Cs,
                _ => Lang::En,
            },
            symbols: if region == "us" {
                SymbolStyle::Ansi
            } else {
                SymbolStyle::Iec
            },
            edit_while_simulating: false,
            author: String::new(),
            license: DEFAULT_LICENSE.to_owned(),
            personal_libraries: None,
            shared_libraries: Vec::new(),
        }
    }

    /// Načte nastavení z textu souboru. Chybějící nebo neznámé hodnoty se vezmou
    /// z `defaults`; chyba jen při neplatném zápisu TOML nebo špatném typu hodnoty.
    ///
    /// [en] Loads the settings from the file text. Missing or unknown values are taken
    /// from `defaults`; an error only for invalid TOML or a wrong value type.
    pub fn from_toml(text: &str, defaults: &Self) -> Result<Self, String> {
        let raw: Raw = toml::from_str(text).map_err(|e| e.message().to_owned())?;
        let mut s = defaults.clone();
        if let Some(l) = raw.jazyk.as_deref().and_then(lang_from_code) {
            s.lang = l;
        }
        match raw.znacky.as_deref() {
            Some("iec") => s.symbols = SymbolStyle::Iec,
            Some("ansi") => s.symbols = SymbolStyle::Ansi,
            _ => {}
        }
        if let Some(v) = raw.upravy_za_behu_simulace {
            s.edit_while_simulating = v;
        }
        if let Some(a) = raw.autor {
            s.author = a.trim().to_owned();
        }
        if let Some(l) = raw.licence.filter(|l| !l.trim().is_empty()) {
            s.license = l.trim().to_owned();
        }
        s.personal_libraries = raw
            .knihovny
            .osobni
            .filter(|p| !p.trim().is_empty())
            .map(PathBuf::from);
        s.shared_libraries = raw
            .knihovny
            .sdilene
            .into_iter()
            .filter(|f| !f.cesta.trim().is_empty())
            .map(|f| SharedFolder {
                path: PathBuf::from(f.cesta),
                read_only: f.jen_pro_cteni,
            })
            .collect();
        Ok(s)
    }

    /// Text souboru s komentáři v jazyce rozhraní.
    ///
    /// [en] File text with comments in the interface language.
    pub fn to_toml(&self) -> String {
        let cs = self.lang == Lang::Cs;
        let c = |cz: &str, en: &str| if cs { cz.to_owned() } else { en.to_owned() };
        let mut out = String::new();
        let mut line = |text: String| {
            out.push_str(&text);
            out.push('\n');
        };
        line(c(
            "# Nastavení LogicFlow – osobní nastavení uživatele.",
            "# LogicFlow settings – the user's personal settings.",
        ));
        line(c(
            "# Soubor jde upravit i ručně; chybějící nebo neplatné hodnoty program nahradí výchozími.",
            "# The file can be edited by hand; missing or invalid values are replaced by defaults.",
        ));
        line(String::new());
        line(c(
            "# Jazyk rozhraní: \"cs\" (čeština), \"en\" (angličtina)",
            "# Interface language: \"cs\" (Czech), \"en\" (English)",
        ));
        line(format!("jazyk = {}", quote(self.lang.code())));
        line(c(
            "# Styl značek: \"iec\" (evropské, ČSN EN 60617-12), \"ansi\" (americké)",
            "# Symbol style: \"iec\" (European, EN 60617-12), \"ansi\" (American)",
        ));
        let sym = match self.symbols {
            SymbolStyle::Iec => "iec",
            SymbolStyle::Ansi => "ansi",
        };
        line(format!("znacky = {}", quote(sym)));
        line(c(
            "# Povolit úpravy schématu i za běhu simulace (pro pokročilé)",
            "# Allow editing the schematic while the simulation runs (advanced)",
        ));
        line(format!(
            "upravy_za_behu_simulace = {}",
            self.edit_while_simulating
        ));
        line(c(
            "# Jméno autora – u prvků exportovaných do knihovny a v informacích o obvodu",
            "# Author name – for elements exported to a library and in circuit information",
        ));
        line(format!("autor = {}", quote(&self.author)));
        line(c(
            "# Výchozí licence nových prvků a obvodů, např. \"CC BY 4.0\", \"neuvedeno\"",
            "# Default licence of new elements and circuits, e.g. \"CC BY 4.0\", \"not specified\"",
        ));
        line(format!("licence = {}", quote(&self.license)));
        line(String::new());
        line("[knihovny]".to_owned());
        line(c(
            "# Složka osobních knihoven (prázdné = Dokumenty/LogicFlow/Knihovny)",
            "# Personal libraries folder (empty = Documents/LogicFlow/Knihovny)",
        ));
        let personal = self
            .personal_libraries
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        line(format!("osobni = {}", quote(&personal)));
        if !self.shared_libraries.is_empty() {
            line(String::new());
            line(c(
                "# Sdílené složky knihoven (síť, OneDrive, kroužek); jen_pro_cteni = program do nich nezapisuje",
                "# Shared library folders (network, OneDrive, club); jen_pro_cteni = the program does not write there",
            ));
        }
        for f in &self.shared_libraries {
            line("[[knihovny.sdilene]]".to_owned());
            line(format!("cesta = {}", quote(&f.path.display().to_string())));
            line(format!("jen_pro_cteni = {}", f.read_only));
        }
        out
    }

    /// Složka osobních knihoven: nastavená, jinak `Dokumenty/LogicFlow/Knihovny`.
    ///
    /// [en] Personal libraries folder: the configured one, otherwise `Documents/LogicFlow/Knihovny`.
    pub fn personal_folder(&self) -> Option<PathBuf> {
        self.personal_libraries
            .clone()
            .or_else(crate::libraries::default_personal_folder)
    }
}

/// Jazyk podle kódu ISO 639-1.
///
/// [en] Language by ISO 639-1 code.
fn lang_from_code(code: &str) -> Option<Lang> {
    match code.trim().to_ascii_lowercase().as_str() {
        "cs" => Some(Lang::Cs),
        "en" => Some(Lang::En),
        _ => None,
    }
}

/// Řetězec TOML v uvozovkách (základní řetězec s escapováním).
///
/// [en] A quoted TOML string (basic string with escaping).
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Podoba souboru pro serde (všechny položky nepovinné).
///
/// [en] Shape of the file for serde (all items optional).
#[derive(Deserialize, Default)]
#[serde(default)]
struct Raw {
    /// Kód jazyka.
    ///
    /// [en] Language code.
    jazyk: Option<String>,
    /// Styl značek („iec“, „ansi“).
    ///
    /// [en] Symbol style ("iec", "ansi").
    znacky: Option<String>,
    /// Úpravy za běhu simulace.
    ///
    /// [en] Editing while simulating.
    upravy_za_behu_simulace: Option<bool>,
    /// Jméno autora.
    ///
    /// [en] Author name.
    autor: Option<String>,
    /// Výchozí licence.
    ///
    /// [en] Default licence.
    licence: Option<String>,
    /// Knihovny.
    ///
    /// [en] Libraries.
    knihovny: RawLibraries,
}

/// Část `[knihovny]`.
///
/// [en] The `[knihovny]` section.
#[derive(Deserialize, Default)]
#[serde(default)]
struct RawLibraries {
    /// Složka osobních knihoven.
    ///
    /// [en] Personal libraries folder.
    osobni: Option<String>,
    /// Sdílené složky.
    ///
    /// [en] Shared folders.
    sdilene: Vec<RawShared>,
}

/// Jedna sdílená složka.
///
/// [en] One shared folder.
#[derive(Deserialize)]
struct RawShared {
    /// Cesta.
    ///
    /// [en] Path.
    cesta: String,
    /// Jen pro čtení (výchozí ano – bezpečnější pro síťové složky).
    ///
    /// [en] Read-only (default yes – safer for network folders).
    #[serde(default = "yes")]
    jen_pro_cteni: bool,
}

/// Výchozí hodnota `true` pro serde.
///
/// [en] Default value `true` for serde.
fn yes() -> bool {
    true
}

/// Složka se souborem s nastavením podle systému.
///
/// [en] Folder with the settings file according to the system.
pub fn folder() -> Option<PathBuf> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let name = "LogicFlow";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let name = "logicflow";
    dirs::config_dir().map(|d| d.join(name))
}

/// Výsledek načtení souboru s nastavením.
///
/// [en] Result of loading the settings file.
#[derive(Debug, PartialEq, Eq)]
pub enum Loaded {
    /// Soubor se načetl.
    ///
    /// [en] The file was loaded.
    Ok(Settings),
    /// Soubor zatím neexistuje (první spuštění).
    ///
    /// [en] The file does not exist yet (first run).
    Missing,
    /// Soubor nešel přečíst – odložil se jako `.bak` (je-li to možné); důvod.
    ///
    /// [en] The file could not be read – it was set aside as `.bak` (if possible); the reason.
    Broken(String),
}

/// Načte nastavení ze souboru; nečitelný soubor odloží jako `.bak`.
///
/// [en] Loads the settings from the file; an unreadable file is set aside as `.bak`.
pub fn load(path: &Path, defaults: &Settings) -> Loaded {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Loaded::Missing,
        Err(e) => return Loaded::Broken(e.to_string()),
    };
    match Settings::from_toml(&text, defaults) {
        Ok(s) => Loaded::Ok(s),
        Err(e) => {
            let _ = std::fs::rename(path, path.with_extension("toml.bak"));
            Loaded::Broken(e)
        }
    }
}

/// Bezpečně uloží nastavení: zapíše dočasný soubor a přejmenuje ho na `path`.
///
/// [en] Saves the settings safely: writes a temporary file and renames it to `path`.
pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, settings.to_toml()).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nastavení s vyplněnými všemi položkami.
    ///
    /// [en] Settings with all items filled in.
    fn full() -> Settings {
        Settings {
            lang: Lang::En,
            symbols: SymbolStyle::Ansi,
            edit_while_simulating: true,
            author: "Jan \"Honza\" Novák".to_owned(),
            license: "CC BY-SA 4.0".to_owned(),
            personal_libraries: Some(PathBuf::from("H:\\LogicFlow\\Knihovny")),
            shared_libraries: vec![
                SharedFolder {
                    path: PathBuf::from("\\\\server\\vyuka\\knihovny"),
                    read_only: true,
                },
                SharedFolder {
                    path: PathBuf::from("/home/jan/OneDrive/krouzek"),
                    read_only: false,
                },
            ],
        }
    }

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
        // Neznámé prostředí: čeština a evropské značky; licence CC BY 4.0.
        // [en] Unknown environment: Czech and European symbols; licence CC BY 4.0.
        let s = Settings::for_locale(None);
        assert_eq!((s.lang, s.symbols), (Lang::Cs, SymbolStyle::Iec));
        assert_eq!(s.license, DEFAULT_LICENSE);
    }

    #[test]
    fn round_trip_in_both_languages() {
        let defaults = Settings::for_locale(Some("cs-CZ"));
        let mut s = full();
        for lang in [Lang::En, Lang::Cs] {
            s.lang = lang;
            let text = s.to_toml();
            assert_eq!(Settings::from_toml(&text, &defaults).unwrap(), s, "{text}");
        }
        // Komentáře v jazyce rozhraní.
        // [en] Comments in the interface language.
        assert!(s.to_toml().contains("# Jazyk rozhraní"));
        s.lang = Lang::En;
        assert!(s.to_toml().contains("# Interface language"));
    }

    #[test]
    fn missing_and_unknown_values_use_defaults() {
        let defaults = Settings::for_locale(Some("en-US"));
        let s = Settings::from_toml("", &defaults).unwrap();
        assert_eq!(s, defaults);
        let s = Settings::from_toml(
            "jazyk = \"de\"\nznacky = \"ansi\"\nautor = \"  Eva \"\nlicence = \"\"\n[knihovny]\nosobni = \"\"\n[[knihovny.sdilene]]\ncesta = \"/sit/knihovny\"\n",
            &defaults,
        )
        .unwrap();
        assert_eq!(
            s.lang,
            Lang::En,
            "neznámý jazyk → výchozí / unknown language → default"
        );
        assert_eq!(s.author, "Eva");
        assert_eq!(s.license, DEFAULT_LICENSE);
        assert_eq!(s.personal_libraries, None);
        // Sdílená složka bez jen_pro_cteni je jen pro čtení.
        // [en] A shared folder without jen_pro_cteni is read-only.
        assert_eq!(
            s.shared_libraries,
            [SharedFolder {
                path: PathBuf::from("/sit/knihovny"),
                read_only: true
            }]
        );
        assert!(Settings::from_toml("jazyk = ", &defaults).is_err());
        assert!(Settings::from_toml("upravy_za_behu_simulace = \"ano\"", &defaults).is_err());
    }

    #[test]
    fn file_save_load_and_backup() {
        let dir = std::env::temp_dir().join(format!("logicflow-nastaveni-{}", std::process::id()));
        let path = dir.join("podslozka").join(FILE_NAME);
        let defaults = Settings::for_locale(Some("cs-CZ"));
        assert_eq!(load(&path, &defaults), Loaded::Missing);
        // Uložení založí složku, nezůstane dočasný soubor.
        // [en] Saving creates the folder, no temporary file is left behind.
        save(&path, &full()).unwrap();
        assert!(!path.with_extension("toml.tmp").exists());
        assert_eq!(load(&path, &defaults), Loaded::Ok(full()));
        // Přepsání existujícího souboru.
        // [en] Overwriting an existing file.
        save(&path, &defaults).unwrap();
        assert_eq!(load(&path, &defaults), Loaded::Ok(defaults.clone()));
        // Vadný soubor se odloží jako .bak.
        // [en] A broken file is set aside as .bak.
        std::fs::write(&path, "jazyk = [").unwrap();
        assert!(matches!(load(&path, &defaults), Loaded::Broken(_)));
        assert!(!path.exists());
        assert_eq!(
            std::fs::read_to_string(path.with_extension("toml.bak")).unwrap(),
            "jazyk = ["
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn quoting() {
        assert_eq!(quote("a\"b\\c"), "\"a\\\"b\\\\c\"");
        let text = format!("autor = {}", quote("řádek\nza\ttab \u{1}"));
        let s = Settings::from_toml(&text, &Settings::for_locale(None)).unwrap();
        assert_eq!(s.author, "řádek\nza\ttab \u{1}".trim());
    }
}
