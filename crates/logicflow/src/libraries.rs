//! Knihovny prvků dostupné v programu: vestavěné knihovny (výuková a vnitřní
//! zapojení funkčních bloků), osobní
//! knihovny (výchozí složka `Dokumenty/LogicFlow/Knihovny`) a sdílené složky
//! ze souboru s nastavením.
//!
//! Toto je jediné místo, kde program čte knihovny ze souborů (viz CONTRIBUTING →
//! Přenositelnost); zbytek programu pracuje s načtenými [`Library`].
//!
//! [en] Element libraries available in the program: the built-in libraries (teaching
//! and internal circuits of function blocks), personal libraries (default folder `Documents/LogicFlow/Knihovny`) and shared
//! folders from the settings file.
//!
//! This is the only place where the program reads libraries from files (see
//! CONTRIBUTING → Portability); the rest of the program works with the loaded
//! [`Library`] values.

use std::path::{Path, PathBuf};

use std::sync::OnceLock;

use logicflow_core::library::{CATEGORIES, Library, LibraryError, LibraryItem, builtin_category};
use logicflow_core::{BlockKind, SubcircuitDef};

use crate::i18n::{Lang, T, tr};
use crate::settings::{Settings, SharedFolder};

/// Vestavěná výuková knihovna „Číslicová technika“ (vložená při sestavení).
///
/// [en] The built-in teaching library “Digital electronics” (embedded at build time).
const BUILTIN: &str = include_str!("../../../docs/knihovny/cislicova-technika.lflib");

/// Vestavěná knihovna „Funkční bloky – vnitřní zapojení“: pro každý funkční blok
/// zapojení z hradel (otevře ho Tab na bloku).
///
/// [en] The built-in library “Function blocks – internal circuits”: a gate-level
/// circuit for every function block (opened by Tab on a block).
const BUILTIN_BLOCKS: &str = include_str!("../../../docs/knihovny/funkcni-bloky.lflib");

/// Podsložka osobních knihoven ve složce dokumentů.
///
/// [en] Subfolder of personal libraries in the documents folder.
const PERSONAL_SUBDIR: [&str; 2] = ["LogicFlow", "Knihovny"];

/// Odkud knihovna pochází.
///
/// [en] Where a library comes from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// Vestavěná v programu, jen pro čtení.
    ///
    /// [en] Built into the program, read-only.
    Builtin,
    /// Soubor v osobní složce knihoven.
    ///
    /// [en] A file in the personal libraries folder.
    Personal(PathBuf),
    /// Soubor ve sdílené složce knihoven.
    ///
    /// [en] A file in a shared library folder.
    Shared {
        /// Cesta k souboru.
        ///
        /// [en] Path to the file.
        file: PathBuf,
        /// Složka je jen pro čtení.
        ///
        /// [en] The folder is read-only.
        read_only: bool,
    },
}

/// Načtená knihovna a její zdroj.
///
/// [en] A loaded library and its source.
#[derive(Clone, Debug)]
pub struct Loaded {
    /// Knihovna.
    ///
    /// [en] The library.
    pub library: Library,
    /// Zdroj knihovny.
    ///
    /// [en] Source of the library.
    pub source: Source,
}

/// Důvod, proč se soubor knihovny nenačetl.
///
/// [en] Reason why a library file did not load.
#[derive(Clone, Debug)]
pub enum Problem {
    /// Soubor nejde přečíst.
    ///
    /// [en] The file cannot be read.
    Io(String),
    /// Obsah není platná knihovna.
    ///
    /// [en] The content is not a valid library.
    Library(LibraryError),
    /// Sdílená složka není dostupná (síť, odpojený disk).
    ///
    /// [en] A shared folder is not available (network, disconnected drive).
    Unavailable,
}

impl Problem {
    /// Text pro uživatele.
    ///
    /// [en] Text for the user.
    pub fn text(&self, lang: Lang) -> String {
        match self {
            Self::Io(e) => e.clone(),
            Self::Library(e) => e.text(lang.code()),
            Self::Unavailable => tr(lang, T::LibUnavailable).to_owned(),
        }
    }
}

/// Všechny knihovny dostupné v programu.
///
/// [en] All libraries available in the program.
#[derive(Clone, Debug, Default)]
pub struct Libraries {
    /// Načtené knihovny: nejdřív vestavěné, pak osobní podle názvu souboru.
    ///
    /// [en] Loaded libraries: the built-in ones first, then personal ones by file name.
    pub list: Vec<Loaded>,
    /// Soubory, které se nepodařilo načíst: název souboru a důvod.
    ///
    /// [en] Files that could not be loaded: file name and reason.
    pub problems: Vec<(String, Problem)>,
    /// Složka osobních knihoven (`None`, když nejde zjistit složku dokumentů).
    ///
    /// [en] Personal libraries folder (`None` when the documents folder cannot be found).
    pub folder: Option<PathBuf>,
    /// Sdílené složky knihoven z nastavení.
    ///
    /// [en] Shared library folders from the settings.
    pub shared: Vec<SharedFolder>,
}

