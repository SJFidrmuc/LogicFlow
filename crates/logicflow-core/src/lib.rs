//! # LogicFlow – simulační jádro
//!
//! Jádro nezávisí na grafickém rozhraní, takže jde testovat samostatně
//! a použít i z příkazové řádky (například pro automatickou kontrolu úloh).
//!
//! Obsahuje logické hodnoty, hradla, funkční bloky, model obvodu, překlad
//! na netlist a událostní simulátor. Paměti přibudou v dalších milnících (viz plán vývoje).
//!
//! [en]
//!
//! # LogicFlow – simulation core
//!
//! The core does not depend on the graphical interface, so it can be tested on
//! its own and used from the command line too (for example to check assignments
//! automatically).
//!
//! It contains logic values, gates, function blocks, the circuit model, the
//! translation into a netlist and the event-driven simulator. Memories will be
//! added in later milestones (see the development plan).

// Každá položka jádra (i neveřejná) musí mít dokumentační komentář.
// [en] Every item of the core (private ones too) must have a doc comment.
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

pub mod block;
pub mod circuit;
pub mod gate;
pub mod sim;
pub mod value;

pub use block::BlockKind;
pub use circuit::{
    Circuit, CompId, Component, ComponentKind, FlipFlopKind, PinSide, Point, ProjectFile, Rotation,
    SubcircuitDef, Wire, WireId,
};
pub use gate::GateKind;
pub use sim::Simulator;
pub use value::LogicValue;

/// Verze jádra (shodná s verzí aplikace).
///
/// [en] Core version (same as the application version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
