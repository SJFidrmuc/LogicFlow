//! Základní logická hradla.
//!
//! [en] Basic logic gates.

use crate::LogicValue;
use serde::{Deserialize, Serialize};

/// Druh základního hradla.
///
/// [en] Kind of a basic gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateKind {
    /// Budič (opakovač) – výstup je rovný vstupu.
    ///
    /// [en] Buffer – the output equals the input.
    Buffer,
    /// Invertor (negace).
    ///
    /// [en] Inverter (negation).
    Not,
    /// Logický součin.
    ///
    /// [en] Logical AND.
    And,
    /// Logický součet.
    ///
    /// [en] Logical OR.
    Or,
    /// Negovaný logický součin.
    ///
    /// [en] Negated AND.
    Nand,
    /// Negovaný logický součet.
    ///
    /// [en] Negated OR.
    Nor,
    /// Nonekvivalence (lichá parita).
    ///
    /// [en] Exclusive OR (odd parity).
    Xor,
    /// Ekvivalence (sudá parita).
    ///
    /// [en] Equivalence (even parity).
    Xnor,
}

impl GateKind {
    /// Všechna hradla v pořadí, ve kterém se zobrazují v knihovně.
    ///
    /// [en] All gates in the order in which they appear in the library.
    pub const ALL: [Self; 8] = [
        Self::Buffer,
        Self::Not,
        Self::And,
        Self::Or,
        Self::Nand,
        Self::Nor,
        Self::Xor,
        Self::Xnor,
    ];

    /// Hradlo s jediným vstupem (budič, invertor).
    ///
    /// [en] Gate with a single input (buffer, inverter).
    pub const fn is_unary(self) -> bool {
        matches!(self, Self::Buffer | Self::Not)
    }

    /// Má výstup negační kroužek?
    ///
    /// [en] Does the output have a negation circle?
    pub const fn is_inverted(self) -> bool {
        matches!(self, Self::Not | Self::Nand | Self::Nor | Self::Xnor)
    }

    /// Značka uvnitř obdélníkového symbolu podle ČSN EN 60617.
    ///
    /// [en] Label inside the rectangular symbol according to EN 60617.
    pub const fn iec_label(self) -> &'static str {
        match self {
            Self::Buffer | Self::Not => "1",
            Self::And | Self::Nand => "&",
            Self::Or | Self::Nor => "≥1",
            Self::Xor | Self::Xnor => "=1",
        }
    }

    /// Anglický název (používá se i v exportu do VHDL).
    ///
    /// [en] English name (also used in the VHDL export).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Buffer => "BUF",
            Self::Not => "NOT",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Nand => "NAND",
            Self::Nor => "NOR",
            Self::Xor => "XOR",
            Self::Xnor => "XNOR",
        }
    }

    /// Vyhodnotí hradlo pro dané vstupy.
    ///
    /// Vícevstupová hradla přijmou libovolný počet vstupů (alespoň jeden).
    /// XOR s více vstupy je lichá parita, jak to dělají i obvody 74xx.
    ///
    /// [en] Evaluates the gate for the given inputs.
    ///
    /// Multi-input gates accept any number of inputs (at least one).
    /// XOR with more inputs is odd parity, as in the 74xx circuits.
    pub fn eval(self, inputs: &[LogicValue]) -> LogicValue {
        let Some((&first, rest)) = inputs.split_first() else {
            return LogicValue::X;
        };
        let first = first.as_input();
        let base = match self {
            Self::Buffer | Self::Not => first,
            Self::And | Self::Nand => rest.iter().fold(first, |acc, &v| acc.and(v)),
            Self::Or | Self::Nor => rest.iter().fold(first, |acc, &v| acc.or(v)),
            Self::Xor | Self::Xnor => rest.iter().fold(first, |acc, &v| acc.xor(v)),
        };
        if self.is_inverted() { base.not() } else { base }
    }
}

#[cfg(test)]
mod tests {
    use super::GateKind;
    use crate::LogicValue::{self, *};

    fn table(kind: GateKind) -> [LogicValue; 4] {
        [
            kind.eval(&[Zero, Zero]),
            kind.eval(&[Zero, One]),
            kind.eval(&[One, Zero]),
            kind.eval(&[One, One]),
        ]
    }

    #[test]
    fn two_input_truth_tables() {
        assert_eq!(table(GateKind::And), [Zero, Zero, Zero, One]);
        assert_eq!(table(GateKind::Or), [Zero, One, One, One]);
        assert_eq!(table(GateKind::Nand), [One, One, One, Zero]);
        assert_eq!(table(GateKind::Nor), [One, Zero, Zero, Zero]);
        assert_eq!(table(GateKind::Xor), [Zero, One, One, Zero]);
        assert_eq!(table(GateKind::Xnor), [One, Zero, Zero, One]);
    }

    #[test]
    fn unary_gates() {
        assert_eq!(GateKind::Not.eval(&[One]), Zero);
        assert_eq!(GateKind::Buffer.eval(&[One]), One);
        assert_eq!(GateKind::Buffer.eval(&[Z]), X);
    }

    #[test]
    fn multi_input() {
        assert_eq!(GateKind::And.eval(&[One, One, One, Zero]), Zero);
        assert_eq!(GateKind::Or.eval(&[Zero, Zero, Zero, One]), One);
        assert_eq!(GateKind::Xor.eval(&[One, One, One]), One, "lichá parita");
        assert_eq!(GateKind::Nand.eval(&[X, Zero]), One);
    }

    #[test]
    fn no_inputs_is_undefined() {
        assert_eq!(GateKind::And.eval(&[]), X);
    }
}
