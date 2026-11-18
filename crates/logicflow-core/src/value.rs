//! Logické hodnoty signálu.
//!
//! Simulátor pracuje se čtyřmi hodnotami, aby uměl ukázat i neurčité stavy
//! a kolize na sběrnici:
//!
//! | Hodnota | Význam |
//! |---------|--------|
//! | `0`     | logická nula |
//! | `1`     | logická jednička |
//! | `X`     | neurčitá hodnota (neinicializováno, kolize dvou budičů) |
//! | `Z`     | vysoká impedance (odpojený třístavový výstup) |

use serde::{Deserialize, Serialize};
use std::fmt;

/// Hodnota jednoho vodiče.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicValue {
    Zero,
    One,
    #[default]
    X,
    Z,
}

impl LogicValue {
    /// Převede `bool` na `0` nebo `1`.
    pub const fn from_bool(b: bool) -> Self {
        if b { Self::One } else { Self::Zero }
    }

    /// Vrací `Some(bool)` pro `0` a `1`, jinak `None`.
    pub const fn to_bool(self) -> Option<bool> {
        match self {
            Self::Zero => Some(false),
            Self::One => Some(true),
            Self::X | Self::Z => None,
        }
    }

    /// Hodnota tak, jak ji vidí vstup hradla: `Z` na vstupu se chová jako `X`.
    pub const fn as_input(self) -> Self {
        match self {
            Self::Z => Self::X,
            v => v,
        }
    }

    /// Negace (NOT).
    #[allow(clippy::should_implement_trait)]
    pub const fn not(self) -> Self {
        match self.as_input() {
            Self::Zero => Self::One,
            Self::One => Self::Zero,
            _ => Self::X,
        }
    }

    /// Logický součin dvou hodnot. `0` vyhrává i nad `X`.
    pub const fn and(self, other: Self) -> Self {
        match (self.as_input(), other.as_input()) {
            (Self::Zero, _) | (_, Self::Zero) => Self::Zero,
            (Self::One, Self::One) => Self::One,
            _ => Self::X,
        }
    }

    /// Logický součet dvou hodnot. `1` vyhrává i nad `X`.
    pub const fn or(self, other: Self) -> Self {
        match (self.as_input(), other.as_input()) {
            (Self::One, _) | (_, Self::One) => Self::One,
            (Self::Zero, Self::Zero) => Self::Zero,
            _ => Self::X,
        }
    }

    /// Nonekvivalence (XOR).
    pub const fn xor(self, other: Self) -> Self {
        match (self.to_bool(), other.to_bool()) {
            (Some(a), Some(b)) => Self::from_bool(a ^ b),
            _ => Self::X,
        }
    }

    /// Výsledná hodnota vodiče, na který je připojeno více budičů.
    ///
    /// `Z` ustoupí jakékoli jiné hodnotě, dvě různé hodnoty `0`/`1` dají `X` (kolize).
    pub const fn resolve(self, other: Self) -> Self {
        match (self, other) {
            (Self::Z, v) | (v, Self::Z) => v,
            (Self::Zero, Self::Zero) => Self::Zero,
            (Self::One, Self::One) => Self::One,
            _ => Self::X,
        }
    }

    /// Znak pro zobrazení v pravdivostní tabulce a časovém diagramu.
    pub const fn symbol(self) -> char {
        match self {
            Self::Zero => '0',
            Self::One => '1',
            Self::X => 'X',
            Self::Z => 'Z',
        }
    }
}

impl From<bool> for LogicValue {
    fn from(b: bool) -> Self {
        Self::from_bool(b)
    }
}

impl fmt::Display for LogicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::LogicValue::{self, *};

    const ALL: [LogicValue; 4] = [Zero, One, X, Z];

    #[test]
    fn and_truth_table() {
        assert_eq!(Zero.and(Zero), Zero);
        assert_eq!(Zero.and(One), Zero);
        assert_eq!(One.and(Zero), Zero);
        assert_eq!(One.and(One), One);
        // Nula je dominantní i nad neurčitou hodnotou.
        assert_eq!(Zero.and(X), Zero);
        assert_eq!(One.and(X), X);
        assert_eq!(One.and(Z), X);
    }

    #[test]
    fn or_truth_table() {
        assert_eq!(Zero.or(Zero), Zero);
        assert_eq!(Zero.or(One), One);
        assert_eq!(One.or(Zero), One);
        assert_eq!(One.or(One), One);
        assert_eq!(One.or(X), One);
        assert_eq!(Zero.or(X), X);
    }

    #[test]
    fn xor_and_not() {
        assert_eq!(Zero.xor(One), One);
        assert_eq!(One.xor(One), Zero);
        assert_eq!(One.xor(X), X);
        assert_eq!(Zero.not(), One);
        assert_eq!(One.not(), Zero);
        assert_eq!(Z.not(), X);
    }

    #[test]
    fn resolve_bus() {
        assert_eq!(Z.resolve(One), One);
        assert_eq!(Zero.resolve(Z), Zero);
        assert_eq!(Z.resolve(Z), Z);
        assert_eq!(Zero.resolve(One), X, "kolize dvou budičů");
        for v in ALL {
            assert_eq!(v.resolve(Z), v);
            assert_eq!(v.resolve(v), v);
        }
    }

    #[test]
    fn operations_are_commutative() {
        for a in ALL {
            for b in ALL {
                assert_eq!(a.and(b), b.and(a));
                assert_eq!(a.or(b), b.or(a));
                assert_eq!(a.xor(b), b.xor(a));
                assert_eq!(a.resolve(b), b.resolve(a));
            }
        }
    }
}
