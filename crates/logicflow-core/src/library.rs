//! Knihovny prvků (soubor `.lflib`).
//!
//! Knihovna je pojmenovaný soubor hotových podobvodů – **prvků** – které jde
//! vložit do libovolného projektu. Formát je stejný JSON jako projekt `.logicflow`,
//! jen místo hlavního obvodu má hlavičku knihovny a seznam prvků. Každý prvek
//! má stabilní identitu (`uid`, UUID), která se nemění při přejmenování ani úpravě,
//! a číslo `id`, na které odkazují instance prvku uvnitř jiných prvků téže knihovny.
//!
//! Jádro jen převádí knihovnu z textu a do textu a kontroluje ji; soubory, datum
//! a náhodná čísla pro nové UUID dodává aplikace (kvůli webové verzi).
//! Popis formátu a vzorový soubor jsou v `docs/knihovny`.
//!
//! [en]
//!
//! Element libraries (the `.lflib` file).
//!
//! A library is a named file of ready-made subcircuits – **items** – that can be
//! inserted into any project. The format is the same JSON as the `.logicflow`
//! project, only instead of the main circuit it has a library header and a list
//! of items. Every item has a stable identity (`uid`, a UUID) that does not change
//! on renaming or editing, and a number `id` that instances of the item inside
//! other items of the same library refer to.
//!
//! The core only converts the library from and to text and checks it; files, the
//! date and random numbers for new UUIDs are provided by the application (because
//! of the web version). The format description and a sample file are in `docs/knihovny`.

use std::collections::{BTreeMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::circuit::{Circuit, ComponentKind, Point, ProjectFile, SubcircuitDef, sync_instances};

/// Výchozí licence nových prvků a knihoven (stejná jako v online knihovně obvodů).
///
/// [en] Default licence of new items and libraries (the same as in the online circuit library).
pub const DEFAULT_LICENSE: &str = "CC BY 4.0";

/// Překlad názvu a popisu do dalšího jazyka.
///
/// [en] Translation of the name and description into another language.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Translation {
    /// Přeložený název (prázdný = použije se hlavní).
    ///
    /// [en] Translated name (empty = the main one is used).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Přeložený popis (prázdný = použije se hlavní).
    ///
    /// [en] Translated description (empty = the main one is used).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// Překlady podle kódu jazyka ISO 639-1 (`"en"`, `"cs"`, `"uk"` …).
///
/// [en] Translations by ISO 639-1 language code (`"en"`, `"cs"`, `"uk"` …).
pub type Translations = BTreeMap<String, Translation>;

/// Název v daném jazyce: překlad, pokud existuje a není prázdný, jinak hlavní text.
///
/// [en] Name in the given language: the translation if it exists and is not empty,
/// otherwise the main text.
fn localized<'a>(
    main: &'a str,
    translations: &'a Translations,
    lang: &str,
    field: fn(&Translation) -> &str,
) -> &'a str {
    translations
        .get(lang)
        .map(field)
        .filter(|t| !t.is_empty())
        .unwrap_or(main)
}

/// Výchozí verze nového prvku nebo knihovny (pro serde).
///
/// [en] Default version of a new item or library (for serde).
fn first_version() -> u32 {
    1
}

/// Hlavička knihovny.
///
/// [en] Library header.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryInfo {
    /// Stabilní identita knihovny (UUID).
    ///
    /// [en] Stable identity of the library (UUID).
    pub uid: String,
    /// Název knihovny v jazyce autora.
    ///
    /// [en] Library name in the author's language.
    pub name: String,
    /// Popis knihovny.
    ///
    /// [en] Library description.
    #[serde(default)]
    pub description: String,
    /// Překlady názvu a popisu.
    ///
    /// [en] Translations of the name and description.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub translations: Translations,
    /// Autor (správce) knihovny.
    ///
    /// [en] Author (maintainer) of the library.
    #[serde(default)]
    pub author: String,
    /// Licence knihovny, např. „CC BY 4.0“.
    ///
    /// [en] Library licence, e.g. “CC BY 4.0”.
    #[serde(default)]
    pub license: String,
    /// Verze knihovny – zvyšuje se při každém uložení.
    ///
    /// [en] Library version – increased on every save.
    #[serde(default = "first_version")]
    pub version: u32,
    /// Datum vytvoření (RRRR-MM-DD).
    ///
    /// [en] Creation date (YYYY-MM-DD).
    #[serde(default)]
    pub created: String,
    /// Datum poslední úpravy (RRRR-MM-DD).
    ///
    /// [en] Date of the last change (YYYY-MM-DD).
    #[serde(default)]
    pub modified: String,
}

