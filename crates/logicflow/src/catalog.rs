//! Nabídka součástek pro Shift+A a knihovnu.
//!
//! [en] Component menu for Shift+A and the library.

use crate::i18n::{Lang, T, tr};
use logicflow_core::{BlockKind, ComponentKind, FlipFlopKind, GateKind, LedColor, PinSide};

/// Kategorie součástek v nabídce (Vstupy, Hradla, …).
///
/// [en] Component category in the menu (Inputs, Gates, …).
pub struct Category {
    /// Název kategorie.
    ///
    /// [en] Category title.
    pub title: T,
    /// Položky: zobrazený název a druh součástky.
    ///
    /// [en] Items: displayed name and kind of component.
    pub items: Vec<(String, ComponentKind)>,
}

/// Všechny kategorie vestavěných součástek v daném jazyce.
///
/// [en] All categories of built-in components in the given language.
pub fn catalog(lang: Lang) -> Vec<Category> {
    vec![
        Category {
            title: T::CatInputs,
            items: vec![
                (
                    tr(lang, T::Switch).to_owned(),
                    ComponentKind::Switch { on: false },
                ),
                (tr(lang, T::Button).to_owned(), ComponentKind::Button),
                (
                    tr(lang, T::ConstZero).to_owned(),
                    ComponentKind::Constant { value: false },
                ),
                (
                    tr(lang, T::ConstOne).to_owned(),
                    ComponentKind::Constant { value: true },
                ),
                (tr(lang, T::PowerVcc).to_owned(), ComponentKind::Vcc),
                (tr(lang, T::PowerGnd).to_owned(), ComponentKind::Gnd),
                (tr(lang, T::Clock).to_owned(), ComponentKind::DEFAULT_CLOCK),
            ],
        },
        Category {
            title: T::CatGates,
            items: GateKind::ALL
                .iter()
                .map(|g| {
                    (
                        format!("{}  [{}]", g.name(), g.iec_label()),
                        ComponentKind::gate(*g),
                    )
                })
                .collect(),
        },
        Category {
            title: T::CatFlipFlops,
            items: FlipFlopKind::ALL
                .iter()
                .map(|f| (ff_name(*f, lang), ComponentKind::flip_flop(*f)))
                .collect(),
        },
        Category {
            title: T::CatBlocks,
            items: BlockKind::ALL
                .iter()
                .map(|b| (block_name(*b, lang), ComponentKind::Block { block: *b }))
                .collect(),
        },
        Category {
            title: T::CatOutputs,
            items: vec![
                (tr(lang, T::Led).to_owned(), ComponentKind::LED),
                (
                    tr(lang, T::LedAk).to_owned(),
                    ComponentKind::LedAk {
                        color: LedColor::Red,
                    },
                ),
                (
                    tr(lang, T::SevenSegment).to_owned(),
                    ComponentKind::SevenSegment {
                        pins: PinSide::Left,
                    },
                ),
                (
                    tr(lang, T::HexDisplay).to_owned(),
                    ComponentKind::HexDisplay {
                        pins: PinSide::Left,
                    },
                ),
            ],
        },
    ]
}

/// Podobvody projektu, které lze vložit do právě upravovaného listu.
///
/// [en] Project subcircuits that can be inserted into the sheet being edited.
pub fn subcircuit_items(project: &crate::project::Project) -> Vec<(String, ComponentKind)> {
    project
        .subcircuits
        .iter()
        .filter(|d| project.can_place_subcircuit(d.id))
        .map(|d| (d.name.clone(), d.kind()))
        .collect()
}

/// Verze, ve které přibudou paměti (knihovna součástek, panel Paměť).
///
/// [en] Version in which memories arrive (component library, Memory panel).
pub const MEMORY_VERSION: &str = "0.6";

/// Verze, ve které přibudou obvody 74xx.
///
/// [en] Version in which the 74xx chips arrive.
pub const CHIPS_74XX_VERSION: &str = "0.8";

/// Součástky, které přibudou v dalších verzích (jen pro zobrazení v knihovně).
///
/// [en] Components coming in later versions (only shown in the library).
pub fn upcoming() -> [(T, &'static [&'static str], &'static str); 2] {
    [
        (T::CatMemory, &["ROM", "RAM", "EEPROM"], MEMORY_VERSION),
        (
            T::Cat74xx,
            &["7400", "7404", "7408", "7432", "7474", "74138", "74161"],
            CHIPS_74XX_VERSION,
        ),
    ]
}

