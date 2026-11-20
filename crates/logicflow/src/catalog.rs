//! Nabídka součástek pro Shift+A a knihovnu.

use crate::i18n::{Lang, T, tr};
use logicflow_core::{BlockKind, ComponentKind, FlipFlopKind, GateKind};

pub struct Category {
    pub title: T,
    pub items: Vec<(String, ComponentKind)>,
}

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
                (tr(lang, T::Led).to_owned(), ComponentKind::Led),
                (
                    tr(lang, T::SevenSegment).to_owned(),
                    ComponentKind::SevenSegment,
                ),
                (
                    tr(lang, T::HexDisplay).to_owned(),
                    ComponentKind::HexDisplay,
                ),
            ],
        },
    ]
}

/// Součástky, které přibudou v dalších verzích (jen pro zobrazení v knihovně).
pub fn upcoming() -> [(T, &'static [&'static str], &'static str); 2] {
    [
        (T::CatMemory, &["ROM", "RAM", "EEPROM"], "0.4"),
        (
            T::Cat74xx,
            &["7400", "7404", "7408", "7432", "7474", "74138", "74161"],
            "0.6",
        ),
    ]
}

/// Název druhu součástky pro panel vlastností a hierarchii.
pub fn kind_name(kind: &ComponentKind, lang: Lang) -> String {
    match kind {
        ComponentKind::Gate { gate, .. } => format!("{} ({})", tr(lang, T::Gate), gate.name()),
        ComponentKind::Switch { .. } => tr(lang, T::Switch).to_owned(),
        ComponentKind::Button => tr(lang, T::Button).to_owned(),
        ComponentKind::Constant { value } => {
            tr(lang, if *value { T::ConstOne } else { T::ConstZero }).to_owned()
        }
        ComponentKind::Led => tr(lang, T::Led).to_owned(),
        ComponentKind::FlipFlop { ff, .. } => {
            format!("{} {}", tr(lang, T::FlipFlop), ff_name(*ff, lang))
        }
        ComponentKind::Clock { .. } => tr(lang, T::Clock).to_owned(),
        ComponentKind::Block { block } => block_name(*block, lang),
        ComponentKind::SevenSegment => tr(lang, T::SevenSegment).to_owned(),
        ComponentKind::HexDisplay => tr(lang, T::HexDisplay).to_owned(),
    }
}

/// Název funkčního bloku i s velikostí („Multiplexor 4:1“, „Čítač 4bitový“).
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
        BlockKind::Adder { bits } => format!("{} {}", tr(lang, T::BlkAdder), bit(bits)),
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
pub fn ff_name(ff: FlipFlopKind, lang: Lang) -> String {
    match ff {
        FlipFlopKind::DLatch => format!("D ({})", tr(lang, T::LevelShort)),
        other => other.name().to_owned(),
    }
}