impl LibraryInfo {
    /// Nová hlavička: verze 1, licence CC BY 4.0, datum vytvoření i úpravy `today`.
    ///
    /// [en] New header: version 1, licence CC BY 4.0, creation and modification date `today`.
    pub fn new(uid: String, name: &str, author: &str, today: &str) -> Self {
        Self {
            uid,
            name: name.to_owned(),
            description: String::new(),
            translations: Translations::new(),
            author: author.to_owned(),
            license: DEFAULT_LICENSE.to_owned(),
            version: 1,
            created: today.to_owned(),
            modified: today.to_owned(),
        }
    }

    /// Název knihovny v jazyce `lang`.
    ///
    /// [en] Library name in the language `lang`.
    pub fn display_name(&self, lang: &str) -> &str {
        localized(&self.name, &self.translations, lang, |t| &t.name)
    }

    /// Popis knihovny v jazyce `lang`.
    ///
    /// [en] Library description in the language `lang`.
    pub fn display_description(&self, lang: &str) -> &str {
        localized(&self.description, &self.translations, lang, |t| {
            &t.description
        })
    }
}

/// Prvek knihovny: podobvod s popisem, autorem, licencí a verzí.
///
/// [en] Library item: a subcircuit with a description, author, licence and version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryItem {
    /// Stabilní identita prvku (UUID) – nemění se při přejmenování ani úpravě.
    ///
    /// [en] Stable identity of the item (UUID) – does not change on renaming or editing.
    pub uid: String,
    /// Číslo prvku v souboru; na něj odkazují instance uvnitř jiných prvků knihovny.
    ///
    /// [en] Item number within the file; instances inside other items of the library refer to it.
    pub id: u32,
    /// Název na značce v jazyce autora.
    ///
    /// [en] Name on the symbol in the author's language.
    pub name: String,
    /// Kategorie v panelu: klíč vestavěné kategorie (viz [`CATEGORIES`]) nebo vlastní text.
    ///
    /// [en] Category in the panel: key of a built-in category (see [`CATEGORIES`]) or custom text.
    #[serde(default)]
    pub category: String,
    /// Popis prvku.
    ///
    /// [en] Item description.
    #[serde(default)]
    pub description: String,
    /// Překlady názvu a popisu.
    ///
    /// [en] Translations of the name and description.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub translations: Translations,
    /// Autor prvku.
    ///
    /// [en] Author of the item.
    #[serde(default)]
    pub author: String,
    /// Licence prvku.
    ///
    /// [en] Licence of the item.
    #[serde(default)]
    pub license: String,
    /// Verze prvku – zvyšuje se při každé úpravě.
    ///
    /// [en] Item version – increased on every change.
    #[serde(default = "first_version")]
    pub version: u32,
    /// Datum vytvoření (RRRR-MM-DD).
    ///
    /// [en] Creation date (YYYY-MM-DD).
    #[serde(default)]
    pub created: String,
    /// Datum poslední úpravy (RRRR-MM-DD).
    ///
    /// [en] Date of the last change (YYYY-MM-DD).
    #[serde(default)]
    pub modified: String,
    /// Názvy vstupů – jen pro čtenáře souboru a web knihovny; při uložení se
    /// přepočítají ze zapojení, při načtení se nepoužijí.
    ///
    /// [en] Input names – only for readers of the file and the library website; they
    /// are recomputed from the circuit on saving and not used on loading.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Názvy výstupů (stejně jako `inputs` jen informativní).
    ///
    /// [en] Output names (informative only, like `inputs`).
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Zapojení prvku (stejná struktura jako obvod v `.logicflow`).
    ///
    /// [en] The item's circuit (the same structure as a circuit in `.logicflow`).
    pub circuit: Circuit,
}

impl LibraryItem {
    /// Definice podobvodu pro simulaci a vložení (id a název jako v knihovně).
    ///
    /// [en] Subcircuit definition for simulation and insertion (id and name as in the library).
    pub fn def(&self) -> SubcircuitDef {
        SubcircuitDef {
            id: self.id,
            name: self.name.clone(),
            circuit: self.circuit.clone(),
        }
    }

    /// Název prvku v jazyce `lang`.
    ///
    /// [en] Item name in the language `lang`.
    pub fn display_name(&self, lang: &str) -> &str {
        localized(&self.name, &self.translations, lang, |t| &t.name)
    }

    /// Popis prvku v jazyce `lang`.
    ///
    /// [en] Item description in the language `lang`.
    pub fn display_description(&self, lang: &str) -> &str {
        localized(&self.description, &self.translations, lang, |t| {
            &t.description
        })
    }

    /// Název kategorie v jazyce `lang` (vlastní kategorie beze změny).
    ///
    /// [en] Category name in the language `lang` (custom categories unchanged).
    pub fn category_name(&self, lang: &str) -> String {
        category_name(&self.category, lang)
    }

    /// Zaznamená úpravu prvku: zvýší verzi a nastaví datum úpravy.
    ///
    /// [en] Records a change of the item: increases the version and sets the modification date.
    pub fn mark_modified(&mut self, today: &str) {
        self.version += 1;
        self.modified = today.to_owned();
    }
}

