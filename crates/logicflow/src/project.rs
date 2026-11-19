//! Otevřený projekt: obvod, simulátor, výběr, historie úprav a záznam signálů.

use logicflow_core::{Circuit, CompId, ComponentKind, LogicValue, ProjectFile, Simulator, WireId};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// Nejvýše tolik událostí se zpracuje za jeden snímek.
const EVENT_BUDGET: usize = 20_000;
const UNDO_LIMIT: usize = 200;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    pub comps: BTreeSet<CompId>,
    pub wires: BTreeSet<WireId>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.comps.is_empty() && self.wires.is_empty()
    }

    pub fn clear(&mut self) {
        self.comps.clear();
        self.wires.clear();
    }

    pub fn single_comp(&self) -> Option<CompId> {
        (self.comps.len() == 1 && self.wires.is_empty())
            .then(|| *self.comps.first().expect("jeden prvek"))
    }

    pub fn comp_vec(&self) -> Vec<CompId> {
        self.comps.iter().copied().collect()
    }

    pub fn wire_vec(&self) -> Vec<WireId> {
        self.wires.iter().copied().collect()
    }
}

/// Signál zobrazený v časovém diagramu.
#[derive(Clone, Debug)]
pub struct Signal {
    pub id: CompId,
    pub label: String,
    /// Změny hodnoty: (čas v sekundách, hodnota).
    pub changes: Vec<(f64, LogicValue)>,
}

pub struct Project {
    pub circuit: Circuit,
    pub sim: Simulator,
    pub path: Option<PathBuf>,
    /// Název příkladu, pokud projekt vznikl z příkladu a ještě nebyl uložen.
    pub title: Option<String>,
    pub modified: bool,
    pub selection: Selection,
    undo: Vec<Circuit>,
    redo: Vec<Circuit>,
    /// Zvyšuje se při každé změně obvodu (pro přepočet pravdivostní tabulky apod.).
    pub revision: u64,
    pub running: bool,
    /// Čas časového diagramu v sekundách; běží jen při spuštěné simulaci.
    pub time: f64,
    pub signals: Vec<Signal>,
    /// Součástka vybraná v knihovně, která se má umístit do schématu.
    pub pending_place: Option<ComponentKind>,
}

impl Project {
    pub fn new(circuit: Circuit) -> Self {
        let mut sim = Simulator::new(&circuit, None);
        sim.run(EVENT_BUDGET);
        let mut p = Self {
            circuit,
            sim,
            path: None,
            title: None,
            modified: false,
            selection: Selection::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            running: true,
            time: 0.0,
            signals: Vec::new(),
            pending_place: None,
        };
        p.record_signals();
        p
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let circuit = ProjectFile::from_json(&text)?;
        let mut p = Self::new(circuit);
        p.path = Some(path.to_path_buf());
        Ok(p)
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        std::fs::write(path, ProjectFile::to_json(&self.circuit)).map_err(|e| e.to_string())?;
        self.path = Some(path.to_path_buf());
        self.title = None;
        self.modified = false;
        Ok(())
    }

    /// Název do titulku okna.
    pub fn display_name(&self) -> String {
        if let Some(p) = &self.path {
            p.file_name().map_or_else(
                || p.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            )
        } else if let Some(t) = &self.title {
            t.clone()
        } else {
            "bez názvu".to_owned()
        }
    }

    /// Uloží stav pro „Zpět“. Volá se před každou úpravou.
    pub fn checkpoint(&mut self) {
        self.push_undo(self.circuit.clone());
    }