impl Libraries {
    /// Načte vestavěnou knihovnu a knihovny ze složek podle nastavení.
    ///
    /// [en] Loads the built-in library and the libraries from the folders in the settings.
    pub fn load(settings: &Settings) -> Self {
        let mut libs = Self::default();
        libs.configure(settings);
        libs
    }

    /// Převezme složky z nastavení a knihovny načte znovu.
    ///
    /// [en] Takes the folders from the settings and reloads the libraries.
    pub fn configure(&mut self, settings: &Settings) {
        self.folder = settings.personal_folder();
        self.shared = settings.shared_libraries.clone();
        self.reload();
    }

    /// Načte knihovny znovu (po změně souborů ve složkách).
    ///
    /// [en] Reloads the libraries (after the files in the folders changed).
    pub fn reload(&mut self) {
        self.list = builtins();
        self.problems.clear();
        if let Some(dir) = &self.folder {
            let (list, problems) = read_folder(dir, Source::Personal);
            self.list.extend(list);
            self.problems.extend(problems);
        }
        for shared in &self.shared {
            if !shared.path.is_dir() {
                self.problems
                    .push((shared.path.display().to_string(), Problem::Unavailable));
                continue;
            }
            let read_only = shared.read_only;
            let (list, problems) =
                read_folder(&shared.path, |file| Source::Shared { file, read_only });
            self.list.extend(list);
            self.problems.extend(problems);
        }
    }
}

/// Vestavěné knihovny: výuková a vnitřní zapojení funkčních bloků.
///
/// [en] The built-in libraries: the teaching one and the internal circuits of function blocks.
pub fn builtins() -> Vec<Loaded> {
    vec![
        Loaded {
            library: Library::from_json(BUILTIN).expect("vestavěná knihovna je platná"),
            source: Source::Builtin,
        },
        Loaded {
            library: block_library().clone(),
            source: Source::Builtin,
        },
    ]
}

/// Knihovna vnitřních zapojení funkčních bloků (načte se jednou).
///
/// [en] The library of internal circuits of function blocks (loaded once).
pub fn block_library() -> &'static Library {
    static LIB: OnceLock<Library> = OnceLock::new();
    LIB.get_or_init(|| Library::from_json(BUILTIN_BLOCKS).expect("knihovna bloků je platná"))
}

/// Vnitřní zapojení bloku: definice podobvodů (prvek bloku a vše, co používá,
/// s názvy v jazyce `lang`) a číslo definice prvku bloku. `None`, když blok
/// v knihovně chybí.
///
/// [en] Internal circuit of a block: subcircuit definitions (the block's item and
/// everything it uses, with names in the language `lang`) and the number of the
/// block item's definition. `None` when the block is missing in the library.
pub fn block_equivalent(block: BlockKind, lang: Lang) -> Option<(Vec<SubcircuitDef>, u32)> {
    let lib = block_library();
    let uid = block.library_uid();
    let item = lib.item(&uid)?;
    let mut defs = lib.item_with_dependencies(&uid)?;
    for d in &mut defs {
        if let Some(i) = lib.items.iter().find(|i| i.id == d.id) {
            d.name = i.display_name(lang.code()).to_owned();
        }
    }
    Some((defs, item.id))
}

/// Text pro vložení vnitřního zapojení bloku do projektu jako upravitelného
/// podobvodu (stejně jako prvek z panelu knihoven): identita prvku a text.
///
/// [en] Text for inserting the internal circuit of a block into the project as an
/// editable subcircuit (like an item from the library panel): item identity and text.
pub fn block_insertion(block: BlockKind, lang: Lang) -> Option<(String, String)> {
    let uid = block.library_uid();
    let text = block_library().insertion_text(&uid, lang.code())?;
    Some((uid, text))
}