/// Vestavěná kategorie prvků.
///
/// [en] Built-in item category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Category {
    /// Klíč uložený v souboru (nezávislý na jazyce).
    ///
    /// [en] Key stored in the file (language independent).
    pub key: &'static str,
    /// Český název.
    ///
    /// [en] Czech name.
    pub cs: &'static str,
    /// Anglický název.
    ///
    /// [en] English name.
    pub en: &'static str,
}

/// Vestavěné kategorie v pořadí, v jakém je nabízí panel. Uživatel smí napsat i vlastní.
///
/// [en] Built-in categories in the order the panel offers them. The user may also type a custom one.
pub const CATEGORIES: &[Category] = &[
    Category {
        key: "combinational",
        cs: "Kombinační obvody",
        en: "Combinational circuits",
    },
    Category {
        key: "arithmetic",
        cs: "Aritmetika",
        en: "Arithmetic",
    },
    Category {
        key: "code-converters",
        cs: "Převodníky kódů",
        en: "Code converters",
    },
    Category {
        key: "sequential",
        cs: "Sekvenční obvody",
        en: "Sequential circuits",
    },
    Category {
        key: "memory",
        cs: "Paměti",
        en: "Memories",
    },
    Category {
        key: "applications",
        cs: "Aplikace",
        en: "Applications",
    },
    Category {
        key: "other",
        cs: "Ostatní",
        en: "Other",
    },
];

/// Vestavěná kategorie podle klíče nebo českého či anglického názvu (bez ohledu na
/// velikost písmen a mezery okolo).
///
/// [en] Built-in category by its key or Czech or English name (ignoring letter case
/// and surrounding spaces).
pub fn builtin_category(text: &str) -> Option<&'static Category> {
    let t = text.trim().to_lowercase();
    CATEGORIES
        .iter()
        .find(|c| c.key == t || c.cs.to_lowercase() == t || c.en.to_lowercase() == t)
}

/// Kategorie pro uložení: u vestavěné její klíč, vlastní text jen bez mezer okolo.
///
/// [en] Category for saving: the key of a built-in one, custom text only trimmed.
pub fn normalize_category(text: &str) -> String {
    builtin_category(text).map_or_else(|| text.trim().to_owned(), |c| c.key.to_owned())
}

/// Název kategorie v jazyce `lang` (`"cs"` česky, jinak anglicky); vlastní beze změny.
///
/// [en] Category name in the language `lang` (`"cs"` Czech, otherwise English); custom unchanged.
pub fn category_name(text: &str, lang: &str) -> String {
    match builtin_category(text) {
        Some(c) if lang == "cs" => c.cs.to_owned(),
        Some(c) => c.en.to_owned(),
        None => text.trim().to_owned(),
    }
}