    pub fn push_undo(&mut self, state: Circuit) {
        self.undo.push(state);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            let current = std::mem::replace(&mut self.circuit, prev);
            self.keep_switch_states(&current);
            self.redo.push(current);
            self.after_structure_change();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            let current = std::mem::replace(&mut self.circuit, next);
            self.keep_switch_states(&current);
            self.undo.push(current);
            self.after_structure_change();
        }
    }

    /// Přepnutí přepínače není úprava schématu – „Zpět“ ho nemá vracet.
    fn keep_switch_states(&mut self, from: &Circuit) {
        for c in &from.components {
            if let ComponentKind::Switch { on } = c.kind
                && let Some(k) = self.circuit.component_mut(c.id)
                && matches!(k.kind, ComponentKind::Switch { .. })
            {
                k.kind = ComponentKind::Switch { on };
            }
        }
    }

    /// Po změně zapojení: přestaví simulátor (se zachováním stavu) a vyčistí výběr.
    pub fn after_structure_change(&mut self) {
        self.sim = Simulator::new(&self.circuit, Some(&self.sim));
        self.sim.run(EVENT_BUDGET);
        self.selection
            .comps
            .retain(|id| self.circuit.components.iter().any(|c| c.id == *id));
        self.selection
            .wires
            .retain(|id| self.circuit.wires.iter().any(|w| w.id == *id));
        self.touch();
    }

    /// Změna, která nemění zapojení (popisek…).
    pub fn touch(&mut self) {
        self.modified = true;
        self.revision += 1;
    }

    /// Přepne přepínač (bez záznamu do historie úprav – jde o „hraní“ s obvodem).
    pub fn toggle_switch(&mut self, id: CompId) {
        let Some(c) = self.circuit.component_mut(id) else {
            return;
        };
        if let ComponentKind::Switch { on } = &mut c.kind {
            *on = !*on;
            let v = (*on).into();
            self.sim.set_source(id, v);
            self.modified = true;
            if !self.running {
                // I při pozastavené simulaci se změna vstupu projeví, ale dál se nešíří.
                self.sim.step();
            }
        }
    }

    pub fn set_button(&mut self, id: CompId, pressed: bool) {
        let Some(c) = self.circuit.component_mut(id) else {
            return;
        };
        if matches!(c.kind, ComponentKind::Button) && c.pressed != pressed {
            c.pressed = pressed;
            self.sim.set_source(id, pressed.into());
            if !self.running {
                self.sim.step();
            }
        }
    }

    /// Volá se každý snímek.
    pub fn update(&mut self, dt: f64) {
        if self.running {
            let prev = self.time;
            self.time += dt;
            self.advance_clocks(prev, self.time);
            self.sim.run(EVENT_BUDGET);
        }
        self.record_signals();
    }

    /// Jeden takt simulace (při pozastavení). Časový diagram se posune o čtvrt sekundy,
    /// aby byl každý krok vidět.
    pub fn step(&mut self) {
        self.time += 0.25;
        self.drive_clocks();
        self.sim.step();
        self.record_signals();
    }

    fn clocks(&self) -> Vec<(CompId, f64)> {
        self.circuit
            .components
            .iter()
            .filter_map(|c| match c.kind {
                ComponentKind::Clock { period_ms } => {
                    Some((c.id, f64::from(period_ms.max(2)) / 2000.0))
                }
                _ => None,
            })
            .collect()
    }

    /// Nastaví výstupy generátorů hodin podle aktuálního času projektu.
    fn drive_clocks(&mut self) {
        for (id, half) in self.clocks() {
            let v = LogicValue::from_bool((self.time / half).floor() as u64 % 2 == 1);
            if self.sim.output(id) != Some(v) {
                self.sim.set_source(id, v);
            }
        }
    }

    /// Projde všechny hrany hodin mezi `from` a `to` v časovém pořadí a po každé
    /// nechá obvod ustálit – žádná hrana se neztratí ani při pomalém snímku.
    fn advance_clocks(&mut self, from: f64, to: f64) {
        let mut edges: Vec<(f64, CompId, LogicValue)> = Vec::new();
        for (id, half) in self.clocks() {
            let first = (from / half).floor() as u64 + 1;
            let last = (to / half).floor() as u64;
            // Při extrémně dlouhém snímku stačí posledních pár hran.
            for k in first.max(last.saturating_sub(64))..=last {
                edges.push((k as f64 * half, id, LogicValue::from_bool(k % 2 == 1)));
            }
        }
        edges.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, id, v) in edges {
            if self.sim.output(id) != Some(v) {
                self.sim.set_source(id, v);
                self.sim.run(EVENT_BUDGET);
            }
        }
        self.drive_clocks();
    }

    /// Restart simulace: klopné obvody se vrátí do počátečního stavu, hodiny na 0.
    pub fn reset_simulation(&mut self) {
        self.sim = Simulator::new(&self.circuit, None);
        self.time = 0.0;
        self.signals.clear();
        self.sim.run(EVENT_BUDGET);
        self.record_signals();
    }

    /// Hodnota zobrazovaná u součástky v časovém diagramu.
    pub fn signal_value(&self, id: CompId) -> LogicValue {
        match self.circuit.component(id).map(|c| c.kind) {
            Some(ComponentKind::Led) => self
                .sim
                .inputs(id)
                .first()
                .copied()
                .unwrap_or(LogicValue::Z),
            _ => self.sim.output(id).unwrap_or(LogicValue::Z),
        }
    }

    /// Vstupy a výstupy obvodu seřazené podle popisku.
    pub fn io_components(&self) -> (Vec<CompId>, Vec<CompId>) {
        let mut ins: Vec<&logicflow_core::Component> = self
            .circuit
            .components
            .iter()
            .filter(|c| c.kind.is_input())
            .collect();
        let mut outs: Vec<&logicflow_core::Component> = self
            .circuit
            .components
            .iter()
            .filter(|c| matches!(c.kind, ComponentKind::Led))
            .collect();
        ins.sort_by(|a, b| natural_cmp(&a.label, &b.label));
        outs.sort_by(|a, b| natural_cmp(&a.label, &b.label));
        (
            ins.iter().map(|c| c.id).collect(),
            outs.iter().map(|c| c.id).collect(),
        )
    }

    fn record_signals(&mut self) {
        let (ins, outs) = self.io_components();
        let mut clocks: Vec<&logicflow_core::Component> = self
            .circuit
            .components
            .iter()
            .filter(|c| matches!(c.kind, ComponentKind::Clock { .. }))
            .collect();
        clocks.sort_by(|a, b| natural_cmp(&a.label, &b.label));
        let ids: Vec<CompId> = clocks.iter().map(|c| c.id).chain(ins).chain(outs).collect();
        let mut old: HashMap<CompId, Signal> = self.signals.drain(..).map(|s| (s.id, s)).collect();
        for id in ids {
            let label = self
                .circuit
                .component(id)
                .map(|c| c.label.clone())
                .unwrap_or_default();
            let v = self.signal_value(id);
            let mut s = old.remove(&id).unwrap_or(Signal {
                id,
                label: String::new(),
                changes: Vec::new(),
            });
            s.label = label;
            match s.changes.last_mut() {
                Some(last) if last.1 == v => {}
                Some(last) if (last.0 - self.time).abs() < 1e-9 => last.1 = v,
                _ => s.changes.push((self.time, v)),
            }
            if s.changes.len() > 5_000 {
                s.changes.drain(..2_500);
            }
            self.signals.push(s);
        }
    }
}

