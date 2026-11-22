//! Otevřený projekt: obvod, simulátor, výběr, historie úprav a záznam signálů.
//!
//! [en] Open project: circuit, simulator, selection, edit history and signal recording.

use logicflow_core::circuit::sync_instances;
use logicflow_core::{
    Circuit, CompId, ComponentKind, LogicValue, Point, ProjectFile, Rotation, Simulator,
    SubcircuitDef, WireId,
};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// Nejvýše tolik událostí se zpracuje za jeden snímek.
///
/// [en] At most this many events are processed per frame.
const EVENT_BUDGET: usize = 20_000;
/// Nejvyšší počet kroků „Zpět“.
///
/// [en] Maximum number of “Undo” steps.
const UNDO_LIMIT: usize = 200;

/// Vybrané součástky a vodiče ve schématu.
///
/// [en] Selected components and wires in the schematic.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    /// Vybrané součástky.
    ///
    /// [en] Selected components.
    pub comps: BTreeSet<CompId>,
    /// Vybrané vodiče.
    ///
    /// [en] Selected wires.
    pub wires: BTreeSet<WireId>,
}

impl Selection {
    /// Není nic vybráno?
    ///
    /// [en] Is nothing selected?
    pub fn is_empty(&self) -> bool {
        self.comps.is_empty() && self.wires.is_empty()
    }

    /// Zruší výběr.
    ///
    /// [en] Clears the selection.
    pub fn clear(&mut self) {
        self.comps.clear();
        self.wires.clear();
    }

    /// Id součástky, pokud je vybraná právě jedna (a žádný vodič).
    ///
    /// [en] Component id if exactly one is selected (and no wire).
    pub fn single_comp(&self) -> Option<CompId> {
        (self.comps.len() == 1 && self.wires.is_empty())
            .then(|| *self.comps.first().expect("jeden prvek"))
    }

    /// Vybrané součástky jako seznam.
    ///
    /// [en] Selected components as a list.
    pub fn comp_vec(&self) -> Vec<CompId> {
        self.comps.iter().copied().collect()
    }

    /// Vybrané vodiče jako seznam.
    ///
    /// [en] Selected wires as a list.
    pub fn wire_vec(&self) -> Vec<WireId> {
        self.wires.iter().copied().collect()
    }
}

/// Režim práce se schématem (jako v DigitalWorks).
///
/// [en] Mode of working with the schematic (as in DigitalWorks).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Úpravy: schéma jde měnit, logika se přepočítává hned, ale čas (hodiny) stojí.
    ///
    /// [en] Editing: the schematic can be changed, logic is recomputed immediately,
    /// but time (clocks) stands still.
    #[default]
    Edit,
    /// Simulace: schéma je zamčené, fungují jen ovládací prvky; čas běží nebo je
    /// pozastavený (krokování).
    ///
    /// [en] Simulation: the schematic is locked, only the controls work; time runs or
    /// is paused (stepping).
    Simulation,
}

/// Signál zobrazený v časovém diagramu.
///
/// [en] Signal shown in the timing diagram.
#[derive(Clone, Debug)]
pub struct Signal {
    /// Součástka, ke které signál patří.
    ///
    /// [en] Component the signal belongs to.
    pub id: CompId,
    /// Název řádku (popisek součástky).
    ///
    /// [en] Row name (component label).
    pub label: String,
    /// Změny hodnoty: (čas v sekundách, hodnota).
    ///
    /// [en] Value changes: (time in seconds, value).
    pub changes: Vec<(f64, LogicValue)>,
}

/// Uložený stav hlavního obvodu po dobu úprav podobvodu.
///
/// [en] Saved state of the main circuit while a subcircuit is being edited.
struct MainStash {
    /// Hlavní obvod.
    ///
    /// [en] Main circuit.
    circuit: Circuit,
    /// Simulátor hlavního obvodu.
    ///
    /// [en] Simulator of the main circuit.
    sim: Simulator,
    /// Čas časového diagramu.
    ///
    /// [en] Time of the timing diagram.
    time: f64,
    /// Záznam signálů.
    ///
    /// [en] Signal recording.
    signals: Vec<Signal>,
}

/// Otevřený projekt – vše, co editory čtou a mění.
///
/// [en] Open project – everything the editors read and change.
pub struct Project {
    /// Právě upravovaný list: hlavní obvod, nebo podobvod `sheet`.
    ///
    /// [en] Sheet being edited: the main circuit, or subcircuit `sheet`.
    pub circuit: Circuit,
    /// Definice podobvodů. Upravovaný podobvod má aktuální obsah v `circuit`.
    ///
    /// [en] Subcircuit definitions. The subcircuit being edited has its current content in `circuit`.
    pub subcircuits: Vec<SubcircuitDef>,
    /// Upravovaný podobvod (`None` = hlavní obvod).
    ///
    /// [en] Subcircuit being edited (`None` = main circuit).
    pub sheet: Option<u32>,
    /// Cesta zpět při procházení do vnořených podobvodů.
    ///
    /// [en] Way back when descending into nested subcircuits.
    sheet_stack: Vec<Option<u32>>,
    /// Hlavní obvod odložený během úprav podobvodu.
    ///
    /// [en] Main circuit put aside while a subcircuit is edited.
    main_stash: Option<MainStash>,
    /// Simulátor právě zobrazeného listu.
    ///
    /// [en] Simulator of the sheet currently shown.
    pub sim: Simulator,
    /// Soubor projektu (`None` = zatím neuloženo).
    ///
    /// [en] Project file (`None` = not saved yet).
    pub path: Option<PathBuf>,
    /// Název příkladu, pokud projekt vznikl z příkladu a ještě nebyl uložen.
    ///
    /// [en] Name of the example if the project was created from an example and has not been saved yet.
    pub title: Option<String>,
    /// Jsou v projektu neuložené změny?
    ///
    /// [en] Does the project have unsaved changes?
    pub modified: bool,
    /// Výběr ve schématu.
    ///
    /// [en] Selection in the schematic.
    pub selection: Selection,
    /// Historie pro „Zpět“ (kopie obvodu před úpravou).
    ///
    /// [en] History for “Undo” (copies of the circuit before an edit).
    undo: Vec<Circuit>,
    /// Historie pro „Znovu“.
    ///
    /// [en] History for “Redo”.
    redo: Vec<Circuit>,
    /// Zvyšuje se při každé změně obvodu (pro přepočet pravdivostní tabulky apod.).
    ///
    /// [en] Increased on every change of the circuit (to recompute the truth table etc.).
    pub revision: u64,
    /// Režim úprav nebo režim simulace.
    ///
    /// [en] Edit mode or simulation mode.
    pub mode: Mode,
    /// Běží v režimu simulace čas (není pozastavená)? V režimu úprav je vždy `false`.
    ///
    /// [en] Does time run in simulation mode (not paused)? Always `false` in edit mode.
    pub running: bool,
    /// Rychlost simulace: násobek skutečného času (×0,1 zpomalí pro výklad, ×10 zrychlí
    /// dlouhé čítače). Nastavuje ji aplikace, do souboru se neukládá.
    ///
    /// [en] Simulation speed: a multiple of real time (×0.1 slows down for explaining,
    /// ×10 speeds up long counters). Set by the application, not saved in the file.
    pub speed: f64,
    /// Posun fáze jednotlivých hodin (s) – po změně frekvence hodiny navážou plynule,
    /// bez skoku. Hodnota hodin se počítá z `(time - posun) / půlperioda`.
    ///
    /// [en] Phase offset of individual clocks (s) – after a frequency change the clock
    /// continues smoothly, without a jump. The clock value is computed from
    /// `(time - offset) / half period`.
    clock_offsets: HashMap<CompId, f64>,
    /// Povolit úpravy schématu i v režimu simulace (volba pro pokročilé, výchozí vypnuto).
    ///
    /// [en] Allow editing the schematic in simulation mode too (advanced option, off by default).
    pub edit_while_simulating: bool,
    /// Čas časového diagramu v sekundách; běží jen při spuštěné simulaci.
    ///
    /// [en] Time of the timing diagram in seconds; runs only while the simulation runs.
    pub time: f64,
    /// Zaznamenané signály pro časový diagram.
    ///
    /// [en] Recorded signals for the timing diagram.
    pub signals: Vec<Signal>,
    /// Součástka, na kterou má editor schématu zaměřit pohled („Najít ve schématu“).
    ///
    /// [en] Component the schematic editor should centre the view on ("Find in schematic").
    pub focus_request: Option<CompId>,
    /// Součástka vybraná v knihovně, která se má umístit do schématu.
    ///
    /// [en] Component chosen in the library that is to be placed in the schematic.
    pub pending_place: Option<ComponentKind>,
}