/// Je text UUID ve tvaru `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (šestnáctkové číslice)?
///
/// [en] Is the text a UUID in the form `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (hex digits)?
pub fn is_uid(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(p, n)| p.len() == n && p.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Nové UUID verze 4 z 16 náhodných bajtů. Náhodu dodává aplikace (jádro žádný
/// zdroj náhody nemá, aby šlo přeložit i pro WebAssembly).
///
/// [en] A new version 4 UUID from 16 random bytes. The application provides the
/// randomness (the core has no source of randomness so that it compiles for WebAssembly).
pub fn uid_from_random(mut bytes: [u8; 16]) -> String {
    // Verze 4 a varianta RFC 4122.
    // [en] Version 4 and the RFC 4122 variant.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Chyba při načtení nebo kontrole knihovny.
///
/// [en] Error when loading or checking a library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LibraryError {
    /// Text není platný JSON nebo neodpovídá struktuře knihovny.
    ///
    /// [en] The text is not valid JSON or does not match the library structure.
    Json(String),
    /// Soubor není knihovna (chybí `"type": "logicflow-library"`).
    ///
    /// [en] The file is not a library (`"type": "logicflow-library"` missing).
    NotLibrary,
    /// Soubor je z novější verze formátu.
    ///
    /// [en] The file is in a newer format version.
    NewerFormat {
        /// Verze formátu v souboru.
        ///
        /// [en] Format version in the file.
        found: u32,
    },
    /// Identita knihovny nebo prvku není UUID.
    ///
    /// [en] The identity of the library or an item is not a UUID.
    BadUid(String),
    /// Dva prvky mají stejnou identitu.
    ///
    /// [en] Two items have the same identity.
    DuplicateUid(String),
    /// Dva prvky mají stejné číslo `id`.
    ///
    /// [en] Two items have the same number `id`.
    DuplicateId(u32),
    /// Prvek používá prvek s číslem, které v knihovně není.
    ///
    /// [en] An item uses an item with a number that is not in the library.
    MissingItem {
        /// Název prvku, který odkaz obsahuje.
        ///
        /// [en] Name of the item containing the reference.
        item: String,
        /// Chybějící číslo prvku.
        ///
        /// [en] The missing item number.
        id: u32,
    },
    /// Prvek obsahuje (i nepřímo) sám sebe.
    ///
    /// [en] An item contains itself (even indirectly).
    Cycle(String),
}

impl LibraryError {
    /// Text chyby pro uživatele v jazyce `lang` (`"cs"` česky, jinak anglicky).
    ///
    /// [en] Error text for the user in the language `lang` (`"cs"` Czech, otherwise English).
    pub fn text(&self, lang: &str) -> String {
        let cs = lang == "cs";
        match self {
            Self::Json(e) if cs => format!("Soubor knihovny nejde přečíst: {e}"),
            Self::Json(e) => format!("The library file cannot be read: {e}"),
            Self::NotLibrary if cs => "Soubor není knihovna LogicFlow.".to_owned(),
            Self::NotLibrary => "The file is not a LogicFlow library.".to_owned(),
            Self::NewerFormat { found } if cs => format!(
                "Knihovna je z novější verze aplikace (formát {found}), tato verze umí formát {}.",
                Library::FORMAT
            ),
            Self::NewerFormat { found } => format!(
                "The library is from a newer version of the application (format {found}), \
                 this version supports format {}.",
                Library::FORMAT
            ),
            Self::BadUid(u) if cs => format!("Neplatná identita (uid) „{u}“."),
            Self::BadUid(u) => format!("Invalid identity (uid) “{u}”."),
            Self::DuplicateUid(u) if cs => format!("Dva prvky mají stejnou identitu „{u}“."),
            Self::DuplicateUid(u) => format!("Two items have the same identity “{u}”."),
            Self::DuplicateId(id) if cs => format!("Dva prvky mají stejné číslo {id}."),
            Self::DuplicateId(id) => format!("Two items have the same number {id}."),
            Self::MissingItem { item, id } if cs => {
                format!("Prvek „{item}“ používá prvek číslo {id}, který v knihovně není.")
            }
            Self::MissingItem { item, id } => {
                format!("Item “{item}” uses item number {id}, which is not in the library.")
            }
            Self::Cycle(name) if cs => format!("Prvek „{name}“ obsahuje sám sebe."),
            Self::Cycle(name) => format!("Item “{name}” contains itself."),
        }
    }
}

impl fmt::Display for LibraryError {
    /// Česky, stejně jako ostatní chyby jádra; pro rozhraní slouží [`LibraryError::text`].
    ///
    /// [en] In Czech, like other core errors; the user interface uses [`LibraryError::text`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text("cs"))
    }
}

impl std::error::Error for LibraryError {}

/// Podoba souboru `.lflib` (pro serde).
///
/// [en] Shape of the `.lflib` file (for serde).
#[derive(Serialize, Deserialize)]
struct LibraryFile {
    /// Verze formátu.
    ///
    /// [en] Format version.
    format: u32,
    /// Druh souboru – vždy `logicflow-library`.
    ///
    /// [en] Kind of file – always `logicflow-library`.
    #[serde(rename = "type", default)]
    file_type: String,
    /// Název a verze aplikace, která soubor uložila.
    ///
    /// [en] Name and version of the application that saved the file.
    #[serde(default)]
    app: String,
    /// Hlavička knihovny.
    ///
    /// [en] Library header.
    library: LibraryInfo,
    /// Prvky knihovny.
    ///
    /// [en] Library items.
    #[serde(default)]
    items: Vec<LibraryItem>,
}

/// Knihovna prvků.
///
/// [en] Element library.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    /// Hlavička knihovny.
    ///
    /// [en] Library header.
    pub info: LibraryInfo,
    /// Prvky v pořadí, v jakém jsou v souboru.
    ///
    /// [en] Items in the order they are in the file.
    pub items: Vec<LibraryItem>,
}

/// Čísla podobvodů použitých přímo v obvodu.
///
/// [en] Numbers of subcircuits used directly in the circuit.
fn used_ids(circuit: &Circuit) -> Vec<u32> {
    circuit
        .components
        .iter()
        .filter_map(|c| match c.kind {
            ComponentKind::Subcircuit { id, .. } => Some(id),
            _ => None,
        })
        .collect()
}

impl Library {
    /// Aktuální verze formátu.
    ///
    /// [en] Current format version.
    pub const FORMAT: u32 = 1;
    /// Přípona souboru.
    ///
    /// [en] File extension.
    pub const EXTENSION: &'static str = "lflib";
    /// Hodnota pole `type`, podle které se knihovna pozná od projektu.
    ///
    /// [en] Value of the `type` field that tells a library from a project.
    pub const FILE_TYPE: &'static str = "logicflow-library";

