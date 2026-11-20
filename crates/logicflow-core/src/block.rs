//! Funkční bloky: multiplexor, dekodér, kodér, sčítačka, komparátor,
//! převodník BCD na sedmisegmentovku, čítač, registr a posuvný registr.
//!
//! Všechny bloky mají obdélníkovou značku podle ČSN EN 60617-12 se vstupy
//! vlevo a výstupy vpravo. Bity jsou seřazené shora od nejnižšího (D0, Q0, …).
//!
//! Řídicí vstupy R (nulování) a EN (povolení) lze nechat nepřipojené:
//! nepřipojený R se chová jako 0, nepřipojený EN jako 1.

use crate::LogicValue;
use serde::{Deserialize, Serialize};

/// Druh funkčního bloku.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BlockKind {
    /// Multiplexor 2^select → 1 se vstupem povolení EN.
    Mux { select: u8 },
    /// Dekodér (demultiplexor) z `bits` adresových bitů na 2^bits výstupů.
    Decoder { bits: u8 },
    /// Prioritní kodér 2^bits → bits s výstupem platnosti V.
    Encoder { bits: u8 },
    /// Sčítačka dvou `bits`bitových čísel s přenosem.
    Adder { bits: u8 },
    /// Komparátor dvou `bits`bitových čísel.
    Comparator { bits: u8 },
    /// Převodník kódu BCD (a hexadecimálního) na sedmisegmentovku.
    BcdTo7Seg,
    /// Synchronní binární čítač s asynchronním nulováním.
    Counter { bits: u8, down: bool },
    /// Paralelní registr (střádač) s asynchronním nulováním.
    Register { bits: u8 },
    /// Posuvný registr se sériovým vstupem a paralelními výstupy.
    ShiftRegister { bits: u8 },
}

/// Rozsah parametru bloku (nejmenší, největší hodnota).
pub type ParamRange = (u8, u8);

impl BlockKind {
    /// Výchozí podoba všech bloků (pro nabídku součástek).
    pub const ALL: [Self; 9] = [
        Self::Mux { select: 1 },
        Self::Decoder { bits: 2 },
        Self::Encoder { bits: 2 },
        Self::Adder { bits: 1 },
        Self::Comparator { bits: 4 },
        Self::BcdTo7Seg,
        Self::Counter {
            bits: 4,
            down: false,
        },
        Self::Register { bits: 4 },
        Self::ShiftRegister { bits: 4 },
    ];

    /// Hlavní parametr (počet bitů / adresových vstupů), pokud ho blok má.
    pub fn param(self) -> Option<u8> {
        match self {
            Self::Mux { select: n }
            | Self::Decoder { bits: n }
            | Self::Encoder { bits: n }
            | Self::Adder { bits: n }
            | Self::Comparator { bits: n }
            | Self::Counter { bits: n, .. }
            | Self::Register { bits: n }
            | Self::ShiftRegister { bits: n } => Some(n),
            Self::BcdTo7Seg => None,
        }
    }

    /// Povolený rozsah hlavního parametru.
    pub fn param_range(self) -> ParamRange {
        match self {
            Self::Mux { .. } | Self::Decoder { .. } => (1, 3),
            Self::Encoder { .. } => (2, 3),
            Self::Adder { .. } | Self::Comparator { .. } => (1, 4),
            Self::Counter { .. } | Self::Register { .. } | Self::ShiftRegister { .. } => (2, 8),
            Self::BcdTo7Seg => (0, 0),
        }
    }

    /// Kopie bloku s novým hlavním parametrem (oříznutým do povoleného rozsahu).
    pub fn with_param(self, n: u8) -> Self {
        let (lo, hi) = self.param_range();
        let n = n.clamp(lo, hi);
        match self {
            Self::Mux { .. } => Self::Mux { select: n },
            Self::Decoder { .. } => Self::Decoder { bits: n },
            Self::Encoder { .. } => Self::Encoder { bits: n },
            Self::Adder { .. } => Self::Adder { bits: n },
            Self::Comparator { .. } => Self::Comparator { bits: n },
            Self::Counter { down, .. } => Self::Counter { bits: n, down },
            Self::Register { .. } => Self::Register { bits: n },
            Self::ShiftRegister { .. } => Self::ShiftRegister { bits: n },
            Self::BcdTo7Seg => Self::BcdTo7Seg,
        }
    }