/// Načte všechny soubory `.lflib` ve složce (seřazené podle názvu); `source` určí
/// zdroj podle cesty k souboru. Chybějící složka není chyba – osobní knihovny
/// zatím nejsou.
///
/// [en] Loads all `.lflib` files in the folder (sorted by name); `source` gives the
/// source from the file path. A missing folder is not an error – there are no
/// personal libraries yet.
pub fn read_folder(
    dir: &Path,
    source: impl Fn(PathBuf) -> Source,
) -> (Vec<Loaded>, Vec<(String, Problem)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (Vec::new(), Vec::new());
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case(Library::EXTENSION))
        })
        .collect();
    files.sort();
    let mut list = Vec::new();
    let mut problems = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match std::fs::read_to_string(&path) {
            Ok(text) => match Library::from_json(&text) {
                Ok(library) => list.push(Loaded {
                    library,
                    source: source(path),
                }),
                Err(e) => problems.push((name, Problem::Library(e))),
            },
            Err(e) => problems.push((name, Problem::Io(e.to_string()))),
        }
    }
    (list, problems)
}

/// Složka dokumentů uživatele podle systému – ve Windows přes známé složky (funguje
/// i přesměrování Dokumentů na síť), v Linuxu podle `user-dirs.dirs` (např.
/// `~/Dokumenty`); záložně `Documents` v domovské složce.
///
/// [en] The user's documents folder according to the system – on Windows via known
/// folders (redirecting Documents to the network works too), on Linux according to
/// `user-dirs.dirs` (e.g. `~/Dokumenty`); falling back to `Documents` in the home folder.
pub fn documents_dir() -> Option<PathBuf> {
    dirs::document_dir().or_else(|| std::env::home_dir().map(|h| h.join("Documents")))
}

/// Výchozí složka osobních knihoven: `Dokumenty/LogicFlow/Knihovny`.
///
/// [en] Default personal libraries folder: `Documents/LogicFlow/Knihovny`.
pub fn default_personal_folder() -> Option<PathBuf> {
    documents_dir().map(|d| PERSONAL_SUBDIR.iter().fold(d, |p, s| p.join(s)))
}

/// Vytvoří složku (pokud chybí) a otevře ji ve správci souborů systému.
///
/// [en] Creates the folder (if missing) and opens it in the system file manager.
pub fn open_folder(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    let program = "explorer";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let program = "xdg-open";
    // Spustí se na pozadí, program na správce souborů nečeká.
    // [en] Started in the background, the program does not wait for the file manager.
    std::process::Command::new(program)
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Klíč požadavku na otevření okna Nastavení (z panelu knihoven).
///
/// [en] Key of the request to open the Settings window (from the library panel).
fn settings_request_id() -> egui::Id {
    egui::Id::unique("logicflow-open-settings")
}

/// Požádá hlavní okno o otevření Nastavení (tlačítko Složky knihoven… v panelu).
///
/// [en] Asks the main window to open the Settings (the Library folders… button in the panel).
pub fn request_settings(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(settings_request_id(), true));
}

/// Vyzvedne požadavek na otevření Nastavení.
///
/// [en] Takes the request to open the Settings.
pub fn take_settings_request(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(settings_request_id()))
        .unwrap_or(false)
}