    /// Prázdná knihovna s danou hlavičkou.
    ///
    /// [en] Empty library with the given header.
    pub fn new(info: LibraryInfo) -> Self {
        Self {
            info,
            items: Vec::new(),
        }
    }

    /// Načte knihovnu z textu souboru `.lflib` a zkontroluje ji ([`Library::check`]).
    ///
    /// [en] Loads a library from the text of a `.lflib` file and checks it ([`Library::check`]).
    pub fn from_json(text: &str) -> Result<Self, LibraryError> {
        // Nejdřív typ a formát – novější soubor může mít strukturu, kterou neznáme.
        // [en] Type and format first – a newer file may have a structure we do not know.
        let head: serde_json::Value =
            serde_json::from_str(text).map_err(|e| LibraryError::Json(e.to_string()))?;
        if head.get("type").and_then(|t| t.as_str()) != Some(Self::FILE_TYPE) {
            return Err(LibraryError::NotLibrary);
        }
        let found = head.get("format").and_then(|f| f.as_u64()).unwrap_or(0);
        if found > u64::from(Self::FORMAT) {
            return Err(LibraryError::NewerFormat {
                found: u32::try_from(found).unwrap_or(u32::MAX),
            });
        }
        let file: LibraryFile =
            serde_json::from_value(head).map_err(|e| LibraryError::Json(e.to_string()))?;
        let lib = Self {
            info: file.library,
            items: file.items,
        };
        lib.check()?;
        Ok(lib)
    }

    /// Text souboru `.lflib`: kategorie vestavěných kategorií jako klíče, názvy
    /// vývodů přepočítané ze zapojení, počty vývodů instancí podle definic.
    ///
    /// [en] Text of the `.lflib` file: built-in categories as keys, pin names
    /// recomputed from the circuit, pin counts of instances according to the definitions.
    pub fn to_json(&self) -> String {
        let defs = self.defs();
        let items = self
            .items
            .iter()
            .map(|item| {
                let mut item = item.clone();
                item.category = normalize_category(&item.category);
                (item.inputs, item.outputs) = item.def().pin_names();
                sync_instances(&mut item.circuit, &defs);
                item
            })
            .collect();
        let file = LibraryFile {
            format: Self::FORMAT,
            file_type: Self::FILE_TYPE.to_owned(),
            app: format!("LogicFlow {}", crate::VERSION),
            library: self.info.clone(),
            items,
        };
        serde_json::to_string_pretty(&file).expect("serializace knihovny nemůže selhat")
    }

    /// Zkontroluje identity (UUID, jedinečnost), čísla prvků, odkazy mezi prvky
    /// a že žádný prvek neobsahuje sám sebe.
    ///
    /// [en] Checks identities (UUID, uniqueness), item numbers, references between
    /// items and that no item contains itself.
    pub fn check(&self) -> Result<(), LibraryError> {
        if !is_uid(&self.info.uid) {
            return Err(LibraryError::BadUid(self.info.uid.clone()));
        }
        let mut uids = HashSet::new();
        let mut ids = HashSet::new();
        for item in &self.items {
            if !is_uid(&item.uid) {
                return Err(LibraryError::BadUid(item.uid.clone()));
            }
            // UUID se porovnává bez ohledu na velikost písmen.
            // [en] UUIDs are compared case-insensitively.
            if !uids.insert(item.uid.to_lowercase()) {
                return Err(LibraryError::DuplicateUid(item.uid.clone()));
            }
            if !ids.insert(item.id) {
                return Err(LibraryError::DuplicateId(item.id));
            }
        }
        for item in &self.items {
            if let Some(id) = used_ids(&item.circuit)
                .into_iter()
                .find(|id| !ids.contains(id))
            {
                return Err(LibraryError::MissingItem {
                    item: item.name.clone(),
                    id,
                });
            }
        }
        let defs = self.defs();
        if let Some(d) = defs.iter().find(|d| d.uses(d.id, &defs)) {
            return Err(LibraryError::Cycle(d.name.clone()));
        }
        Ok(())
    }

    /// Definice podobvodů všech prvků (pro simulaci a kontrolu).
    ///
    /// [en] Subcircuit definitions of all items (for simulation and checking).
    pub fn defs(&self) -> Vec<SubcircuitDef> {
        self.items.iter().map(LibraryItem::def).collect()
    }

    /// Prvek podle identity (bez ohledu na velikost písmen).
    ///
    /// [en] Item by its identity (case-insensitive).
    pub fn item(&self, uid: &str) -> Option<&LibraryItem> {
        self.items.iter().find(|i| i.uid.eq_ignore_ascii_case(uid))
    }

    /// Prvek podle identity pro úpravu.
    ///
    /// [en] Item by its identity for editing.
    pub fn item_mut(&mut self, uid: &str) -> Option<&mut LibraryItem> {
        self.items
            .iter_mut()
            .find(|i| i.uid.eq_ignore_ascii_case(uid))
    }