    /// Kvalifikační značka uvnitř obdélníku (ČSN EN 60617-12).
    pub fn qualifier(self) -> String {
        match self {
            Self::Mux { .. } => "MUX".into(),
            Self::Decoder { .. } => "X/Y".into(),
            Self::Encoder { .. } => "HPRI".into(),
            Self::Adder { .. } => "Σ".into(),
            Self::Comparator { .. } => "COMP".into(),
            Self::BcdTo7Seg => "BCD/7SEG".into(),
            Self::Counter { bits, .. } => format!("CTR{bits}"),
            Self::Register { bits } => format!("RG{bits}"),
            Self::ShiftRegister { bits } => format!("SRG{bits}"),
        }
    }

    /// Obsahuje blok paměť (stav)?
    pub fn is_sequential(self) -> bool {
        matches!(
            self,
            Self::Counter { .. } | Self::Register { .. } | Self::ShiftRegister { .. }
        )
    }

    /// Počet bitů vnitřního stavu.
    pub fn state_bits(self) -> usize {
        match self {
            Self::Counter { bits, .. } | Self::Register { bits } | Self::ShiftRegister { bits } => {
                bits as usize
            }
            _ => 0,
        }
    }

    /// Názvy vstupů shora dolů.
    pub fn input_names(self) -> Vec<String> {
        let bus = |p: &str, n: u8| (0..n).map(|i| format!("{p}{i}")).collect::<Vec<_>>();
        match self {
            Self::Mux { select } => {
                let mut v = bus("S", select);
                v.extend(bus("D", 1 << select));
                v.push("EN".into());
                v
            }
            Self::Decoder { bits } => {
                let mut v = bus("A", bits);
                v.push("EN".into());
                v
            }
            Self::Encoder { bits } => bus("I", 1 << bits),
            Self::Adder { bits } => {
                let mut v = bus("A", bits);
                v.extend(bus("B", bits));
                v.push("CI".into());
                v
            }
            Self::Comparator { bits } => {
                let mut v = bus("A", bits);
                v.extend(bus("B", bits));
                v
            }
            Self::BcdTo7Seg => bus("D", 4),
            Self::Counter { .. } => vec!["R".into(), "EN".into(), "C".into()],
            Self::Register { bits } => {
                let mut v = bus("D", bits);
                v.extend(["R".into(), "C".into()]);
                v
            }
            Self::ShiftRegister { .. } => vec!["SI".into(), "R".into(), "C".into()],
        }
    }

    /// Názvy výstupů shora dolů.
    pub fn output_names(self) -> Vec<String> {
        let bus = |p: &str, n: u8| (0..n).map(|i| format!("{p}{i}")).collect::<Vec<_>>();
        match self {
            Self::Mux { .. } => vec!["Y".into()],
            Self::Decoder { bits } => bus("Y", 1 << bits),
            Self::Encoder { bits } => {
                let mut v = bus("A", bits);
                v.push("V".into());
                v
            }
            Self::Adder { bits } => {
                let mut v = bus("S", bits);
                v.push("CO".into());
                v
            }
            Self::Comparator { .. } => vec!["A<B".into(), "A=B".into(), "A>B".into()],
            Self::BcdTo7Seg => ["a", "b", "c", "d", "e", "f", "g"]
                .map(String::from)
                .to_vec(),
            Self::Counter { bits, .. } => {
                let mut v = bus("Q", bits);
                v.push("CO".into());
                v
            }
            Self::Register { bits } | Self::ShiftRegister { bits } => bus("Q", bits),
        }
    }

    /// Index hodinového vstupu (kreslí se u něj dynamický vstup – trojúhelník).
    pub fn clock_input(self) -> Option<usize> {
        self.is_sequential().then(|| self.input_names().len() - 1)
    }

    /// Počet řádků vývodů (výška bloku).
    fn rows(self) -> i32 {
        self.input_names().len().max(self.output_names().len()) as i32
    }

    /// Ohraničení těla v místních souřadnicích.
    pub fn body_local(self) -> (f32, f32, f32, f32) {
        let rows = self.rows() as f32;
        (-3.0, -rows - 1.0, 3.0, rows + 1.0)
    }