/// Prvky knihovny seskupené podle kategorie: vestavěné kategorie v pořadí
/// [`CATEGORIES`], vlastní abecedně za nimi. Název skupiny je v jazyce `lang`.
///
/// [en] Library items grouped by category: built-in categories in the order of
/// [`CATEGORIES`], custom ones alphabetically after them. The group name is in the
/// language `lang`.
pub fn by_category(library: &Library, lang: Lang) -> Vec<(String, Vec<&LibraryItem>)> {
    let rank = |cat: &str| {
        builtin_category(cat).map_or(CATEGORIES.len(), |c| {
            CATEGORIES.iter().position(|k| k.key == c.key).unwrap_or(0)
        })
    };
    let mut groups: Vec<(usize, String, Vec<&LibraryItem>)> = Vec::new();
    for item in &library.items {
        let name = item.category_name(lang.code());
        match groups.iter_mut().find(|g| g.1 == name) {
            Some(g) => g.2.push(item),
            None => groups.push((rank(&item.category), name, vec![item])),
        }
    }
    groups.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    groups.into_iter().map(|(_, n, items)| (n, items)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_library_loads() {
        let b = builtins().swap_remove(0);
        assert_eq!(b.source, Source::Builtin);
        assert!(b.library.items.len() >= 30);
        // Skupiny v pořadí vestavěných kategorií, česky i anglicky.
        // [en] Groups in the order of the built-in categories, in Czech and English.
        let cs: Vec<String> = by_category(&b.library, Lang::Cs)
            .into_iter()
            .map(|g| g.0)
            .collect();
        assert_eq!(
            cs,
            [
                "Kombinační obvody",
                "Aritmetika",
                "Převodníky kódů",
                "Sekvenční obvody",
                "Aplikace"
            ]
        );
        let en = by_category(&b.library, Lang::En);
        assert_eq!(en[0].0, "Combinational circuits");
        assert_eq!(en[0].1[0].display_name("en"), "NOT from NAND");
    }

    #[test]
    fn every_block_has_an_equivalent() {
        for block in BlockKind::variants() {
            let (defs, root) = block_equivalent(block, Lang::Cs).expect("ekvivalent bloku");
            let def = defs.iter().find(|d| d.id == root).unwrap();
            // Stejné názvy vývodů jako blok (pořadí podle popisků).
            // [en] The same pin names as the block (order by labels).
            let (ins, outs) = def.pin_names();
            let mut want = block.input_names();
            want.sort();
            let mut got = ins.clone();
            got.sort();
            assert_eq!(got, want, "{block:?}");
            assert_eq!(outs.len(), block.output_names().len(), "{block:?}");
            // Název jako v Knihovně součástek (čítač navíc se směrem).
            // [en] The name as in the component library (the counter with its direction too).
            for lang in [Lang::Cs, Lang::En] {
                let (defs, root) = block_equivalent(block, lang).unwrap();
                let name = &defs.iter().find(|d| d.id == root).unwrap().name;
                assert!(
                    name.starts_with(&crate::catalog::block_name(block, lang)),
                    "{name}"
                );
            }
            // Všechny použité podobvody jsou mezi definicemi.
            // [en] All used subcircuits are among the definitions.
            for d in &defs {
                for c in &d.circuit.components {
                    if let logicflow_core::ComponentKind::Subcircuit { id, .. } = c.kind {
                        assert!(defs.iter().any(|x| x.id == id), "{block:?}");
                    }
                }
            }
        }
        let (defs, root) = block_equivalent(
            BlockKind::Counter {
                bits: 4,
                down: false,
            },
            Lang::En,
        )
        .unwrap();
        let name = |id: u32| defs.iter().find(|d| d.id == id).unwrap().name.clone();
        assert_eq!(name(root), "Counter 4-bit up");
        assert_eq!(defs.len(), 3);
        assert!(block_insertion(BlockKind::BcdTo7Seg, Lang::Cs).is_some());
    }

    #[test]
    fn folder_with_good_and_bad_files() {
        let dir = std::env::temp_dir().join(format!("logicflow-knihovny-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let vzor = include_str!("../../../docs/knihovny/vzor.lflib");
        std::fs::write(dir.join("b-vzor.lflib"), vzor).unwrap();
        std::fs::write(dir.join("a-vadna.LFLIB"), "{ nic }").unwrap();
        std::fs::write(dir.join("poznamky.txt"), "jiný soubor").unwrap();
        let (list, problems) = read_folder(&dir, Source::Personal);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].library.items.len(), 2);
        assert!(matches!(&list[0].source, Source::Personal(p) if p.ends_with("b-vzor.lflib")));
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].0, "a-vadna.LFLIB");
        assert!(!problems[0].1.text(Lang::Cs).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
        // Chybějící složka: žádné knihovny ani chyby.
        // [en] Missing folder: no libraries and no errors.
        let (list, problems) = read_folder(&dir, Source::Personal);
        assert!(list.is_empty() && problems.is_empty());
    }

    #[test]
    fn shared_folders_from_settings() {
        let base = std::env::temp_dir().join(format!("logicflow-sdilene-{}", std::process::id()));
        let shared = base.join("sdilena");
        std::fs::create_dir_all(&shared).unwrap();
        let vzor = include_str!("../../../docs/knihovny/vzor.lflib");
        std::fs::write(shared.join("vzor.lflib"), vzor).unwrap();
        let mut settings = Settings::for_locale(Some("cs-CZ"));
        settings.personal_libraries = Some(base.join("osobni-neexistuje"));
        settings.shared_libraries = vec![
            SharedFolder {
                path: shared.clone(),
                read_only: true,
            },
            SharedFolder {
                path: base.join("odpojena"),
                read_only: false,
            },
        ];
        let libs = Libraries::load(&settings);
        // Dvě vestavěné + jedna ze sdílené složky; nedostupná složka je problém.
        // [en] Two built-in + one from the shared folder; an unavailable folder is a problem.
        assert_eq!(libs.list.len(), 3);
        assert!(matches!(
            &libs.list[2].source,
            Source::Shared { read_only: true, file } if file.ends_with("vzor.lflib")
        ));
        assert_eq!(libs.problems.len(), 1);
        assert!(matches!(libs.problems[0].1, Problem::Unavailable));
        assert_eq!(libs.folder, Some(base.join("osobni-neexistuje")));
        std::fs::remove_dir_all(&base).unwrap();
    }
}
