//! Ukázkový obvod pro milník M0.
//!
//! Dva přepínače A a B, hradla AND a OR a dvě LED. Slouží k ověření vzhledu
//! a propojení rozhraní se simulačním jádrem, dokud neexistuje skutečný editor
//! schémat (verze 0.1).

use logicflow_core::{GateKind, LogicValue};

/// Prvek ukázkového obvodu, který jde vybrat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    SwitchA,
    SwitchB,
    GateAnd,
    GateOr,
    LedY1,
    LedY2,
}

/// Jeden záznam pro časový diagram.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub t: f64,
    pub a: LogicValue,
    pub b: LogicValue,
    pub y1: LogicValue,
    pub y2: LogicValue,
}

pub struct DemoCircuit {
    pub a: bool,
    pub b: bool,
    pub selected: Option<Item>,
    /// Simulační čas v sekundách; běží jen když je simulace spuštěná.
    pub time: f64,
    pub running: bool,
    pub history: Vec<Sample>,
}

impl Default for DemoCircuit {
    fn default() -> Self {
        let mut c = Self {
            a: true,
            b: false,
            selected: None,
            time: 0.0,
            running: true,
            history: Vec::new(),
        };
        c.record();
        c
    }
}

impl DemoCircuit {
    pub fn va(&self) -> LogicValue {
        self.a.into()
    }

    pub fn vb(&self) -> LogicValue {
        self.b.into()
    }

    /// Výstup hradla AND – vyhodnocený simulačním jádrem.
    pub fn y1(&self) -> LogicValue {
        GateKind::And.eval(&[self.va(), self.vb()])
    }

    /// Výstup hradla OR – vyhodnocený simulačním jádrem.
    pub fn y2(&self) -> LogicValue {
        GateKind::Or.eval(&[self.va(), self.vb()])
    }

    pub fn toggle(&mut self, item: Item) {
        match item {
            Item::SwitchA => self.a = !self.a,
            Item::SwitchB => self.b = !self.b,
            _ => return,
        }
        self.record();
    }

    /// Posune simulační čas (volá se každý snímek).
    pub fn advance(&mut self, dt: f64) {
        if self.running {
            self.time += dt;
        }
    }

    fn record(&mut self) {
        let s = Sample {
            t: self.time,
            a: self.va(),
            b: self.vb(),
            y1: self.y1(),
            y2: self.y2(),
        };
        // Více změn ve stejném okamžiku: platí poslední.
        if let Some(last) = self.history.last_mut()
            && (last.t - s.t).abs() < f64::EPSILON
        {
            *last = s;
            return;
        }
        self.history.push(s);
        // Historie stačí na pár minut.
        if self.history.len() > 10_000 {
            self.history.drain(..5_000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_updates_outputs_and_history() {
        let mut c = DemoCircuit::default();
        assert_eq!(c.y1(), LogicValue::Zero);
        assert_eq!(c.y2(), LogicValue::One);
        c.advance(1.0);
        c.toggle(Item::SwitchB);
        assert_eq!(c.y1(), LogicValue::One);
        assert_eq!(c.history.len(), 2);
        // Kliknutí na hradlo nic nepřepíná.
        c.toggle(Item::GateAnd);
        assert_eq!(c.history.len(), 2);
    }

    #[test]
    fn paused_time_does_not_move() {
        let mut c = DemoCircuit {
            running: false,
            ..Default::default()
        };
        c.advance(5.0);
        assert_eq!(c.time, 0.0);
    }
}