    /// Vývody v místních souřadnicích: (vstupy, výstupy). Mezi vývody jsou 2 body
    /// mřížky; nad prvním vývodem je místo na kvalifikační značku.
    pub fn pins_local(self) -> crate::circuit::LocalPins {
        let first = 2 - self.rows();
        let ins = (0..self.input_names().len() as i32)
            .map(|i| (-4, first + 2 * i))
            .collect();
        let outs = (0..self.output_names().len() as i32)
            .map(|i| (4, first + 2 * i))
            .collect();
        (ins, outs)
    }

    /// Výstupy kombinačního bloku podle vstupů (vstupy jsou hodnoty sítí).
    pub fn eval(self, ins: &[LogicValue]) -> Vec<LogicValue> {
        use LogicValue::{One, X, Zero};
        let n_out = self.output_names().len();
        let all_x = || vec![X; n_out];
        let raw = ins;
        let ins: Vec<LogicValue> = ins.iter().map(|v| v.as_input()).collect();
        match self {
            Self::Mux { select } => {
                let s = select as usize;
                let en = enable(raw[s + (1 << s)]);
                match en {
                    Zero => vec![Zero],
                    One => match number(&ins[..s]) {
                        Some(a) => vec![ins[s + a as usize]],
                        None => {
                            // Neznámá adresa: výsledek je jistý jen při shodě všech dat.
                            let data = &ins[s..s + (1 << s)];
                            let same = data.iter().all(|&d| d == data[0]);
                            vec![if same { data[0] } else { X }]
                        }
                    },
                    _ => vec![X],
                }
            }
            Self::Decoder { bits } => {
                let b = bits as usize;
                match (enable(raw[b]), number(&ins[..b])) {
                    (Zero, _) => vec![Zero; n_out],
                    (One, Some(a)) => (0..n_out).map(|k| (k == a as usize).into()).collect(),
                    _ => all_x(),
                }
            }
            Self::Encoder { bits } => {
                // Nejvyšší aktivní vstup vyhrává; X nad ním znamená nejistý výsledek.
                for (k, v) in ins.iter().enumerate().rev() {
                    match v {
                        One => {
                            let mut out = bits_of(k as u32, bits as usize);
                            out.push(One);
                            return out;
                        }
                        Zero => {}
                        _ => return all_x(),
                    }
                }
                vec![Zero; n_out]
            }
            Self::Adder { bits } => {
                let b = bits as usize;
                match (
                    number(&ins[..b]),
                    number(&ins[b..2 * b]),
                    ins[2 * b].to_bool(),
                ) {
                    (Some(a), Some(c), Some(ci)) => bits_of(a + c + ci as u32, b + 1),
                    _ => all_x(),
                }
            }
            Self::Comparator { bits } => {
                let b = bits as usize;
                match (number(&ins[..b]), number(&ins[b..2 * b])) {
                    (Some(a), Some(c)) => vec![(a < c).into(), (a == c).into(), (a > c).into()],
                    _ => all_x(),
                }
            }
            Self::BcdTo7Seg => match number(&ins) {
                Some(d) => {
                    let mask = SEVEN_SEG_HEX[d as usize];
                    (0..7).map(|k| (mask >> k & 1 == 1).into()).collect()
                }
                None => all_x(),
            },
            // Sekvenční bloky se vyhodnocují přes `step`.
            _ => all_x(),
        }
    }

    /// Výstupy sekvenčního bloku pro daný stav.
    pub fn outputs_from_state(self, state: &[LogicValue]) -> Vec<LogicValue> {
        let mut out = state.to_vec();
        if let Self::Counter { down, .. } = self {
            let target = if down {
                LogicValue::Zero
            } else {
                LogicValue::One
            };
            let co = if state.iter().all(|&q| q == target) {
                LogicValue::One
            } else if state.iter().any(|q| q.to_bool().is_none()) {
                LogicValue::X
            } else {
                LogicValue::Zero
            };
            out.push(co);
        }
        out
    }

