//! # LogicFlow – simulační jádro
//!
//! Jádro nezávisí na grafickém rozhraní, takže jde testovat samostatně
//! a použít i z příkazové řádky (například pro automatickou kontrolu úloh).
//!
//! Zatím obsahuje logické hodnoty a základní hradla. Událostní simulátor,
//! netlist a paměti přibudou v dalších milnících (viz plán vývoje).

pub mod gate;
pub mod value;

pub use gate::GateKind;
pub use value::LogicValue;

/// Verze jádra (shodná s verzí aplikace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