impl Project {
    /// Projekt s jedním obvodem bez podobvodů.
    ///
    /// [en] Project with one circuit and no subcircuits.
    pub fn new(circuit: Circuit) -> Self {
        Self::with_subcircuits(circuit, Vec::new())
    }

    /// Projekt s hlavním obvodem a definicemi podobvodů.
    ///
    /// [en] Project with a main circuit and subcircuit definitions.
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
            mode: Mode::Edit,
            running: false,
            edit_while_simulating: false,
            speed: 1.0,
            clock_offsets: HashMap::new(),
            time: 0.0,
            signals: Vec::new(),
            pending_place: None,
            focus_request: None,
        };
        p.record_signals();
        p
    }

    /// Načte projekt ze souboru .logicflow.
    ///
    /// [en] Loads a project from a .logicflow file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut p = Self::from_text(&text)?;
        p.path = Some(path.to_path_buf());
        Ok(p)
    }

    /// Vytvoří projekt z textu souboru .logicflow (bez práce se soubory – použije
    /// i budoucí webová verze, která dostane obsah souboru z prohlížeče).
    ///
    /// [en] Creates a project from the text of a .logicflow file (no file access – also
    /// used by the future web version, which gets the file content from the browser).
    pub fn from_text(text: &str) -> Result<Self, String> {
        let (mut circuit, mut subs) = ProjectFile::from_json(text)?;
        // Počty vývodů instancí podle aktuálních definic.
        // [en] Pin counts of instances according to the current definitions.
        let defs = subs.clone();
        sync_instances(&mut circuit, &defs);
        for d in &mut subs {
            sync_instances(&mut d.circuit, &defs);
        }
        Ok(Self::with_subcircuits(circuit, subs))
    }

    /// Uloží projekt (hlavní obvod i všechny podobvody) do souboru.
    ///
    /// [en] Saves the project (main circuit and all subcircuits) to a file.
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.to_text()).map_err(|e| e.to_string())?;
        self.path = Some(path.to_path_buf());
        self.title = None;
        self.modified = false;
        Ok(())
    }

    /// Text souboru .logicflow s hlavním obvodem i všemi podobvody (bez práce se
    /// soubory – webová verze ho nabídne ke stažení).
    ///
    /// [en] Text of the .logicflow file with the main circuit and all subcircuits (no file
    /// access – the web version offers it as a download).
    pub fn to_text(&self) -> String {
        let (main, subs) = self.snapshot();
        ProjectFile::to_json(&main, &subs)
    }

    /// Název do titulku okna.
    ///
    /// [en] Name for the window title.
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
    ///
    /// [en] Main circuit and subcircuit definitions including the sheet being edited.
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
    ///
    /// [en] Subcircuit definition.
    pub fn subcircuit(&self, id: u32) -> Option<&SubcircuitDef> {
        self.subcircuits.iter().find(|d| d.id == id)
    }

    /// Název právě upravovaného listu (`None` = hlavní obvod).
    ///
    /// [en] Name of the sheet being edited (`None` = main circuit).
    pub fn sheet_name(&self) -> Option<String> {
        self.sheet
            .and_then(|id| self.subcircuit(id))
            .map(|d| d.name.clone())
    }

    /// Lze do právě upravovaného listu vložit instanci podobvodu `id`?
    /// (Podobvod nesmí obsahovat sám sebe, ani nepřímo.)
    ///
    /// [en] Can an instance of subcircuit `id` be placed in the sheet being edited?
    /// (A subcircuit must not contain itself, not even indirectly.)
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
    ///
    /// [en] Writes the subcircuit being edited back into the definitions.
    fn store_sheet(&mut self) {
        if let Some(id) = self.sheet
            && let Some(d) = self.subcircuits.iter_mut().find(|d| d.id == id)
        {
            d.circuit = self.circuit.clone();
        }
    }

    /// Otevře list k úpravám: hlavní obvod (`None`) nebo podobvod.
    ///
    /// [en] Opens a sheet for editing: the main circuit (`None`) or a subcircuit.
    pub fn open_sheet(&mut self, target: Option<u32>) {
        self.open_sheet_inner(target);
        self.sheet_stack.clear();
    }

    /// Otevře podobvod a zapamatuje si, odkud se přišlo (Tab zpět).
    ///
    /// [en] Opens a subcircuit and remembers where it came from (Tab back).
    pub fn enter_subcircuit(&mut self, id: u32) {
        let from = self.sheet;
        if from != Some(id) && self.subcircuit(id).is_some() {
            self.open_sheet_inner(Some(id));
            self.sheet_stack.push(from);
        }
    }

    /// Vrátí se o úroveň výš (do nadřazeného listu). Vrací `false`, pokud není kam.
    ///
    /// [en] Goes one level up (to the parent sheet). Returns `false` if there is nowhere to go.
    pub fn leave_subcircuit(&mut self) -> bool {
        if self.sheet.is_none() {
            return false;
        }
        let back = self.sheet_stack.pop().flatten();
        self.open_sheet_inner(back);
        true
    }

    /// Přepne list: uloží rozpracovaný, odloží nebo obnoví hlavní obvod a postaví simulátor.
    ///
    /// [en] Switches the sheet: stores the one being edited, puts aside or restores the main circuit and builds the simulator.
    fn open_sheet_inner(&mut self, target: Option<u32>) {
        if target == self.sheet || target.is_some_and(|id| self.subcircuit(id).is_none()) {
            return;
        }
        self.store_sheet();
        // Rozhraní upravených podobvodů se promítne do instancí.
        // [en] The interface of edited subcircuits is reflected in the instances.
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
    ///
    /// [en] Creates a new empty subcircuit and opens it. Returns its id.
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

    /// Přejmenuje podobvod.
    ///
    /// [en] Renames a subcircuit.
    pub fn rename_subcircuit(&mut self, id: u32, name: String) {
        if let Some(d) = self.subcircuits.iter_mut().find(|d| d.id == id) {
            d.name = name;
            self.touch();
        }
    }

    /// Je podobvod někde použitý (v hlavním obvodu nebo jiném podobvodu)?
    ///
    /// [en] Is the subcircuit used anywhere (in the main circuit or another subcircuit)?
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
    ///
    /// [en] Deletes an unused subcircuit. If it is open, goes back to the main circuit.
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
    ///
    /// [en] Saves the state for “Undo”. Called before every edit.
    pub fn checkpoint(&mut self) {
        self.push_undo(self.circuit.clone());
    }

    /// Přidá stav do historie „Zpět“ (nejvýš UNDO_LIMIT kroků) a vymaže „Znovu“.
    ///
    /// [en] Adds a state to the “Undo” history (at most UNDO_LIMIT steps) and clears “Redo”.
    pub fn push_undo(&mut self, state: Circuit) {
        self.undo.push(state);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Je co vracet?
    ///
    /// [en] Is there anything to undo?
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Je co opakovat?
    ///
    /// [en] Is there anything to redo?
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Vrátí poslední úpravu.
    ///
    /// [en] Undoes the last edit.
    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            let current = std::mem::replace(&mut self.circuit, prev);
            self.keep_switch_states(&current);
            self.redo.push(current);
            self.after_structure_change();
        }
    }

    /// Znovu provede vrácenou úpravu.
    ///
    /// [en] Redoes the undone edit.
    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            let current = std::mem::replace(&mut self.circuit, next);
            self.keep_switch_states(&current);
            self.undo.push(current);
            self.after_structure_change();
        }
    }

    /// Přepnutí přepínače není úprava schématu – „Zpět“ ho nemá vracet.
    ///
    /// [en] Flipping a switch is not a schematic edit – “Undo” should not revert it.
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
    ///
    /// [en] After a wiring change: rebuilds the simulator (keeping the state) and cleans up the selection.
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
    ///
    /// [en] A change that does not alter the wiring (label…).
    pub fn touch(&mut self) {
        self.modified = true;
        self.revision += 1;
    }

    /// Smaže vybrané součástky a vodiče (s uložením pro „Zpět“).
    ///
    /// [en] Deletes the selected components and wires (saving for “Undo”).
    pub fn delete_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        self.checkpoint();
        for id in self.selection.comp_vec() {
            self.circuit.remove_component(id);
        }
        for id in self.selection.wire_vec() {
            self.circuit.remove_wire(id);
        }
        self.selection.clear();
        self.after_structure_change();
    }

    /// Otočí vybrané součástky o 90° (s uložením pro „Zpět“).
    ///
    /// [en] Rotates the selected components by 90° (saving for “Undo”).
    pub fn rotate_selection(&mut self) {
        if self.selection.comps.is_empty() {
            return;
        }
        self.checkpoint();
        for id in self.selection.comp_vec() {
            if let Some(c) = self.circuit.component_mut(id) {
                c.rotation = c.rotation.next();
            }
        }
        self.after_structure_change();
    }

    /// Změní druh součástky (frekvence hodin, počet vstupů, strana vývodů…) s uložením
    /// pro „Zpět“. Stav obvodu se zachová.
    ///
    /// [en] Changes the kind of a component (clock frequency, number of inputs, pin
    /// side…) saving for "Undo". The circuit state is kept.
    pub fn set_kind(&mut self, id: CompId, kind: ComponentKind) {
        if self.circuit.component(id).is_none_or(|c| c.kind == kind) {
            return;
        }
        self.checkpoint();
        if let Some(c) = self.circuit.component_mut(id) {
            c.kind = kind;
        }
        self.after_structure_change();
    }

    /// Přidá součástku na bod mřížky, vybere ji (s uložením pro „Zpět“).
    ///
    /// [en] Adds a component at a grid point and selects it (saving for "Undo").
    pub fn add_component_at(
        &mut self,
        kind: ComponentKind,
        pos: Point,
        rotation: Rotation,
    ) -> CompId {
        self.checkpoint();
        let id = self.circuit.add_component(kind, pos);
        if let Some(c) = self.circuit.component_mut(id) {
            c.rotation = rotation;
        }
        self.selection.clear();
        self.selection.comps.insert(id);
        self.after_structure_change();
        id
    }

    /// Všechny vodiče sítě, do které patří vodič `wire` (spojené vodiče).
    ///
    /// [en] All wires of the net the wire `wire` belongs to (connected wires).
    pub fn net_wires(&self, wire: WireId) -> Vec<WireId> {
        let nets = &self.sim.netlist().wire_net;
        let Some(net) = nets.get(&wire) else {
            return vec![wire];
        };
        let mut wires: Vec<WireId> = nets
            .iter()
            .filter(|(_, n)| *n == net)
            .map(|(w, _)| *w)
            .collect();
        wires.sort();
        wires
    }

    /// Smaže vodiče (s uložením pro „Zpět“).
    ///
    /// [en] Deletes wires (saving for "Undo").
    pub fn delete_wires(&mut self, wires: &[WireId]) {
        if wires.is_empty() {
            return;
        }
        self.checkpoint();
        for &w in wires {
            self.circuit.remove_wire(w);
        }
        self.after_structure_change();
    }

    /// Výběr jako text pro schránku: soubor .logicflow s vybranými součástkami,
    /// vodiči mezi nimi a definicemi použitých podobvodů. `None`, když není nic vybráno.
    ///
    /// [en] The selection as clipboard text: a .logicflow file with the selected
    /// components, the wires between them and the definitions of the subcircuits used.
    /// `None` when nothing is selected.
    pub fn copy_selection(&self) -> Option<String> {
        if self.selection.is_empty() {
            return None;
        }
        let clip = self
            .circuit
            .extract(&self.selection.comp_vec(), &self.selection.wire_vec());
        let (_, defs) = self.snapshot();
        let mut needed: Vec<SubcircuitDef> = Vec::new();
        let mut stack: Vec<u32> = subcircuit_ids(&clip);
        while let Some(id) = stack.pop() {
            if needed.iter().any(|d| d.id == id) {
                continue;
            }
            if let Some(d) = defs.iter().find(|d| d.id == id) {
                stack.extend(subcircuit_ids(&d.circuit));
                needed.push(d.clone());
            }
        }
        needed.sort_by_key(|d| d.id);
        Some(ProjectFile::to_json(&clip, &needed))
    }

    /// Vloží obvod ze schránky (text souboru .logicflow). Se `at` se vloží tak, aby
    /// střed vkládaných součástek ležel v tomto bodě; bez něj o 2 body vedle původní
    /// polohy. Podobvody, které projekt ještě nemá, se přidají (stejné se použijí
    /// znovu, kolize názvů se přejmenují); podobvod, který by obsahoval sám sebe,
    /// se vynechá. Vložené prvky se vyberou. Vrací `false`, když text není obvod.
    ///
    /// [en] Pastes a circuit from the clipboard (the text of a .logicflow file). With `at`
    /// it is placed so that the centre of the pasted components lies at this point;
    /// without it 2 points next to the original position. Subcircuits the project does
    /// not have yet are added (identical ones are reused, name clashes are renamed); a
    /// subcircuit that would contain itself is left out. The pasted items get selected.
    /// Returns `false` when the text is not a circuit.
    pub fn paste_text(&mut self, text: &str, at: Option<Point>) -> bool {
        let Ok((mut clip, clip_defs)) = ProjectFile::from_json(text) else {
            return false;
        };
        if clip.components.is_empty() && clip.wires.is_empty() {
            return false;
        }
        self.checkpoint();
        // Sloučení definic podobvodů.
        // [en] Merging subcircuit definitions.
        self.store_sheet();
        let mut map: HashMap<u32, u32> = HashMap::new();
        let mut added: Vec<SubcircuitDef> = Vec::new();
        for d in &clip_defs {
            let same = self.subcircuits.iter().find(|e| {
                e.name == d.name
                    && e.circuit.components == d.circuit.components
                    && e.circuit.wires == d.circuit.wires
            });
            if let Some(e) = same {
                map.insert(d.id, e.id);
                continue;
            }
            let id = self
                .subcircuits
                .iter()
                .chain(&added)
                .map(|e| e.id + 1)
                .max()
                .unwrap_or(1);
            let taken = |n: &str| self.subcircuits.iter().chain(&added).any(|e| e.name == n);
            let name = if taken(&d.name) {
                (2..)
                    .map(|k| format!("{} ({k})", d.name))
                    .find(|n| !taken(n))
                    .unwrap_or_default()
            } else {
                d.name.clone()
            };
            map.insert(d.id, id);
            added.push(SubcircuitDef {
                id,
                name,
                circuit: d.circuit.clone(),
            });
        }
        for d in &mut added {
            remap_subcircuits(&mut d.circuit, &map);
        }
        remap_subcircuits(&mut clip, &map);
        self.subcircuits.extend(added);
        // Podobvod nesmí obsahovat sám sebe.
        // [en] A subcircuit must not contain itself.
        clip.components.retain(|c| match c.kind {
            ComponentKind::Subcircuit { id, .. } => self.can_place_subcircuit(id),
            _ => true,
        });
        let defs = self.subcircuits.clone();
        sync_instances(&mut clip, &defs);
        // Posun: střed součástek (nebo vodičů) do bodu `at`.
        // [en] Offset: the centre of the components (or wires) to the point `at`.
        let (dx, dy) = match at {
            Some(p) => {
                let pts: Vec<Point> = if clip.components.is_empty() {
                    clip.wires.iter().flat_map(|w| [w.a, w.b]).collect()
                } else {
                    clip.components.iter().map(|c| c.pos).collect()
                };
                let (min_x, max_x) = (
                    pts.iter().map(|q| q.x).min().unwrap_or(0),
                    pts.iter().map(|q| q.x).max().unwrap_or(0),
                );
                let (min_y, max_y) = (
                    pts.iter().map(|q| q.y).min().unwrap_or(0),
                    pts.iter().map(|q| q.y).max().unwrap_or(0),
                );
                (p.x - (min_x + max_x) / 2, p.y - (min_y + max_y) / 2)
            }
            None => (2, 2),
        };
        let (comps, wires) = self.circuit.insert(&clip, dx, dy);
        self.selection.comps = comps.into_iter().collect();
        self.selection.wires = wires.into_iter().collect();
        self.after_structure_change();
        true
    }

    /// Vyjmout: zkopíruje výběr jako text a smaže ho.
    ///
    /// [en] Cut: copies the selection as text and deletes it.
    pub fn cut_selection(&mut self) -> Option<String> {
        let text = self.copy_selection()?;
        self.delete_selection();
        Some(text)
    }

    /// Vybere všechny součástky a vodiče otevřeného listu.
    ///
    /// [en] Selects all components and wires of the open sheet.
    pub fn select_all(&mut self) {
        self.selection.comps = self.circuit.components.iter().map(|c| c.id).collect();
        self.selection.wires = self.circuit.wires.iter().map(|w| w.id).collect();
    }

    /// Id definice podobvodu, je-li vybraná právě jedna instance podobvodu.
    ///
    /// [en] Id of the subcircuit definition if exactly one subcircuit instance is selected.
    pub fn selected_subcircuit(&self) -> Option<u32> {
        let id = self.selection.single_comp()?;
        match self.circuit.component(id)?.kind {
            ComponentKind::Subcircuit { id, .. } => Some(id),
            _ => None,
        }
    }

    /// Tab: otevře vybraný podobvod, jinak se vrátí o úroveň výš.
    ///
    /// [en] Tab: opens the selected subcircuit, otherwise goes one level up.
    pub fn toggle_subcircuit(&mut self) {
        match self.selected_subcircuit() {
            Some(id) => self.enter_subcircuit(id),
            None => {
                self.leave_subcircuit();
            }
        }
    }

    /// Přepne přepínač (bez záznamu do historie úprav – jde o „hraní“ s obvodem).
    ///
    /// [en] Toggles a switch (without recording it in the edit history – it is “playing” with the circuit).
    pub fn toggle_switch(&mut self, id: CompId) {
        let Some(c) = self.circuit.component_mut(id) else {
            return;
        };
        if let ComponentKind::Switch { on } = &mut c.kind {
            *on = !*on;
            let v = (*on).into();
            self.sim.set_source(id, v);
            self.modified = true;
            self.after_input_change();
        }
    }

    /// Stiskne nebo uvolní tlačítko v obvodu.
    ///
    /// [en] Presses or releases a push button in the circuit.
    pub fn set_button(&mut self, id: CompId, pressed: bool) {
        let Some(c) = self.circuit.component_mut(id) else {
            return;
        };
        if matches!(c.kind, ComponentKind::Button) && c.pressed != pressed {
            c.pressed = pressed;
            self.sim.set_source(id, pressed.into());
            self.after_input_change();
        }
    }

    /// Po změně vstupu (přepínač, tlačítko): v režimu úprav se obvod hned ustálí,
    /// při pozastavené simulaci se udělá jen jeden krok (změna se projeví, ale dál se
    /// nešíří – krokování), při běžící simulaci se o šíření postará `update`.
    ///
    /// [en] After an input change (switch, button): in edit mode the circuit settles at
    /// once, with a paused simulation only one step is made (the change shows but does
    /// not propagate further – stepping), with a running simulation `update` takes care
    /// of propagation.
    fn after_input_change(&mut self) {
        match (self.mode, self.running) {
            (Mode::Edit, _) => {
                self.sim.run(EVENT_BUDGET);
            }
            (Mode::Simulation, false) => {
                self.sim.step();
            }
            (Mode::Simulation, true) => {}
        }
    }

    /// Je projekt v režimu simulace?
    ///
    /// [en] Is the project in simulation mode?
    pub fn is_simulating(&self) -> bool {
        self.mode == Mode::Simulation
    }

    /// Jde schéma upravovat? V režimu úprav ano, v režimu simulace jen se zapnutou
    /// volbou „Povolit úpravy za běhu simulace“.
    ///
    /// [en] Can the schematic be edited? In edit mode yes, in simulation mode only with
    /// the "Allow editing while simulating" option switched on.
    pub fn can_edit(&self) -> bool {
        self.mode == Mode::Edit || self.edit_while_simulating
    }

    /// Spustit (F5): přepne do režimu simulace a pustí čas.
    ///
    /// [en] Run (F5): switches to simulation mode and starts time.
    pub fn start_simulation(&mut self) {
        self.mode = Mode::Simulation;
        self.running = true;
    }

    /// Zastavit (Esc, Shift+F5): návrat do režimu úprav. Stav obvodu (klopné obvody,
    /// čítače, LED, časový diagram) zůstává – vynuluje ho jen Restart.
    ///
    /// [en] Stop (Esc, Shift+F5): back to edit mode. The circuit state (flip-flops,
    /// counters, LEDs, timing diagram) stays – only Restart clears it.
    pub fn stop_simulation(&mut self) {
        self.mode = Mode::Edit;
        self.running = false;
    }

    /// Mezerník: v režimu úprav spustí simulaci, v režimu simulace ji pozastaví nebo
    /// znovu rozběhne.
    ///
    /// [en] Space: in edit mode starts the simulation, in simulation mode pauses or
    /// resumes it.
    pub fn toggle_run(&mut self) {
        match self.mode {
            Mode::Edit => self.start_simulation(),
            Mode::Simulation => self.running = !self.running,
        }
    }

    /// Volá se každý snímek. Čas (hodiny, časový diagram) běží jen v režimu simulace;
    /// v režimu úprav se obvod jen ustaluje po změnách.
    ///
    /// [en] Called every frame. Time (clocks, timing diagram) runs only in simulation
    /// mode; in edit mode the circuit only settles after changes.
    pub fn update(&mut self, dt: f64) {
        match (self.mode, self.running) {
            (Mode::Simulation, true) => {
                let prev = self.time;
                self.time += dt * self.speed;
                self.advance_clocks(prev, self.time);
                self.sim.run(EVENT_BUDGET);
            }
            (Mode::Edit, _) => {
                self.sim.run(EVENT_BUDGET);
            }
            (Mode::Simulation, false) => {}
        }
        self.record_signals();
    }

    /// Jeden takt simulace (šipka vpravo). Z režimu úprav přejde do pozastavené
    /// simulace. Časový diagram se posune o čtvrt sekundy, aby byl každý krok vidět.
    ///
    /// [en] One simulation tick (right arrow). From edit mode it switches to a paused
    /// simulation. The timing diagram moves by a quarter of a second so that every
    /// step is visible.
    pub fn step(&mut self) {
        self.mode = Mode::Simulation;
        self.running = false;
        self.time += 0.25;
        self.drive_clocks();
        self.sim.step();
        self.record_signals();
    }

    /// Generátory hodin: id, délka půlperiody (s) a posun fáze (s).
    ///
    /// [en] Clock generators: id, half period length (s) and phase offset (s).
    fn clocks(&self) -> Vec<(CompId, f64, f64)> {
        self.circuit
            .components
            .iter()
            .filter_map(|c| match c.kind {
                ComponentKind::Clock { period_ms } => Some((
                    c.id,
                    half_period(period_ms),
                    self.clock_offsets.get(&c.id).copied().unwrap_or(0.0),
                )),
                _ => None,
            })
            .collect()
    }

    /// Má otevřený list nějaký generátor hodin?
    ///
    /// [en] Does the open sheet have any clock generator?
    pub fn has_clocks(&self) -> bool {
        self.circuit
            .components
            .iter()
            .any(|c| matches!(c.kind, ComponentKind::Clock { .. }))
    }

    /// Čas (s) příští hrany některého generátoru hodin; s `rising_only` jen náběžné
    /// hrany (hodiny přejdou do 1 – od jedné náběžné hrany ke druhé je celá perioda).
    ///
    /// [en] Time (s) of the next edge of any clock generator; with `rising_only` only
    /// rising edges (the clock goes to 1 – from one rising edge to the next is a whole period).
    pub fn next_clock_edge(&self, rising_only: bool) -> Option<f64> {
        self.clocks()
            .into_iter()
            .map(|(_, half, off)| {
                let mut k = ((self.time - off) / half).floor() as i64 + 1;
                if rising_only && k.rem_euclid(2) == 0 {
                    k += 1;
                }
                k as f64 * half + off
            })
            .min_by(f64::total_cmp)
    }

    /// Za jak dlouho (s simulovaného času) nastane příští hrana některého generátoru hodin.
    ///
    /// [en] In how long (s of simulated time) the next edge of any clock generator occurs.
    pub fn time_to_next_clock_edge(&self) -> Option<f64> {
        self.next_clock_edge(false).map(|t| t - self.time)
    }

    /// Krok na příští hranu hodin (Shift+šipka vpravo), s `rising_only` na příští
    /// náběžnou hranu – o celou periodu (Ctrl+šipka vpravo). Obvod se po hraně ustálí.
    /// Z režimu úprav přejde do pozastavené simulace.
    ///
    /// [en] Step to the next clock edge (Shift+right arrow), with `rising_only` to the next
    /// rising edge – a whole period (Ctrl+right arrow). The circuit settles after the
    /// edge. From edit mode it switches to a paused simulation.
    pub fn step_to_edge(&mut self, rising_only: bool) {
        let Some(edge) = self.next_clock_edge(rising_only) else {
            return;
        };
        self.mode = Mode::Simulation;
        self.running = false;
        // Kousek za hranu, aby zaokrouhlení nevrátilo čas před ni.
        // [en] A tiny bit past the edge so that rounding does not put the time before it.
        let to = edge + 1e-9;
        let from = self.time;
        self.advance_clocks(from, to);
        self.time = to;
        self.sim.run(EVENT_BUDGET);
        self.record_signals();
    }

    /// Nastaví periodu hodin (ms) bez skoku fáze: hodiny pokračují z místa, kde byly
    /// (stejná úroveň i rozpracovaná část půlperiody). Se `undo` se uloží stav pro
    /// „Zpět“ (při tažení posuvníkem jen na začátku).
    ///
    /// [en] Sets the clock period (ms) without a phase jump: the clock continues from
    /// where it was (the same level and the elapsed part of the half period). With
    /// `undo` the state is saved for "Undo" (when dragging a slider only at the start).
    pub fn set_clock_period(&mut self, id: CompId, period_ms: u32, undo: bool) {
        let Some(ComponentKind::Clock { period_ms: old }) =
            self.circuit.component(id).map(|c| c.kind)
        else {
            return;
        };
        let period_ms = period_ms.clamp(MIN_CLOCK_PERIOD_MS, MAX_CLOCK_PERIOD_MS);
        if old == period_ms {
            return;
        }
        let off = self.clock_offsets.get(&id).copied().unwrap_or(0.0);
        let cycles = (self.time - off) / half_period(old);
        self.clock_offsets
            .insert(id, self.time - cycles * half_period(period_ms));
        if undo {
            self.checkpoint();
        }
        if let Some(c) = self.circuit.component_mut(id) {
            c.kind = ComponentKind::Clock { period_ms };
        }
        self.after_structure_change();
    }

    /// Nastaví výstupy generátorů hodin podle aktuálního času projektu.
    ///
    /// [en] Sets the outputs of the clock generators according to the current project time.
    fn drive_clocks(&mut self) {
        for (id, half, off) in self.clocks() {
            let k = ((self.time - off) / half).floor() as i64;
            let v = LogicValue::from_bool(k.rem_euclid(2) == 1);
            if self.sim.output(id) != Some(v) {
                self.sim.set_source(id, v);
            }
        }
    }

    /// Projde všechny hrany hodin mezi `from` a `to` v časovém pořadí a po každé
    /// nechá obvod ustálit – žádná hrana se neztratí ani při pomalém snímku.
    ///
    /// [en] Goes through all clock edges between `from` and `to` in time order and lets
    /// the circuit settle after each – no edge is lost even with a slow frame.
    fn advance_clocks(&mut self, from: f64, to: f64) {
        let mut edges: Vec<(f64, CompId, LogicValue)> = Vec::new();
        for (id, half, off) in self.clocks() {
            let first = ((from - off) / half).floor() as i64 + 1;
            let last = ((to - off) / half).floor() as i64;
            // Při extrémně dlouhém snímku stačí posledních pár hran.
            // [en] With an extremely long frame the last few edges are enough.
            for k in first.max(last - 64)..=last {
                edges.push((
                    k as f64 * half + off,
                    id,
                    LogicValue::from_bool(k.rem_euclid(2) == 1),
                ));
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
    ///
    /// [en] Simulation restart: flip-flops return to their initial state, clocks to 0.
    pub fn reset_simulation(&mut self) {
        self.sim = Simulator::with_subcircuits(&self.circuit, &self.subcircuits, None);
        self.time = 0.0;
        self.clock_offsets.clear();
        self.signals.clear();
        self.sim.run(EVENT_BUDGET);
        self.record_signals();
    }

    /// Hodnota zobrazovaná u součástky v časovém diagramu.
    ///
    /// [en] Value shown for a component in the timing diagram.
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
    ///
    /// [en] Circuit inputs and outputs sorted by label.
    pub fn io_components(&self) -> (Vec<CompId>, Vec<CompId>) {
        let (ins, outs) = self.circuit.io();
        (
            ins.iter().map(|c| c.id).collect(),
            outs.iter().map(|c| c.id).collect(),
        )
    }

    /// Zapíše změny hodnot hodin, vstupů a výstupů do záznamu signálů.
    ///
    /// [en] Records value changes of clocks, inputs and outputs into the signal recording.
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

/// Nejkratší perioda hodin (ms) – 100 Hz.
///
/// [en] Shortest clock period (ms) – 100 Hz.
pub const MIN_CLOCK_PERIOD_MS: u32 = 10;
/// Nejdelší perioda hodin (ms) – 0,1 Hz.
///
/// [en] Longest clock period (ms) – 0.1 Hz.
pub const MAX_CLOCK_PERIOD_MS: u32 = 10_000;
/// Rychlé volby frekvence hodin (Hz).
///
/// [en] Quick choices of the clock frequency (Hz).
pub const CLOCK_PRESETS_HZ: [f32; 6] = [0.2, 0.5, 1.0, 2.0, 5.0, 10.0];
/// Volby rychlosti simulace (násobek skutečného času).
///
/// [en] Choices of the simulation speed (multiple of real time).
pub const SPEED_CHOICES: [f64; 7] = [0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0];

/// Perioda hodin (ms) pro frekvenci v Hz, oříznutá na povolený rozsah.
///
/// [en] Clock period (ms) for a frequency in Hz, clamped to the allowed range.
pub fn period_for_hz(hz: f32) -> u32 {
    ((1000.0 / hz.max(0.001)).round() as u32).clamp(MIN_CLOCK_PERIOD_MS, MAX_CLOCK_PERIOD_MS)
}

/// Délka půlperiody (s) pro periodu v ms.
///
/// [en] Half period length (s) for a period in ms.
fn half_period(period_ms: u32) -> f64 {
    f64::from(period_ms.max(2)) / 2000.0
}

/// Id definic podobvodů, jejichž instance obvod přímo obsahuje.
///
/// [en] Ids of the subcircuit definitions whose instances the circuit directly contains.
fn subcircuit_ids(c: &Circuit) -> Vec<u32> {
    c.components
        .iter()
        .filter_map(|k| match k.kind {
            ComponentKind::Subcircuit { id, .. } => Some(id),
            _ => None,
        })
        .collect()
}

/// Přečísluje odkazy instancí podobvodů podle mapy staré id → nové id.
///
/// [en] Renumbers the references of subcircuit instances according to the map old id → new id.
fn remap_subcircuits(c: &mut Circuit, map: &HashMap<u32, u32>) {
    for k in &mut c.components {
        if let ComponentKind::Subcircuit { id, .. } = &mut k.kind
            && let Some(&new) = map.get(id)
        {
            *id = new;
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
        // [en] Inside: switch → LED (interface 1 input, 1 output).
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
        // [en] Saving from a subcircuit must write the main circuit too.
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
        p.start_simulation();
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
        // [en] Clock 1 Hz → T flip-flop (T = 1) → LED: the LED changes every second.
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Clock { period_ms: 1000 }, Point::new(0, 1));
        c.add_component(ComponentKind::Constant { value: true }, Point::new(0, -1));
        let ff = c.add_component(
            ComponentKind::flip_flop(logicflow_core::FlipFlopKind::T),
            Point::new(10, 0),
        );
        // výstup hodin (3,1) → vstup C (7,1); konstanta (2,-1) → vstup T (7,-1)
        // [en] clock output (3,1) → input C (7,1); constant (2,-1) → input T (7,-1)
        c.add_wire(Point::new(3, 1), Point::new(7, 1));
        c.add_wire(Point::new(2, -1), Point::new(7, -1));
        let mut p = Project::new(c);
        p.start_simulation();
        let mut states = Vec::new();
        for _ in 0..8 {
            p.update(0.5);
            states.push(p.sim.flip_flop_state(ff).unwrap());
        }
        use LogicValue::*;
        // Náběžná hrana hodin každou sekundu (v čase 0,5; 1,5; …) překlopí T.
        // [en] The rising clock edge every second (at 0.5; 1.5; …) toggles T.
        assert_eq!(states, [One, One, Zero, Zero, One, One, Zero, Zero]);
        p.reset_simulation();
        assert_eq!(p.sim.flip_flop_state(ff), Some(Zero));
        assert_eq!(p.signals[0].label, "CLK", "hodiny jsou v diagramu první");
    }

    #[test]
    fn next_clock_edge() {
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Clock { period_ms: 1000 }, Point::new(0, 1));
        c.add_component(ComponentKind::Clock { period_ms: 300 }, Point::new(0, 5));
        let mut p = Project::new(c);
        // Půlperiody 0,5 s a 0,15 s → nejbližší hrana v čase 0,15 s.
        // [en] Half periods 0.5 s and 0.15 s → the nearest edge at 0.15 s.
        assert!((p.time_to_next_clock_edge().unwrap() - 0.15).abs() < 1e-9);
        p.start_simulation();
        p.update(0.4);
        // Další hrany v 0,45 s (rychlé hodiny) a 0,5 s → za 0,05 s.
        // [en] Next edges at 0.45 s (fast clock) and 0.5 s → in 0.05 s.
        assert!((p.time_to_next_clock_edge().unwrap() - 0.05).abs() < 1e-9);
        assert_eq!(
            Project::new(Circuit::default()).time_to_next_clock_edge(),
            None
        );
    }

    /// Obvod hodiny 1 Hz → klopný obvod T (T = 1); vrací projekt a id klopného obvodu.
    ///
    /// [en] Circuit clock 1 Hz → T flip-flop (T = 1); returns the project and the flip-flop id.
    fn clocked_t() -> (Project, CompId) {
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Clock { period_ms: 1000 }, Point::new(0, 1));
        c.add_component(ComponentKind::Constant { value: true }, Point::new(0, -1));
        let ff = c.add_component(
            ComponentKind::flip_flop(logicflow_core::FlipFlopKind::T),
            Point::new(10, 0),
        );
        c.add_wire(Point::new(3, 1), Point::new(7, 1));
        c.add_wire(Point::new(2, -1), Point::new(7, -1));
        (Project::new(c), ff)
    }

    #[test]
    fn edit_mode_settles_logic_but_time_stands() {
        let mut p = small();
        let a = p.circuit.components[0].id;
        let y = p.circuit.components[1].id;
        assert_eq!(p.mode, Mode::Edit);
        assert!(p.can_edit());
        // Přepínač v režimu úprav: LED svítí hned, bez spuštění simulace.
        // [en] A switch in edit mode: the LED lights up at once, without running the simulation.
        p.toggle_switch(a);
        assert_eq!(p.signal_value(y), LogicValue::One);
        p.update(1.0);
        assert_eq!(p.time, 0.0, "v režimu úprav čas stojí");
        // Hodiny v režimu úprav netikají.
        // [en] Clocks do not tick in edit mode.
        let (mut p, ff) = clocked_t();
        for _ in 0..6 {
            p.update(0.5);
        }
        assert_eq!(p.sim.flip_flop_state(ff), Some(LogicValue::Zero));
    }

    #[test]
    fn stop_keeps_state_and_start_resumes() {
        let (mut p, ff) = clocked_t();
        p.start_simulation();
        assert!(p.is_simulating() && p.running && !p.can_edit());
        p.update(0.5);
        p.update(0.5);
        assert_eq!(p.sim.flip_flop_state(ff), Some(LogicValue::One));
        let t = p.time;
        p.stop_simulation();
        assert!(!p.is_simulating() && !p.running && p.can_edit());
        p.update(5.0);
        // Po zastavení zůstává stav i čas.
        // [en] After stopping, the state and the time stay.
        assert_eq!(p.sim.flip_flop_state(ff), Some(LogicValue::One));
        assert_eq!(p.time, t);
        // Mezerník v úpravách spustí, v simulaci pozastaví.
        // [en] Space starts in edit mode, pauses in simulation mode.
        p.toggle_run();
        assert!(p.is_simulating() && p.running);
        p.toggle_run();
        assert!(p.is_simulating() && !p.running);
        // Volba pro pokročilé povolí úpravy i v simulaci.
        // [en] The advanced option allows editing in simulation too.
        p.edit_while_simulating = true;
        assert!(p.can_edit());
    }

    #[test]
    fn step_from_edit_mode_pauses_simulation() {
        let (mut p, _) = clocked_t();
        p.step();
        assert!(p.is_simulating() && !p.running);
        assert!((p.time - 0.25).abs() < 1e-9);
    }

    #[test]
    fn context_menu_operations() {
        // Přepínač → LED, vodič ze dvou úseků v jedné síti a jeden samostatný vodič.
        // [en] Switch → LED, a wire of two segments in one net and one separate wire.
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        c.add_component(ComponentKind::Led, Point::new(8, 0));
        let w1 = c.add_wire(Point::new(2, 0), Point::new(4, 0)).unwrap();
        let w2 = c.add_wire(Point::new(4, 0), Point::new(6, 0)).unwrap();
        let lone = c.add_wire(Point::new(0, 10), Point::new(4, 10)).unwrap();
        let mut p = Project::new(c);
        let mut net = p.net_wires(w1);
        net.sort();
        assert_eq!(net, [w1, w2]);
        assert_eq!(p.net_wires(lone), [lone]);
        // Smazání celé sítě jde vrátit.
        // [en] Deleting the whole net can be undone.
        p.delete_wires(&net);
        assert_eq!(p.circuit.wires.len(), 1);
        p.undo();
        assert_eq!(p.circuit.wires.len(), 3);
        // Přidání na bod a změna druhu.
        // [en] Adding at a point and changing the kind.
        let id = p.add_component_at(
            ComponentKind::Clock { period_ms: 1000 },
            Point::new(20, 20),
            Rotation::R90,
        );
        assert_eq!(p.selection.single_comp(), Some(id));
        assert_eq!(p.circuit.component(id).unwrap().rotation, Rotation::R90);
        p.set_kind(id, ComponentKind::Clock { period_ms: 200 });
        assert_eq!(
            p.circuit.component(id).unwrap().kind,
            ComponentKind::Clock { period_ms: 200 }
        );
        p.undo();
        assert_eq!(
            p.circuit.component(id).unwrap().kind,
            ComponentKind::Clock { period_ms: 1000 }
        );
    }

    #[test]
    fn copy_and_paste() {
        let mut p = small();
        p.select_all();
        let text = p.copy_selection().unwrap();
        // Vložení do téhož projektu o 2 body vedle; popisek A je obsazený.
        // [en] Pasting into the same project 2 points aside; label A is taken.
        assert!(p.paste_text(&text, None));
        assert_eq!(p.circuit.components.len(), 4);
        assert_eq!(p.circuit.wires.len(), 2);
        assert_eq!(p.selection.comps.len(), 2);
        let sw = p.selection.comp_vec()[0];
        assert_eq!(p.circuit.component(sw).unwrap().pos, Point::new(2, 2));
        // Zpět vložení vrátí.
        // [en] Undo reverts the paste.
        p.undo();
        assert_eq!(p.circuit.components.len(), 2);
        // Vložení do jiného projektu na bod (20, 20): popisky zůstanou.
        // [en] Pasting into another project at point (20, 20): labels stay.
        let mut q = Project::new(Circuit::default());
        assert!(q.paste_text(&text, Some(Point::new(20, 20))));
        let labels: Vec<&str> = q
            .circuit
            .components
            .iter()
            .map(|c| c.label.as_str())
            .collect();
        assert_eq!(labels, ["A", "Y1"]);
        assert_eq!(q.circuit.components[0].pos, Point::new(17, 20));
        // Text, který není obvod, se ignoruje.
        // [en] Text that is not a circuit is ignored.
        assert!(!q.paste_text("ahoj", None));
        // Vyjmout = kopírovat + smazat.
        // [en] Cut = copy + delete.
        q.select_all();
        assert!(q.cut_selection().is_some());
        assert!(q.circuit.components.is_empty());
    }

    #[test]
    fn paste_brings_subcircuits() {
        // Projekt s podobvodem a jeho instancí v hlavním obvodu.
        // [en] A project with a subcircuit and its instance in the main circuit.
        let mut p = small();
        let id = p.new_subcircuit("Blok");
        p.circuit
            .add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        p.circuit
            .add_component(ComponentKind::Led, Point::new(4, 0));
        p.after_structure_change();
        p.leave_subcircuit();
        let kind = p.subcircuit(id).unwrap().kind();
        p.add_component_at(kind, Point::new(20, 0), Rotation::R0);
        let text = p.copy_selection().unwrap();
        // Do jiného projektu přijde i definice podobvodu.
        // [en] The subcircuit definition comes to another project too.
        let mut q = small();
        assert!(q.paste_text(&text, Some(Point::new(0, 10))));
        assert_eq!(q.subcircuits.len(), 1);
        assert_eq!(q.subcircuits[0].name, "Blok 1");
        // Do téhož projektu se definice použije znovu (žádná kopie).
        // [en] Into the same project the definition is reused (no copy).
        assert!(p.paste_text(&text, None));
        assert_eq!(p.subcircuits.len(), 1);
        // Uvnitř podobvodu nejde vložit jeho vlastní instanci.
        // [en] Inside a subcircuit its own instance cannot be pasted.
        p.enter_subcircuit(id);
        let before = p.circuit.components.len();
        p.paste_text(&text, None);
        assert_eq!(p.circuit.components.len(), before);
    }

    #[test]
    fn clock_frequency_change_keeps_phase() {
        let (mut p, _) = clocked_t();
        let clk = p.circuit.components[0].id;
        p.start_simulation();
        // 1 Hz, čas 0,75 s: hodiny jsou v 1 a uběhla polovina půlperiody.
        // [en] 1 Hz, time 0.75 s: the clock is 1 and half of the half period has passed.
        p.update(0.75);
        assert_eq!(p.sim.output(clk), Some(LogicValue::One));
        // Změna na 2 Hz: úroveň zůstane 1 a příští hrana přijde za polovinu nové
        // půlperiody (0,125 s), ne skokem.
        // [en] Change to 2 Hz: the level stays 1 and the next edge comes in half of the
        // new half period (0.125 s), not with a jump.
        p.set_clock_period(clk, 500, true);
        assert_eq!(p.sim.output(clk), Some(LogicValue::One));
        assert!((p.time_to_next_clock_edge().unwrap() - 0.125).abs() < 1e-9);
        p.update(0.1);
        assert_eq!(p.sim.output(clk), Some(LogicValue::One));
        p.update(0.05);
        assert_eq!(p.sim.output(clk), Some(LogicValue::Zero));
        // Rozsah 0,1–100 Hz.
        // [en] Range 0.1–100 Hz.
        assert_eq!(period_for_hz(1000.0), MIN_CLOCK_PERIOD_MS);
        assert_eq!(period_for_hz(0.01), MAX_CLOCK_PERIOD_MS);
        assert_eq!(period_for_hz(4.0), 250);
    }

    #[test]
    fn speed_and_edge_steps() {
        let (mut p, ff) = clocked_t();
        // Rychlost ×2: za 0,5 s skutečného času uběhne 1 s simulace.
        // [en] Speed ×2: 0.5 s of real time is 1 s of simulation.
        p.start_simulation();
        p.speed = 2.0;
        p.update(0.5);
        assert!((p.time - 1.0).abs() < 1e-9);
        p.reset_simulation();
        p.speed = 1.0;
        p.stop_simulation();
        // Krok na hranu: hodiny 1 Hz mají hrany v 0,5; 1,0; 1,5 s.
        // [en] Edge step: a 1 Hz clock has edges at 0.5, 1.0, 1.5 s.
        p.step_to_edge(false);
        assert!(p.is_simulating() && !p.running);
        assert!((p.time - 0.5).abs() < 1e-6);
        assert_eq!(p.sim.flip_flop_state(ff), Some(LogicValue::One));
        p.step_to_edge(false);
        assert!((p.time - 1.0).abs() < 1e-6);
        // Krok o periodu (na náběžnou hranu): z 1,0 s na 1,5 s – T se překlopí.
        // [en] Period step (to the rising edge): from 1.0 s to 1.5 s – T toggles.
        p.step_to_edge(true);
        assert!((p.time - 1.5).abs() < 1e-6);
        assert_eq!(p.sim.flip_flop_state(ff), Some(LogicValue::Zero));
        p.step_to_edge(true);
        assert!((p.time - 2.5).abs() < 1e-6);
        assert_eq!(p.sim.flip_flop_state(ff), Some(LogicValue::One));
        // Bez hodin krok na hranu nic neudělá.
        // [en] Without clocks the edge step does nothing.
        let mut q = small();
        q.step_to_edge(false);
        assert!(!q.is_simulating());
    }

    #[test]
    fn natural_order() {
        let mut v = vec!["Y10", "Y2", "B", "A"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(v, ["A", "B", "Y2", "Y10"]);
    }
}