    /// Krok sekvenčního bloku: podle vstupů změní stav a vrátí nové výstupy.
    /// `prev_clock` je předchozí hodnota hodinového vstupu (aktualizuje se).
    pub fn step(
        self,
        ins: &[LogicValue],
        state: &mut [LogicValue],
        prev_clock: &mut LogicValue,
    ) -> Vec<LogicValue> {
        use LogicValue::{One, X, Zero};
        let c = self.clock_input().expect("sekvenční blok má hodiny");
        let clk = ins[c].as_input();
        let edge = *prev_clock == Zero && clk == One;
        *prev_clock = clk;
        let reset = match self {
            Self::Counter { .. } => ins[0],
            Self::Register { bits } => ins[bits as usize],
            Self::ShiftRegister { .. } => ins[1],
            _ => LogicValue::Zero,
        };
        match reset_level(reset) {
            One => {
                state.fill(Zero);
                return self.outputs_from_state(state);
            }
            Zero => {}
            _ => {
                // Neznámé nulování: jistě zůstanou jen bity, které už jsou 0.
                for q in state.iter_mut().filter(|q| **q != Zero) {
                    *q = X;
                }
                return self.outputs_from_state(state);
            }
        }
        if edge {
            match self {
                Self::Counter { bits, down } => match enable(ins[1]) {
                    One => match number(state) {
                        Some(q) => {
                            let m = 1u32 << bits;
                            let next = if down { (q + m - 1) % m } else { (q + 1) % m };
                            state.copy_from_slice(&bits_of(next, bits as usize));
                        }
                        None => state.fill(X),
                    },
                    Zero => {}
                    _ => state.fill(X),
                },
                Self::Register { bits } => {
                    for (q, d) in state.iter_mut().zip(&ins[..bits as usize]) {
                        *q = d.as_input();
                    }
                }
                Self::ShiftRegister { .. } => {
                    state.rotate_right(1);
                    state[0] = ins[0].as_input();
                }
                _ => {}
            }
        }
        self.outputs_from_state(state)
    }
}

/// Vstup povolení: nepřipojený (Z) znamená povoleno.
fn enable(v: LogicValue) -> LogicValue {
    match v {
        LogicValue::Z => LogicValue::One,
        other => other.as_input(),
    }
}

/// Vstup nulování: nepřipojený (Z) znamená neaktivní.
fn reset_level(v: LogicValue) -> LogicValue {
    match v {
        LogicValue::Z => LogicValue::Zero,
        other => other.as_input(),
    }
}

/// Číslo z bitů (první je nejnižší). `None`, pokud některý bit není 0 ani 1.
pub fn number(bits: &[LogicValue]) -> Option<u32> {
    bits.iter().enumerate().try_fold(0u32, |acc, (i, v)| {
        v.as_input().to_bool().map(|b| acc | (b as u32) << i)
    })
}

/// Bity čísla (první je nejnižší).
pub fn bits_of(n: u32, count: usize) -> Vec<LogicValue> {
    (0..count).map(|i| (n >> i & 1 == 1).into()).collect()
}

/// Rozsvícené segmenty pro číslice 0–F (bit 0 = a … bit 6 = g).
pub const SEVEN_SEG_HEX: [u8; 16] = [
    0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, // 0–7
    0x7F, 0x6F, 0x77, 0x7C, 0x39, 0x5E, 0x79, 0x71, // 8–F
];

#[cfg(test)]
mod tests {
    use super::*;
    use LogicValue::{One, X, Z, Zero};

    fn v(bits: &[u8]) -> Vec<LogicValue> {
        bits.iter().map(|&b| (b == 1).into()).collect()
    }

    #[test]
    fn mux_selects_input() {
        let m = BlockKind::Mux { select: 2 };
        // S0 S1 | D0 D1 D2 D3 | EN
        let mut ins = v(&[0, 1, 0, 0, 1, 0]);
        ins.push(Z); // EN nepřipojený = povoleno
        assert_eq!(m.eval(&ins), vec![One]);
        ins[6] = Zero;
        assert_eq!(m.eval(&ins), vec![Zero]);
    }

    #[test]
    fn decoder_and_encoder_are_inverse() {
        let d = BlockKind::Decoder { bits: 3 };
        let e = BlockKind::Encoder { bits: 3 };
        for a in 0..8 {
            let mut ins = bits_of(a, 3);
            ins.push(One);
            let y = d.eval(&ins);
            assert_eq!(y.iter().filter(|&&q| q == One).count(), 1);
            let back = e.eval(&y);
            assert_eq!(number(&back[..3]), Some(a));
            assert_eq!(back[3], One);
        }
        assert_eq!(e.eval(&[Zero; 8]), vec![Zero; 4]);
    }

