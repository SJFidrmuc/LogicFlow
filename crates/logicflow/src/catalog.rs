//! Nabídka součástek pro Shift+A a knihovnu.

use crate::i18n::{Lang, T, tr};
use logicflow_core::{ComponentKind, FlipFlopKind, GateKind};

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
            title: T::CatOutputs,
            items: vec![(tr(lang, T::Led).to_owned(), ComponentKind::Led)],
        },
    ]
}

/// Součástky, které přibudou v dalších verzích (jen pro zobrazení v knihovně).
pub fn upcoming() -> [(T, &'static [&'static str], &'static str); 3] {
    [
        (T::CatOutputs, &["Sedmisegmentovka", "Hex displej"], "0.3"),
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
    }
}

/// Název druhu klopného obvodu.
pub fn ff_name(ff: FlipFlopKind, lang: Lang) -> String {
    match ff {
        FlipFlopKind::DLatch => format!("D ({})", tr(lang, T::LevelShort)),
        other => other.name().to_owned(),
    }
}
