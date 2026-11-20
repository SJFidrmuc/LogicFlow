//! Otevřený projekt: obvod, simulátor, výběr, historie úprav a záznam signálů.

use logicflow_core::circuit::sync_instances;
use logicflow_core::{
    Circuit, CompId, ComponentKind, LogicValue, ProjectFile, Simulator, SubcircuitDef, WireId,
};
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

/// Uložený stav hlavního obvodu po dobu úprav podobvodu.
struct MainStash {
    circuit: Circuit,
    sim: Simulator,
    time: f64,
    signals: Vec<Signal>,
}

pub struct Project {
    /// Právě upravovaný list: hlavní obvod, nebo podobvod `sheet`.
    pub circuit: Circuit,
    /// Definice podobvodů. Upravovaný podobvod má aktuální obsah v `circuit`.
    pub subcircuits: Vec<SubcircuitDef>,
    /// Upravovaný podobvod (`None` = hlavní obvod).
    pub sheet: Option<u32>,
    /// Cesta zpět při procházení do vnořených podobvodů.
    sheet_stack: Vec<Option<u32>>,
    main_stash: Option<MainStash>,
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
        Self::with_subcircuits(circuit, Vec::new())
    }

    pub fn with_subcircuits(circuit: Circuit, subcircuits: Vec<SubcircuitDef>) -> Self {
        let mut sim = Simulator::with_subcircuits(&circuit, &subcircuits, None);
        sim.run(EVENT_BUDGET);
        let mut p = Self {
            circuit,
            subcircuits,
            sheet: None,
            sheet_stack: Vec::new(),
            main_stash: None,
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
        let (mut circuit, mut subs) = ProjectFile::from_json(&text)?;
        // Počty vývodů instancí podle aktuálních definic.
        let defs = subs.clone();
        sync_instances(&mut circuit, &defs);
        for d in &mut subs {
            sync_instances(&mut d.circuit, &defs);
        }
        let mut p = Self::with_subcircuits(circuit, subs);
        p.path = Some(path.to_path_buf());
        Ok(p)
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let (main, subs) = self.snapshot();
        std::fs::write(path, ProjectFile::to_json(&main, &subs)).map_err(|e| e.to_string())?;
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

    /// Hlavní obvod a definice podobvodů včetně právě upravovaného listu.
    pub fn snapshot(&self) -> (Circuit, Vec<SubcircuitDef>) {
        let mut subs = self.subcircuits.clone();
        match self.sheet {
            None => (self.circuit.clone(), subs),
            Some(id) => {
                if let Some(d) = subs.iter_mut().find(|d| d.id == id) {
                    d.circuit = self.circuit.clone();
                }
                let main = self
                    .main_stash
                    .as_ref()
                    .map(|m| m.circuit.clone())
                    .unwrap_or_default();
                (main, subs)
            }
        }
    }

    /// Definice podobvodu.
    pub fn subcircuit(&self, id: u32) -> Option<&SubcircuitDef> {
        self.subcircuits.iter().find(|d| d.id == id)
    }

    /// Název právě upravovaného listu (`None` = hlavní obvod).
    pub fn sheet_name(&self) -> Option<String> {
        self.sheet
            .and_then(|id| self.subcircuit(id))
            .map(|d| d.name.clone())
    }

    /// Lze do právě upravovaného listu vložit instanci podobvodu `id`?
    /// (Podobvod nesmí obsahovat sám sebe, ani nepřímo.)
    pub fn can_place_subcircuit(&self, id: u32) -> bool {
        match self.sheet {
            None => true,
            Some(cur) => {
                id != cur
                    && self
                        .subcircuit(id)
                        .is_some_and(|d| !d.uses(cur, &self.subcircuits))
            }
        }
    }

    /// Zapíše právě upravovaný podobvod zpět do definic.
    fn store_sheet(&mut self) {
        if let Some(id) = self.sheet
            && let Some(d) = self.subcircuits.iter_mut().find(|d| d.id == id)
        {
            d.circuit = self.circuit.clone();
        }
    }

    /// Otevře list k úpravám: hlavní obvod (`None`) nebo podobvod.
    pub fn open_sheet(&mut self, target: Option<u32>) {
        self.open_sheet_inner(target);
        self.sheet_stack.clear();
    }

    /// Otevře podobvod a zapamatuje si, odkud se přišlo (Tab zpět).
    pub fn enter_subcircuit(&mut self, id: u32) {
        let from = self.sheet;
        if from != Some(id) && self.subcircuit(id).is_some() {
            self.open_sheet_inner(Some(id));
            self.sheet_stack.push(from);
        }
    }

    /// Vrátí se o úroveň výš (do nadřazeného listu). Vrací `false`, pokud není kam.
    pub fn leave_subcircuit(&mut self) -> bool {
        if self.sheet.is_none() {
            return false;
        }
        let back = self.sheet_stack.pop().flatten();
        self.open_sheet_inner(back);
        true
    }

    fn open_sheet_inner(&mut self, target: Option<u32>) {
        if target == self.sheet || target.is_some_and(|id| self.subcircuit(id).is_none()) {
            return;
        }
        self.store_sheet();
        // Rozhraní upravených podobvodů se promítne do instancí.
        let defs = self.subcircuits.clone();
        for d in &mut self.subcircuits {
            sync_instances(&mut d.circuit, &defs);
        }
        let leaving = std::mem::take(&mut self.circuit);
        if self.sheet.is_none() {
            self.main_stash = Some(MainStash {
                circuit: leaving,
                sim: self.sim.clone(),
                time: self.time,
                signals: std::mem::take(&mut self.signals),
            });
        }
        self.sheet = target;
        self.undo.clear();
        self.redo.clear();
        self.selection.clear();
        self.pending_place = None;
        match target {
            None => {
                let stash = self.main_stash.take().expect("hlavní obvod je uložený");
                self.circuit = stash.circuit;
                sync_instances(&mut self.circuit, &self.subcircuits);
                self.sim =
                    Simulator::with_subcircuits(&self.circuit, &self.subcircuits, Some(&stash.sim));
                self.time = stash.time;
                self.signals = stash.signals;
            }
            Some(id) => {
                self.circuit = self
                    .subcircuit(id)
                    .map(|d| d.circuit.clone())
                    .unwrap_or_default();
                self.sim = Simulator::with_subcircuits(&self.circuit, &self.subcircuits, None);
                self.time = 0.0;
                self.signals.clear();
            }
        }
        self.sim.run(EVENT_BUDGET);
        self.revision += 1;
        self.record_signals();
    }

    /// Vytvoří nový prázdný podobvod a otevře ho. Vrací jeho id.
    pub fn new_subcircuit(&mut self, name_prefix: &str) -> u32 {
        let id = self.subcircuits.iter().map(|d| d.id + 1).max().unwrap_or(1);
        let name = (1..)
            .map(|n| format!("{name_prefix} {n}"))
            .find(|n| !self.subcircuits.iter().any(|d| &d.name == n))
            .unwrap_or_default();
        self.subcircuits.push(SubcircuitDef {
            id,
            name,
            circuit: Circuit::default(),
        });
        self.modified = true;
        self.enter_subcircuit(id);
        id
    }

    pub fn rename_subcircuit(&mut self, id: u32, name: String) {
        if let Some(d) = self.subcircuits.iter_mut().find(|d| d.id == id) {
            d.name = name;
            self.touch();
        }
    }

    /// Je podobvod někde použitý (v hlavním obvodu nebo jiném podobvodu)?
    pub fn is_subcircuit_used(&self, id: u32) -> bool {
        let (main, subs) = self.snapshot();
        let used_in = |c: &Circuit| {
            c.components
                .iter()
                .any(|k| matches!(k.kind, ComponentKind::Subcircuit { id: i, .. } if i == id))
        };
        used_in(&main) || subs.iter().any(|d| d.id != id && used_in(&d.circuit))
    }

    /// Smaže nepoužitý podobvod. Pokud je právě otevřený, vrátí se do hlavního obvodu.
    pub fn delete_subcircuit(&mut self, id: u32) -> bool {
        if self.is_subcircuit_used(id) {
            return false;
        }
        if self.sheet == Some(id) {
            self.open_sheet(None);
        }
        self.subcircuits.retain(|d| d.id != id);
        self.sheet_stack.retain(|s| *s != Some(id));
        self.touch();
        true
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
        self.sim = Simulator::with_subcircuits(&self.circuit, &self.subcircuits, Some(&self.sim));
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
        self.sim = Simulator::with_subcircuits(&self.circuit, &self.subcircuits, None);
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
        let (ins, outs) = self.circuit.io();
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

pub use logicflow_core::circuit::natural_cmp;

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
    fn subcircuit_sheets_and_file_roundtrip() {
        let mut p = small();
        let id = p.new_subcircuit("Podobvod");
        assert_eq!(p.sheet, Some(id));
        assert_eq!(p.sheet_name().as_deref(), Some("Podobvod 1"));
        // Uvnitř: přepínač → LED (rozhraní 1 vstup, 1 výstup).
        p.circuit
            .add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        p.circuit
            .add_component(ComponentKind::Led, Point::new(4, 0));
        p.after_structure_change();
        assert!(
            !p.can_place_subcircuit(id),
            "podobvod nesmí obsahovat sám sebe"
        );
        assert!(p.leave_subcircuit());
        assert_eq!(p.sheet, None);
        assert_eq!(p.circuit.components.len(), 2, "hlavní obvod se vrátil");
        let kind = p.subcircuit(id).unwrap().kind();
        assert_eq!(
            kind,
            ComponentKind::Subcircuit {
                id,
                inputs: 1,
                outputs: 1
            }
        );
        p.circuit.add_component(kind, Point::new(20, 0));
        p.after_structure_change();
        assert!(p.is_subcircuit_used(id));
        assert!(!p.delete_subcircuit(id));

        let dir = std::env::temp_dir().join(format!("lf-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("podobvod.logicflow");
        // Uložení z podobvodu musí zapsat i hlavní obvod.
        p.enter_subcircuit(id);
        p.save(&path).unwrap();
        let q = Project::load(&path).unwrap();
        assert_eq!(q.subcircuits.len(), 1);
        assert_eq!(q.circuit.components.len(), 3);
        assert_eq!(q.subcircuits[0].circuit.components.len(), 2);
        std::fs::remove_dir_all(&dir).ok();
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