/// Název druhu součástky pro panel vlastností a hierarchii.
///
/// [en] Name of the kind of component for the properties panel and the hierarchy.
pub fn kind_name(kind: &ComponentKind, lang: Lang) -> String {
    match kind {
        ComponentKind::Gate { gate, .. } => format!("{} ({})", tr(lang, T::Gate), gate.name()),
        ComponentKind::Switch { .. } => tr(lang, T::Switch).to_owned(),
        ComponentKind::Button => tr(lang, T::Button).to_owned(),
        ComponentKind::Constant { value } => {
            tr(lang, if *value { T::ConstOne } else { T::ConstZero }).to_owned()
        }
        ComponentKind::Vcc => tr(lang, T::PowerVcc).to_owned(),
        ComponentKind::Gnd => tr(lang, T::PowerGnd).to_owned(),
        ComponentKind::Led { .. } => tr(lang, T::Led).to_owned(),
        ComponentKind::LedAk { .. } => tr(lang, T::LedAk).to_owned(),
        ComponentKind::FlipFlop { ff, .. } => {
            format!("{} {}", tr(lang, T::FlipFlop), ff_name(*ff, lang))
        }
        ComponentKind::Clock { .. } => tr(lang, T::Clock).to_owned(),
        ComponentKind::Block { block } => block_name(*block, lang),
        ComponentKind::SevenSegment { .. } => tr(lang, T::SevenSegment).to_owned(),
        ComponentKind::HexDisplay { .. } => tr(lang, T::HexDisplay).to_owned(),
        ComponentKind::Subcircuit { .. } => tr(lang, T::Subcircuit).to_owned(),
    }
}

/// Název barvy LED (malým písmenem, jako hodnota v nabídce).
///
/// [en] Name of an LED colour (lower case, as a value in a menu).
pub fn led_color_name(color: LedColor, lang: Lang) -> &'static str {
    tr(
        lang,
        match color {
            LedColor::Red => T::ColorRed,
            LedColor::Green => T::ColorGreen,
            LedColor::Yellow => T::ColorYellow,
            LedColor::Blue => T::ColorBlue,
            LedColor::White => T::ColorWhite,
        },
    )
}

/// Název funkčního bloku i s velikostí („Multiplexor 4:1“, „Čítač 4bitový“).
///
/// [en] Name of a function block including its size (“Multiplexer 4:1”, “4-bit counter”).
pub fn block_name(block: BlockKind, lang: Lang) -> String {
    let bit = |n: u8| match lang {
        Lang::Cs => format!("{n}bitový"),
        Lang::En => format!("{n}-bit"),
    };
    match block {
        BlockKind::Mux { select } => format!("{} {}:1", tr(lang, T::BlkMux), 1 << select),
        BlockKind::Decoder { bits } => {
            format!("{} {bits}:{}", tr(lang, T::BlkDecoder), 1 << bits)
        }
        BlockKind::Encoder { bits } => {
            format!("{} {}:{bits}", tr(lang, T::BlkEncoder), 1 << bits)
        }
        BlockKind::Adder { bits: 1 } => match lang {
            Lang::Cs => "Úplná sčítačka (1 bit)".to_owned(),
            Lang::En => "Full adder (1 bit)".to_owned(),
        },
        // Sčítačka je ženského rodu: „4bitová“.
        // [en] “Sčítačka” is feminine in Czech: “4bitová”.
        BlockKind::Adder { bits } => match lang {
            Lang::Cs => format!("{} {bits}bitová", tr(lang, T::BlkAdder)),
            Lang::En => format!("{} {}", tr(lang, T::BlkAdder), bit(bits)),
        },
        BlockKind::Comparator { bits } => {
            format!("{} {}", tr(lang, T::BlkComparator), bit(bits))
        }
        BlockKind::BcdTo7Seg => tr(lang, T::BlkBcd7Seg).to_owned(),
        BlockKind::Counter { bits, .. } => format!("{} {}", tr(lang, T::BlkCounter), bit(bits)),
        BlockKind::Register { bits } => format!("{} {}", tr(lang, T::BlkRegister), bit(bits)),
        BlockKind::ShiftRegister { bits } => format!("{} {}", tr(lang, T::BlkShift), bit(bits)),
    }
}

/// Název druhu klopného obvodu.
///
/// [en] Name of the kind of flip-flop.
pub fn ff_name(ff: FlipFlopKind, lang: Lang) -> String {
    match ff {
        FlipFlopKind::DLatch => format!("D ({})", tr(lang, T::LevelShort)),
        other => other.name().to_owned(),
    }
}