    #[test]
    fn adder_and_comparator() {
        let a = BlockKind::Adder { bits: 4 };
        let mut ins = bits_of(9, 4);
        ins.extend(bits_of(8, 4));
        ins.push(One);
        assert_eq!(number(&a.eval(&ins)), Some(18));
        let c = BlockKind::Comparator { bits: 4 };
        let mut ins = bits_of(3, 4);
        ins.extend(bits_of(5, 4));
        assert_eq!(c.eval(&ins), vec![One, Zero, Zero]);
        ins[4] = X;
        assert_eq!(c.eval(&ins), vec![X, X, X]);
    }

    #[test]
    fn seven_segment_digits() {
        let b = BlockKind::BcdTo7Seg;
        // 1 = segmenty b, c
        assert_eq!(b.eval(&bits_of(1, 4)), v(&[0, 1, 1, 0, 0, 0, 0]));
        // 8 = všechny segmenty
        assert_eq!(b.eval(&bits_of(8, 4)), vec![One; 7]);
    }

    fn clock(b: BlockKind, ins: &mut [LogicValue], st: &mut [LogicValue], pc: &mut LogicValue) {
        let c = b.clock_input().unwrap();
        ins[c] = Zero;
        b.step(ins, st, pc);
        ins[c] = One;
        b.step(ins, st, pc);
    }

    #[test]
    fn counter_counts_and_wraps() {
        let b = BlockKind::Counter {
            bits: 3,
            down: false,
        };
        let mut st = vec![Zero; 3];
        let mut pc = X;
        let mut ins = vec![Z, Z, Zero];
        for k in 1..=8 {
            clock(b, &mut ins, &mut st, &mut pc);
            assert_eq!(number(&st), Some(k % 8));
        }
        // Přenos při stavu 7.
        for _ in 0..7 {
            clock(b, &mut ins, &mut st, &mut pc);
        }
        assert_eq!(b.outputs_from_state(&st)[3], One);
        // EN = 0 zastaví čítání.
        ins[1] = Zero;
        clock(b, &mut ins, &mut st, &mut pc);
        assert_eq!(number(&st), Some(7));
        // Nulování.
        ins[0] = One;
        b.step(&ins, &mut st, &mut pc);
        assert_eq!(number(&st), Some(0));
    }

    #[test]
    fn down_counter() {
        let b = BlockKind::Counter {
            bits: 4,
            down: true,
        };
        let mut st = vec![Zero; 4];
        let mut pc = Zero;
        let mut ins = vec![Zero, One, Zero];
        clock(b, &mut ins, &mut st, &mut pc);
        assert_eq!(number(&st), Some(15));
    }

    #[test]
    fn shift_register_shifts() {
        let b = BlockKind::ShiftRegister { bits: 4 };
        let mut st = vec![Zero; 4];
        let mut pc = Zero;
        let mut ins = vec![One, Z, Zero];
        clock(b, &mut ins, &mut st, &mut pc);
        ins[0] = Zero;
        clock(b, &mut ins, &mut st, &mut pc);
        clock(b, &mut ins, &mut st, &mut pc);
        assert_eq!(st, v(&[0, 0, 1, 0]));
    }

    #[test]
    fn register_loads_on_edge_only() {
        let b = BlockKind::Register { bits: 4 };
        let mut st = vec![Zero; 4];
        let mut pc = Zero;
        let mut ins = bits_of(0xA, 4);
        ins.extend([Z, Zero]);
        b.step(&ins, &mut st, &mut pc);
        assert_eq!(number(&st), Some(0));
        clock(b, &mut ins, &mut st, &mut pc);
        assert_eq!(number(&st), Some(0xA));
    }

    #[test]
    fn pins_fit_body() {
        for b in BlockKind::ALL {
            let (x0, y0, x1, y1) = b.body_local();
            let (ins, outs) = b.pins_local();
            assert_eq!(ins.len(), b.input_names().len());
            assert_eq!(outs.len(), b.output_names().len());
            for (x, y) in ins.iter().chain(&outs) {
                assert!((*y as f32) > y0 + 2.0 && (*y as f32) < y1, "{b:?}");
                assert!((*x as f32) < x0 || (*x as f32) > x1);
            }
        }
    }
}