    /// Číslo pro nový prvek (o jedna vyšší než nejvyšší použité).
    ///
    /// [en] Number for a new item (one higher than the highest used).
    pub fn next_item_id(&self) -> u32 {
        self.items.iter().map(|i| i.id + 1).max().unwrap_or(1)
    }

    /// Prvek `uid` a všechny prvky, které (i nepřímo) používá, jako definice
    /// podobvodů seřazené podle čísla. `None`, když prvek v knihovně není.
    ///
    /// [en] Item `uid` and all items it uses (even indirectly) as subcircuit
    /// definitions sorted by number. `None` when the item is not in the library.
    pub fn item_with_dependencies(&self, uid: &str) -> Option<Vec<SubcircuitDef>> {
        let root = self.item(uid)?;
        let defs = self.defs();
        let mut needed: Vec<SubcircuitDef> = Vec::new();
        let mut stack = vec![root.id];
        while let Some(id) = stack.pop() {
            if needed.iter().any(|d| d.id == id) {
                continue;
            }
            if let Some(d) = defs.iter().find(|d| d.id == id) {
                stack.extend(used_ids(&d.circuit));
                needed.push(d.clone());
            }
        }
        needed.sort_by_key(|d| d.id);
        Some(needed)
    }

    /// Text pro vložení prvku do projektu: stejný tvar jako schránka (soubor
    /// `.logicflow` s jednou instancí prvku a definicemi všech potřebných prvků),
    /// takže se vloží stejně jako Ctrl+V – podobvody se sloučí s těmi, které
    /// projekt už má. Podobvody dostanou názvy v jazyce `lang` (jinak hlavní).
    /// `None`, když prvek v knihovně není.
    ///
    /// [en] Text for inserting an item into a project: the same shape as the
    /// clipboard (a `.logicflow` file with one instance of the item and the
    /// definitions of all needed items), so it is inserted just like Ctrl+V –
    /// subcircuits are merged with those the project already has. The subcircuits
    /// get their names in the language `lang` (otherwise the main ones). `None` when
    /// the item is not in the library.
    pub fn insertion_text(&self, uid: &str, lang: &str) -> Option<String> {
        let mut defs = self.item_with_dependencies(uid)?;
        for d in &mut defs {
            if let Some(i) = self.items.iter().find(|i| i.id == d.id) {
                d.name = i.display_name(lang).to_owned();
            }
        }
        let item = self.item(uid)?;
        let mut circuit = Circuit::default();
        circuit.add_component(item.def().kind(), Point::new(0, 0));
        sync_instances(&mut circuit, &defs);
        Some(ProjectFile::to_json(&circuit, &defs))
    }

    /// Zaznamená uložení knihovny: zvýší verzi a nastaví datum úpravy.
    ///
    /// [en] Records saving the library: increases the version and sets the modification date.
    pub fn mark_modified(&mut self, today: &str) {
        self.info.version += 1;
        self.info.modified = today.to_owned();
    }
}

