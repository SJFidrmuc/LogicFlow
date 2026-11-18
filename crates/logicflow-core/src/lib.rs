//! # LogicFlow – simulační jádro
//!
//! Jádro nezávisí na grafickém rozhraní, takže jde testovat samostatně
//! a použít i z příkazové řádky (například pro automatickou kontrolu úloh).
//!
//! Obsahuje logické hodnoty, hradla, model obvodu, překlad na netlist
//! a událostní simulátor. Paměti přibudou v dalších milnících (viz plán vývoje).

pub mod circuit;
pub mod gate;
pub mod sim;
pub mod value;

pub use circuit::{
    Circuit, CompId, Component, ComponentKind, Point, ProjectFile, Rotation, Wire, WireId,
};
pub use gate::GateKind;
pub use sim::Simulator;
pub use value::LogicValue;

/// Verze jádra (shodná s verzí aplikace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