/// Řazení „A2“ před „A10“.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let split = |s: &str| {
        let i = s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len());
        (s[..i].to_owned(), s[i..].parse::<u64>().unwrap_or(0))
    };
    split(a).cmp(&split(b)).then_with(|| a.cmp(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use logicflow_core::Point;

    fn small() -> Project {
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        c.add_component(ComponentKind::Led, Point::new(6, 0));
        c.add_wire(Point::new(2, 0), Point::new(4, 0));
        Project::new(c)
    }

    #[test]
    fn undo_redo_roundtrip() {
        let mut p = small();
        p.checkpoint();
        p.circuit.add_wire(Point::new(0, 5), Point::new(3, 5));
        p.after_structure_change();
        assert_eq!(p.circuit.wires.len(), 2);
        assert!(p.modified);
        p.undo();
        assert_eq!(p.circuit.wires.len(), 1);
        p.redo();
        assert_eq!(p.circuit.wires.len(), 2);
    }

    #[test]
    fn undo_keeps_switch_positions() {
        let mut p = small();
        let a = p.circuit.components[0].id;
        p.checkpoint();
        p.circuit.add_wire(Point::new(0, 5), Point::new(3, 5));
        p.after_structure_change();
        p.toggle_switch(a);
        p.undo();
        assert_eq!(p.circuit.wires.len(), 1);
        assert_eq!(
            p.circuit.component(a).unwrap().kind,
            ComponentKind::Switch { on: true }
        );
    }

    #[test]
    fn toggle_records_signal_change() {
        let mut p = small();
        let a = p.circuit.components[0].id;
        let y = p.circuit.components[1].id;
        p.update(1.0);
        p.toggle_switch(a);
        p.update(0.5);
        assert_eq!(p.signal_value(y), LogicValue::One);
        let led = p.signals.iter().find(|s| s.id == y).unwrap();
        assert_eq!(led.changes.len(), 2);
    }

    #[test]
    fn clock_drives_flip_flop_counter() {
        // Hodiny 1 Hz → klopný obvod T (T = 1) → LED: LED mění stav každou sekundu.
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Clock { period_ms: 1000 }, Point::new(0, 1));
        c.add_component(ComponentKind::Constant { value: true }, Point::new(0, -1));
        let ff = c.add_component(
            ComponentKind::flip_flop(logicflow_core::FlipFlopKind::T),
            Point::new(10, 0),
        );
        // výstup hodin (3,1) → vstup C (7,1); konstanta (2,-1) → vstup T (7,-1)
        c.add_wire(Point::new(3, 1), Point::new(7, 1));
        c.add_wire(Point::new(2, -1), Point::new(7, -1));
        let mut p = Project::new(c);
        let mut states = Vec::new();
        for _ in 0..8 {
            p.update(0.5);
            states.push(p.sim.flip_flop_state(ff).unwrap());
        }
        use LogicValue::*;
        // Náběžná hrana hodin každou sekundu (v čase 0,5; 1,5; …) překlopí T.
        assert_eq!(states, [One, One, Zero, Zero, One, One, Zero, Zero]);
        p.reset_simulation();
        assert_eq!(p.sim.flip_flop_state(ff), Some(Zero));
        assert_eq!(p.signals[0].label, "CLK", "hodiny jsou v diagramu první");
    }

    #[test]
    fn natural_order() {
        let mut v = vec!["Y10", "Y2", "B", "A"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(v, ["A", "B", "Y2", "Y10"]);
    }
}