/// Je text souboru knihovna (a ne projekt)? Rychlá zkouška bez úplného načtení.
///
/// [en] Is the file text a library (and not a project)? A quick test without full loading.
pub fn is_library_text(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v.get("type")?.as_str().map(|t| t == Library::FILE_TYPE))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::circuit_uses;
    use crate::{CompId, LogicValue, Simulator};

    /// Vzorová knihovna z `docs/knihovny` (úplná sčítačka a 2bitová sčítačka z ní).
    ///
    /// [en] Sample library from `docs/knihovny` (a full adder and a 2-bit adder made of it).
    const VZOR: &str = include_str!("../../../docs/knihovny/vzor.lflib");

    /// Identita úplné sčítačky ve vzorové knihovně.
    ///
    /// [en] Identity of the full adder in the sample library.
    const FULL_ADDER: &str = "ce146411-078a-4bdc-8ad0-5b80bbab6fa4";
    /// Identita 2bitové sčítačky ve vzorové knihovně.
    ///
    /// [en] Identity of the 2-bit adder in the sample library.
    const ADDER_2BIT: &str = "5f0c2a1e-8d3b-4e6a-9c71-2b4d8e0f6a13";

    /// Projde všechny kombinace vstupů prvku a vrátí hodnoty výstupů (vstupy a výstupy
    /// seřazené podle popisku, první vstup je nejvyšší bit čísla kombinace).
    ///
    /// [en] Goes through all input combinations of the item and returns the output
    /// values (inputs and outputs sorted by label, the first input is the highest bit
    /// of the combination number).
    fn truth_table(lib: &Library, uid: &str) -> Vec<Vec<LogicValue>> {
        let item = lib.item(uid).unwrap();
        let defs = lib.defs();
        let (ins, outs) = item.circuit.io();
        let ins: Vec<CompId> = ins.iter().map(|c| c.id).collect();
        let outs: Vec<CompId> = outs.iter().map(|c| c.id).collect();
        let mut sim = Simulator::with_subcircuits(&item.circuit, &defs, None);
        (0..1u32 << ins.len())
            .map(|n| {
                for (i, &id) in ins.iter().enumerate() {
                    let bit = n >> (ins.len() - 1 - i) & 1 == 1;
                    sim.set_source(id, LogicValue::from(bit));
                }
                assert!(sim.run(10_000), "obvod se ustálí / settles");
                outs.iter().map(|&id| sim.inputs(id)[0]).collect()
            })
            .collect()
    }

    /// Číslo z hodnot (první je nejvyšší bit).
    ///
    /// [en] Number from values (the first is the highest bit).
    fn number(bits: &[LogicValue]) -> u32 {
        bits.iter()
            .fold(0, |n, &b| n << 1 | u32::from(b == LogicValue::One))
    }

    #[test]
    fn sample_loads() {
        let lib = Library::from_json(VZOR).unwrap();
        assert_eq!(lib.items.len(), 2);
        let fa = lib.item(FULL_ADDER).unwrap();
        assert_eq!(fa.display_name("cs"), "Úplná sčítačka 1 bit");
        assert_eq!(fa.display_name("en"), "1-bit full adder");
        // Jazyk bez překladu → hlavní název.
        // [en] Language without a translation → the main name.
        assert_eq!(fa.display_name("uk"), "Úplná sčítačka 1 bit");
        assert_eq!(fa.category, "arithmetic");
        assert_eq!(fa.category_name("cs"), "Aritmetika");
        assert_eq!(fa.license, DEFAULT_LICENSE);
        let (ins, outs) = fa.def().pin_names();
        assert_eq!((ins, outs), (fa.inputs.clone(), fa.outputs.clone()));
    }

    #[test]
    fn full_adder_truth_table() {
        let lib = Library::from_json(VZOR).unwrap();
        // Vstupy A, B, Cin; výstupy Cout, S → Cout·2 + S = A + B + Cin.
        // [en] Inputs A, B, Cin; outputs Cout, S → Cout·2 + S = A + B + Cin.
        for (n, out) in truth_table(&lib, FULL_ADDER).iter().enumerate() {
            assert_eq!(number(out), (n as u32).count_ones(), "kombinace {n:03b}");
        }
    }

    #[test]
    fn nested_item_adds() {
        let lib = Library::from_json(VZOR).unwrap();
        // Vstupy A0, A1, B0, B1, Cin; výstupy Cout, S0, S1.
        // [en] Inputs A0, A1, B0, B1, Cin; outputs Cout, S0, S1.
        for (n, out) in truth_table(&lib, ADDER_2BIT).iter().enumerate() {
            let n = n as u32;
            let bit = |k: u32| n >> k & 1;
            let a = bit(4) | bit(3) << 1;
            let b = bit(2) | bit(1) << 1;
            let sum = a + b + bit(0);
            let got = number(&[out[0], out[2], out[1]]);
            assert_eq!(got, sum, "A={a} B={b} Cin={}", bit(0));
        }
    }

    #[test]
    fn round_trip() {
        let lib = Library::from_json(VZOR).unwrap();
        let again = Library::from_json(&lib.to_json()).unwrap();
        assert_eq!(lib, again);
        // Vzorový soubor je uložený přesně tak, jak by ho uložil program
        // (kromě řádku s verzí aplikace).
        // [en] The sample file is saved exactly as the program would save it
        // (except the line with the application version).
        let strip = |t: &str| {
            t.lines()
                .filter(|l| !l.trim_start().starts_with("\"app\""))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(strip(&lib.to_json()), strip(VZOR.trim_end()));
    }

    #[test]
    fn dependencies_and_insertion() {
        let lib = Library::from_json(VZOR).unwrap();
        let deps = lib.item_with_dependencies(ADDER_2BIT).unwrap();
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["Úplná sčítačka 1 bit", "Sčítačka 2 bity"]);
        assert_eq!(lib.item_with_dependencies(FULL_ADDER).unwrap().len(), 1);
        assert!(
            lib.item_with_dependencies("00000000-0000-4000-8000-000000000000")
                .is_none()
        );
        // Text pro vložení je obvod s jednou instancí a definicemi (jako schránka).
        // [en] The insertion text is a circuit with one instance and definitions (like the clipboard).
        let (c, defs) =
            ProjectFile::from_json(&lib.insertion_text(ADDER_2BIT, "cs").unwrap()).unwrap();
        assert_eq!(c.components.len(), 1);
        assert_eq!(
            c.components[0].kind,
            ComponentKind::Subcircuit {
                id: 2,
                inputs: 5,
                outputs: 3
            }
        );
        assert_eq!(defs, deps);
        // V angličtině dostanou podobvody anglické názvy.
        // [en] In English the subcircuits get English names.
        let (_, defs) =
            ProjectFile::from_json(&lib.insertion_text(ADDER_2BIT, "en").unwrap()).unwrap();
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["1-bit full adder", "2-bit adder"]);
    }

    #[test]
    fn rejects_bad_files() {
        let lib = Library::from_json(VZOR).unwrap();
        assert_eq!(
            Library::from_json("{\"format\": 1, \"app\": \"x\", \"circuit\": {}}"),
            Err(LibraryError::NotLibrary)
        );
        let newer = VZOR.replacen("\"format\": 1", "\"format\": 2", 1);
        assert_eq!(
            Library::from_json(&newer),
            Err(LibraryError::NewerFormat { found: 2 })
        );
        assert!(matches!(
            Library::from_json("nic"),
            Err(LibraryError::Json(_))
        ));

        let mut bad = lib.clone();
        bad.items[1].uid = bad.items[0].uid.to_uppercase();
        assert!(matches!(bad.check(), Err(LibraryError::DuplicateUid(_))));

        let mut bad = lib.clone();
        bad.items[1].id = bad.items[0].id;
        assert_eq!(bad.check(), Err(LibraryError::DuplicateId(1)));

        let mut bad = lib.clone();
        bad.items[1].uid = "scitacka".to_owned();
        assert!(matches!(bad.check(), Err(LibraryError::BadUid(_))));

        // Bez úplné sčítačky chybí prvek, který 2bitová sčítačka používá.
        // [en] Without the full adder, an item used by the 2-bit adder is missing.
        let mut bad = lib.clone();
        bad.items.remove(0);
        assert_eq!(
            bad.check(),
            Err(LibraryError::MissingItem {
                item: "Sčítačka 2 bity".to_owned(),
                id: 1
            })
        );

        // Úplná sčítačka, která obsahuje 2bitovou sčítačku → cyklus.
        // [en] A full adder containing the 2-bit adder → a cycle.
        let mut bad = lib.clone();
        let kind = bad.items[1].def().kind();
        bad.items[0].circuit.add_component(kind, Point::new(40, 0));
        assert!(matches!(bad.check(), Err(LibraryError::Cycle(_))));
        assert!(circuit_uses(&bad.items[0].circuit, 2, &bad.defs(), 0));

        // Chyby mají text v obou jazycích.
        // [en] Errors have text in both languages.
        let e = LibraryError::DuplicateId(3);
        assert!(e.text("cs").contains("stejné číslo"));
        assert!(e.text("en").contains("same number"));
        assert_eq!(e.to_string(), e.text("cs"));
    }

    #[test]
    fn categories() {
        assert_eq!(normalize_category("Aritmetika"), "arithmetic");
        assert_eq!(normalize_category("  sequential circuits "), "sequential");
        assert_eq!(normalize_category("Moje bloky "), "Moje bloky");
        assert_eq!(category_name("code-converters", "cs"), "Převodníky kódů");
        assert_eq!(category_name("code-converters", "en"), "Code converters");
        assert_eq!(category_name("Moje bloky", "en"), "Moje bloky");
        // Klíče jsou jedinečné a nemají velká písmena (porovnávají se malými).
        // [en] Keys are unique and lowercase (they are compared in lowercase).
        let keys: HashSet<&str> = CATEGORIES.iter().map(|c| c.key).collect();
        assert_eq!(keys.len(), CATEGORIES.len());
        assert!(CATEGORIES.iter().all(|c| c.key == c.key.to_lowercase()));
    }

    #[test]
    fn uids() {
        let u = uid_from_random([0xff; 16]);
        assert_eq!(u, "ffffffff-ffff-4fff-bfff-ffffffffffff");
        assert!(is_uid(&u));
        assert!(is_uid(&uid_from_random([0; 16])));
        assert!(!is_uid("ffffffff-ffff-4fff-bfff-fffffffffff"));
        assert!(!is_uid("uplna-scitacka-1-bit"));
        assert!(is_library_text(VZOR));
        assert!(!is_library_text("{\"format\": 1}"));
    }

    #[test]
    fn new_library_and_modification() {
        let mut lib = Library::new(LibraryInfo::new(
            uid_from_random([7; 16]),
            "Moje knihovna",
            "Jan Novák",
            "2026-11-22",
        ));
        assert!(lib.check().is_ok());
        assert_eq!(lib.next_item_id(), 1);
        assert_eq!(lib.info.license, DEFAULT_LICENSE);
        lib.mark_modified("2026-11-23");
        assert_eq!(
            (lib.info.version, lib.info.modified.as_str()),
            (2, "2026-11-23")
        );
        let sample = Library::from_json(VZOR).unwrap();
        assert_eq!(sample.next_item_id(), 3);
        let mut item = sample.items[0].clone();
        item.mark_modified("2026-11-23");
        assert_eq!(item.version, 2);
    }
}
