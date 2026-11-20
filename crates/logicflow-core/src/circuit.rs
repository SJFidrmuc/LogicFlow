//! Model obvodu: součástky na mřížce a vodiče mezi body mřížky.
//!
//! Souřadnice jsou celočíselné body mřížky. Vodič je vodorovná nebo svislá
//! úsečka. Vodiče se propojují ve svých koncových bodech; konec vodiče, který
//! leží uprostřed jiného vodiče, se s ním také propojí (odbočka „T“).
//! Vodiče, které se jen kříží, propojené nejsou.

use crate::{BlockKind, GateKind, LogicValue};
use serde::{Deserialize, Serialize};

/// Bod mřížky.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// Otočení součástky po 90° ve směru hodinových ručiček.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    pub const fn next(self) -> Self {
        match self {
            Self::R0 => Self::R90,
            Self::R90 => Self::R180,
            Self::R180 => Self::R270,
            Self::R270 => Self::R0,
        }
    }

    /// Otočí bod `(x, y)` kolem počátku (osa y míří dolů, jako na obrazovce).
    pub const fn apply(self, x: i32, y: i32) -> (i32, i32) {
        match self {
            Self::R0 => (x, y),
            Self::R90 => (-y, x),
            Self::R180 => (-x, -y),
            Self::R270 => (y, -x),
        }
    }

    /// Stejné otočení pro desetinné souřadnice (kreslení).
    pub fn apply_f(self, x: f32, y: f32) -> (f32, f32) {
        match self {
            Self::R0 => (x, y),
            Self::R90 => (-y, x),
            Self::R180 => (-x, -y),
            Self::R270 => (y, -x),
        }
    }

    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::R90 | Self::R270)
    }
}

/// Druh součástky.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComponentKind {
    /// Hradlo s daným počtem vstupů (u NOT a BUF vždy 1).
    Gate { gate: GateKind, inputs: u8 },
    /// Přepínač – drží hodnotu 0 nebo 1.
    Switch { on: bool },
    /// Tlačítko – 1 jen po dobu stisku.
    Button,
    /// Konstanta 0 nebo 1.
    Constant { value: bool },
    /// Svítivá dioda.
    Led,
    /// Klopný obvod s výstupy Q a Q̄. `initial` je stav Q po spuštění simulace.
    FlipFlop {
        ff: FlipFlopKind,
        /// Spouštění sestupnou hranou (jinak náběžnou). Jen u obvodů řízených hranou.
        #[serde(default)]
        falling: bool,
        #[serde(default)]
        initial: bool,
    },
    /// Generátor hodinového signálu s periodou v milisekundách.
    Clock { period_ms: u32 },
    /// Funkční blok (multiplexor, dekodér, čítač, …).
    Block { block: BlockKind },
    /// Sedmisegmentový zobrazovač se vstupy a–g a desetinnou tečkou.
    SevenSegment,
    /// Hexadecimální zobrazovač se 4bitovým vstupem.
    HexDisplay,
}

/// Druh klopného obvodu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlipFlopKind {
    /// Asynchronní RS (vstupy S, R).
    Rs,
    /// D řízený úrovní (vstupy D, C) – „průhledný“, dokud je C = 1.
    DLatch,
    /// D řízený hranou (vstupy D, C).
    D,
    /// JK řízený hranou (vstupy J, C, K).
    Jk,
    /// T řízený hranou (vstupy T, C).
    T,
}

impl FlipFlopKind {
    pub const ALL: [Self; 5] = [Self::Rs, Self::DLatch, Self::D, Self::Jk, Self::T];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Rs => "RS",
            Self::DLatch => "D (úroveň)",
            Self::D => "D",
            Self::Jk => "JK",
            Self::T => "T",
        }
    }

    /// Názvy vstupů shora dolů.
    pub const fn input_names(self) -> &'static [&'static str] {
        match self {
            Self::Rs => &["S", "R"],
            Self::DLatch | Self::D => &["D", "C"],
            Self::Jk => &["J", "C", "K"],
            Self::T => &["T", "C"],
        }
    }

    /// Index hodinového (řídicího) vstupu.
    pub const fn clock_input(self) -> Option<usize> {
        match self {
            Self::Rs => None,
            Self::DLatch | Self::D | Self::T => Some(1),
            Self::Jk => Some(1),
        }
    }

    /// Řízený hranou hodinového signálu?
    pub const fn is_edge_triggered(self) -> bool {
        matches!(self, Self::D | Self::Jk | Self::T)
    }
}

impl ComponentKind {
    pub const MIN_INPUTS: u8 = 2;
    pub const MAX_INPUTS: u8 = 8;

    pub fn gate(gate: GateKind) -> Self {
        let inputs = if gate.is_unary() { 1 } else { 2 };
        Self::Gate { gate, inputs }
    }

    pub fn flip_flop(ff: FlipFlopKind) -> Self {
        Self::FlipFlop {
            ff,
            falling: false,
            initial: false,
        }
    }

    pub const DEFAULT_CLOCK: Self = Self::Clock { period_ms: 1000 };

    pub fn input_count(&self) -> usize {
        self.pins_local().0.len()
    }

    pub fn output_count(&self) -> usize {
        self.pins_local().1.len()
    }

    pub fn has_output(&self) -> bool {
        self.output_count() > 0
    }

    /// Obsahuje součástka paměť (stav)?
    pub fn is_sequential(&self) -> bool {
        match self {
            Self::FlipFlop { .. } => true,
            Self::Block { block } => block.is_sequential(),
            _ => false,
        }
    }

    /// Je to výstup obvodu (zobrazovací prvek)?
    pub fn is_display(&self) -> bool {
        matches!(self, Self::Led | Self::SevenSegment | Self::HexDisplay)
    }

    /// Je to vstup obvodu (zdroj signálu ovládaný uživatelem)?
    pub fn is_input(&self) -> bool {
        matches!(self, Self::Switch { .. } | Self::Button)
    }

    /// Předpona pro automatické pojmenování.
    pub fn label_prefix(&self) -> &'static str {
        match self {
            Self::Gate { .. } => "G",
            Self::Switch { .. } | Self::Button => "",
            Self::Constant { .. } => "K",
            Self::Led | Self::SevenSegment | Self::HexDisplay => "Y",
            Self::Block { .. } => "IO",
            Self::FlipFlop { .. } => "KO",
            Self::Clock { .. } => "CLK",
        }
    }

    /// Ohraničení těla v místních souřadnicích (bez vývodů), pro otočení R0.
    pub fn body_local(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::Gate { inputs, gate } => {
                if gate.is_unary() {
                    (-2.0, -1.5, 2.0, 1.5)
                } else {
                    let h = (*inputs as f32) + 0.5;
                    (-2.0, -h, 2.0, h)
                }
            }
            Self::Switch { .. } | Self::Button => (-1.2, -0.8, 1.2, 0.8),
            Self::Constant { .. } => (-0.8, -0.8, 0.8, 0.8),
            Self::Led => (-0.9, -0.9, 0.9, 0.9),
            Self::FlipFlop {
                ff: FlipFlopKind::Jk,
                ..
            } => (-2.0, -3.5, 2.0, 3.5),
            Self::FlipFlop { .. } => (-2.0, -2.5, 2.0, 2.5),
            Self::Clock { .. } => (-1.4, -1.0, 1.4, 1.0),
            Self::Block { block } => block.body_local(),
            Self::SevenSegment => (-3.0, -8.0, 3.0, 8.0),
            Self::HexDisplay => (-3.0, -4.0, 3.0, 4.0),
        }
    }

    /// Vývody v místních souřadnicích: (vstupy, výstupy).
    pub fn pins_local(&self) -> LocalPins {
        match self {
            Self::Gate { inputs, .. } => {
                let n = *inputs as i32;
                let ins = (0..n).map(|i| (-3, -(n - 1) + 2 * i)).collect();
                (ins, vec![(3, 0)])
            }
            Self::Switch { .. } | Self::Button | Self::Constant { .. } => (vec![], vec![(2, 0)]),
            Self::Led => (vec![(-2, 0)], vec![]),
            Self::FlipFlop {
                ff: FlipFlopKind::Jk,
                ..
            } => (vec![(-3, -2), (-3, 0), (-3, 2)], vec![(3, -2), (3, 2)]),
            Self::FlipFlop { .. } => (vec![(-3, -1), (-3, 1)], vec![(3, -1), (3, 1)]),
            Self::Clock { .. } => (vec![], vec![(3, 0)]),
            Self::Block { block } => block.pins_local(),
            Self::SevenSegment => ((0..8).map(|i| (-4, -7 + 2 * i)).collect(), vec![]),
            Self::HexDisplay => ((0..4).map(|i| (-4, -3 + 2 * i)).collect(), vec![]),
        }
    }
}

/// Místní souřadnice vývodů: (vstupy, výstupy).
pub type LocalPins = (Vec<(i32, i32)>, Vec<(i32, i32)>);

/// Identifikátor součástky (stabilní v rámci souboru).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CompId(pub u32);

/// Identifikátor vodiče.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WireId(pub u32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub id: CompId,
    pub kind: ComponentKind,
    pub pos: Point,
    #[serde(default)]
    pub rotation: Rotation,
    #[serde(default)]
    pub label: String,
    /// Stav tlačítka (neukládá se).
    #[serde(skip)]
    pub pressed: bool,
}

impl Component {
    fn to_world(&self, (x, y): (i32, i32)) -> Point {
        let (rx, ry) = self.rotation.apply(x, y);
        self.pos.offset(rx, ry)
    }

    pub fn input_pins(&self) -> Vec<Point> {
        self.kind
            .pins_local()
            .0
            .into_iter()
            .map(|p| self.to_world(p))
            .collect()
    }

    pub fn output_pins(&self) -> Vec<Point> {
        self.kind
            .pins_local()
            .1
            .into_iter()
            .map(|p| self.to_world(p))
            .collect()
    }

    /// První (hlavní) výstup.
    pub fn output_pin(&self) -> Option<Point> {
        self.output_pins().first().copied()
    }

    /// Všechny vývody (vstupy, pak výstupy).
    pub fn pins(&self) -> Vec<Point> {
        let mut v = self.input_pins();
        v.extend(self.output_pins());
        v
    }

    /// Ohraničení těla ve světových souřadnicích (min_x, min_y, max_x, max_y).
    pub fn body(&self) -> (f32, f32, f32, f32) {
        let (x0, y0, x1, y1) = self.kind.body_local();
        let (ax, ay) = self.rotation.apply_f(x0, y0);
        let (bx, by) = self.rotation.apply_f(x1, y1);
        let (px, py) = (self.pos.x as f32, self.pos.y as f32);
        (
            px + ax.min(bx),
            py + ay.min(by),
            px + ax.max(bx),
            py + ay.max(by),
        )
    }

    /// Hodnota, kterou součástka sama vytváří (zdroje signálu).
    pub fn source_value(&self) -> Option<LogicValue> {
        match self.kind {
            ComponentKind::Switch { on } => Some(on.into()),
            ComponentKind::Button => Some(self.pressed.into()),
            ComponentKind::Constant { value } => Some(value.into()),
            // Hodiny řídí aplikace podle času; na začátku dávají 0.
            ComponentKind::Clock { .. } => Some(LogicValue::Zero),
            _ => None,
        }
    }
}

/// Vodič – vodorovná nebo svislá úsečka.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wire {
    pub id: WireId,
    pub a: Point,
    pub b: Point,
}

impl Wire {
    /// Leží bod na vodiči (včetně konců)?
    pub fn contains(&self, p: Point) -> bool {
        let (a, b) = (self.a, self.b);
        if a.x == b.x {
            p.x == a.x && p.y >= a.y.min(b.y) && p.y <= a.y.max(b.y)
        } else if a.y == b.y {
            p.y == a.y && p.x >= a.x.min(b.x) && p.x <= a.x.max(b.x)
        } else {
            p == a || p == b
        }
    }

    pub fn is_endpoint(&self, p: Point) -> bool {
        p == self.a || p == self.b
    }

    pub fn len(&self) -> i32 {
        (self.a.x - self.b.x).abs() + (self.a.y - self.b.y).abs()
    }

    pub fn is_empty(&self) -> bool {
        self.a == self.b
    }
}

/// Celý obvod (jeden list schématu).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    pub components: Vec<Component>,
    pub wires: Vec<Wire>,
    #[serde(default)]
    next_id: u32,
}

impl Circuit {
    fn alloc_id(&mut self) -> u32 {
        // Po načtení souboru se pokračuje za nejvyšším existujícím id.
        let max = self
            .components
            .iter()
            .map(|c| c.id.0)
            .chain(self.wires.iter().map(|w| w.id.0))
            .max()
            .map_or(0, |m| m + 1);
        self.next_id = self.next_id.max(max);
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Přidá součástku a automaticky ji pojmenuje (A, B, … / Y1, Y2, …).
    pub fn add_component(&mut self, kind: ComponentKind, pos: Point) -> CompId {
        let id = CompId(self.alloc_id());
        let label = self.auto_label(&kind);
        self.components.push(Component {
            id,
            kind,
            pos,
            rotation: Rotation::R0,
            label,
            pressed: false,
        });
        id
    }

    fn auto_label(&self, kind: &ComponentKind) -> String {
        let taken = |l: &str| self.components.iter().any(|c| c.label == l);
        if kind.is_input() {
            for c in 'A'..='Z' {
                let l = c.to_string();
                if !taken(&l) {
                    return l;
                }
            }
        }
        if matches!(kind, ComponentKind::Gate { .. }) {
            return String::new();
        }
        if matches!(kind, ComponentKind::Clock { .. }) && !taken("CLK") {
            return "CLK".to_owned();
        }
        let prefix = if kind.is_input() {
            "X"
        } else {
            kind.label_prefix()
        };
        (1..)
            .map(|n| format!("{prefix}{n}"))
            .find(|l| !taken(l))
            .unwrap_or_default()
    }

    /// Přidá vodič; nulovou nebo šikmou úsečku odmítne.
    pub fn add_wire(&mut self, a: Point, b: Point) -> Option<WireId> {
        if a == b || (a.x != b.x && a.y != b.y) {
            return None;
        }
        // Stejný vodič už existuje.
        if self
            .wires
            .iter()
            .any(|w| (w.a == a && w.b == b) || (w.a == b && w.b == a))
        {
            return None;
        }
        let id = WireId(self.alloc_id());
        self.wires.push(Wire { id, a, b });
        Some(id)
    }

    pub fn component(&self, id: CompId) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }

    pub fn component_mut(&mut self, id: CompId) -> Option<&mut Component> {
        self.components.iter_mut().find(|c| c.id == id)
    }

    pub fn wire(&self, id: WireId) -> Option<&Wire> {
        self.wires.iter().find(|w| w.id == id)
    }

    pub fn remove_component(&mut self, id: CompId) {
        self.components.retain(|c| c.id != id);
    }

    pub fn remove_wire(&mut self, id: WireId) {
        self.wires.retain(|w| w.id != id);
    }

    /// Vrátí kopii obvodu, ve které jsou vybrané součástky a vodiče posunuté o (dx, dy).
    ///
    /// Nevybrané vodiče připojené k vývodu posunuté součástky se přepojí:
    /// konec u vývodu se posune a z úsečky vznikne „L“ (vodorovná + svislá část),
    /// takže spojení zůstane zachované.
    pub fn moved(&self, comps: &[CompId], wires: &[WireId], dx: i32, dy: i32) -> Circuit {
        let mut c = self.clone();
        if dx == 0 && dy == 0 {
            return c;
        }
        let moving_pins: Vec<Point> = self
            .components
            .iter()
            .filter(|k| comps.contains(&k.id))
            .flat_map(|k| k.pins())
            .collect();
        for k in c.components.iter_mut().filter(|k| comps.contains(&k.id)) {
            k.pos = k.pos.offset(dx, dy);
        }
        let ends_move = |w: &Wire| {
            let sel = wires.contains(&w.id);
            (
                sel || moving_pins.contains(&w.a),
                sel || moving_pins.contains(&w.b),
            )
        };
        let mut extra: Vec<(Point, Point)> = Vec::new();
        for (i, w) in self.wires.iter().enumerate() {
            match ends_move(w) {
                (true, true) => {
                    c.wires[i].a = w.a.offset(dx, dy);
                    c.wires[i].b = w.b.offset(dx, dy);
                }
                (false, false) => {}
                (a_side, _) => {
                    let (moving, fixed) = if a_side { (w.a, w.b) } else { (w.b, w.a) };
                    let horizontal = moving.y == fixed.y;
                    let new = moving.offset(dx, dy);
                    // Bod u pevného konce: zachová směr části u vývodu.
                    let corner = if horizontal {
                        Point::new(fixed.x, new.y)
                    } else {
                        Point::new(new.x, fixed.y)
                    };
                    let (na, nb) = if a_side { (new, corner) } else { (corner, new) };
                    c.wires[i].a = na;
                    c.wires[i].b = nb;
                    match self.sole_partner(i, fixed, &ends_move) {
                        // Pevný konec je roh lomeného vodiče: roh se posune s ním.
                        Some(j) => {
                            let pw = &mut c.wires[j];
                            if pw.a == fixed {
                                pw.a = corner;
                            } else {
                                pw.b = corner;
                            }
                        }
                        // Jinak vznikne nový roh a spojka k původnímu bodu.
                        None => extra.push((corner, fixed)),
                    }
                }
            }
        }
        c.wires.retain(|w| !w.is_empty());
        for (a, b) in extra {
            c.add_wire(a, b);
        }
        c
    }

    /// Je bod `p` rohem, kde se vodič `i` stýká s jediným dalším kolmým vodičem
    /// (a s ničím jiným)? Pak vrátí index toho vodiče.
    fn sole_partner(
        &self,
        i: usize,
        p: Point,
        ends_move: &impl Fn(&Wire) -> (bool, bool),
    ) -> Option<usize> {
        if self.components.iter().any(|k| k.pins().contains(&p)) {
            return None;
        }
        let w = &self.wires[i];
        let mut touching = self
            .wires
            .iter()
            .enumerate()
            .filter(|(k, other)| *k != i && other.contains(p));
        let (j, other) = touching.next()?;
        if touching.next().is_some() || !other.is_endpoint(p) || ends_move(other) != (false, false)
        {
            return None;
        }
        let w_horizontal = w.a.y == w.b.y;
        let o_horizontal = other.a.y == other.b.y;
        (w_horizontal != o_horizontal).then_some(j)
    }

    /// Zkopíruje vybrané součástky a vodiče (posunuté o 2 body) a vrátí id kopií.
    pub fn duplicate(&mut self, comps: &[CompId], wires: &[WireId]) -> (Vec<CompId>, Vec<WireId>) {
        let originals: Vec<Component> = self
            .components
            .iter()
            .filter(|k| comps.contains(&k.id))
            .cloned()
            .collect();
        let new_comps = originals
            .into_iter()
            .map(|k| {
                let id = self.add_component(k.kind, k.pos.offset(2, 2));
                let copy = self.component_mut(id).expect("právě přidáno");
                copy.rotation = k.rotation;
                if matches!(k.kind, ComponentKind::Gate { .. }) {
                    copy.label = k.label;
                }
                id
            })
            .collect();
        let segs: Vec<(Point, Point)> = self
            .wires
            .iter()
            .filter(|w| wires.contains(&w.id))
            .map(|w| (w.a.offset(2, 2), w.b.offset(2, 2)))
            .collect();
        let new_wires = segs
            .into_iter()
            .filter_map(|(a, b)| self.add_wire(a, b))
            .collect();
        (new_comps, new_wires)
    }

    /// Body, ve kterých se kreslí spojovací tečka: setkávají se zde aspoň tři větve.
    pub fn junctions(&self) -> Vec<Point> {
        let mut points: Vec<Point> = self.wires.iter().flat_map(|w| [w.a, w.b]).collect();
        points.sort();
        points.dedup();
        points
            .into_iter()
            .filter(|&p| {
                let mut branches = 0;
                for w in &self.wires {
                    if w.is_endpoint(p) {
                        branches += 1;
                    } else if w.contains(p) {
                        branches += 2;
                    }
                }
                branches += self
                    .components
                    .iter()
                    .flat_map(|c| c.pins())
                    .filter(|&q| q == p)
                    .count();
                branches >= 3
            })
            .collect()
    }

    /// Ohraničení celého obvodu (min, max) v bodech mřížky.
    pub fn bounds(&self) -> Option<(f32, f32, f32, f32)> {
        let mut it = self
            .components
            .iter()
            .map(|c| {
                let (x0, y0, x1, y1) = c.body();
                // Rezerva na vývody a popisky.
                (x0 - 4.0, y0 - 1.5, x1 + 7.0, y1 + 1.5)
            })
            .chain(self.wires.iter().map(|w| {
                (
                    w.a.x.min(w.b.x) as f32,
                    w.a.y.min(w.b.y) as f32,
                    w.a.x.max(w.b.x) as f32,
                    w.a.y.max(w.b.y) as f32,
                )
            }));
        let first = it.next()?;
        Some(it.fold(first, |a, b| {
            (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
        }))
    }
}

/// Formát souboru projektu `.logicflow`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Verze formátu – zvýší se při nekompatibilní změně.
    pub format: u32,
    pub app: String,
    pub circuit: Circuit,
}

impl ProjectFile {
    pub const FORMAT: u32 = 1;
    pub const EXTENSION: &'static str = "logicflow";

    pub fn to_json(circuit: &Circuit) -> String {
        let file = ProjectFile {
            format: Self::FORMAT,
            app: format!("LogicFlow {}", crate::VERSION),
            circuit: circuit.clone(),
        };
        serde_json::to_string_pretty(&file).expect("serializace obvodu nemůže selhat")
    }

    pub fn from_json(text: &str) -> Result<Circuit, String> {
        let file: ProjectFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if file.format > Self::FORMAT {
            return Err(format!(
                "Soubor je z novější verze aplikace (formát {}), tato verze umí formát {}.",
                file.format,
                Self::FORMAT
            ));
        }
        Ok(file.circuit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_moves_pins() {
        let mut c = Circuit::default();
        let id = c.add_component(ComponentKind::gate(GateKind::And), Point::new(10, 10));
        let g = c.component(id).unwrap();
        assert_eq!(g.input_pins(), vec![Point::new(7, 9), Point::new(7, 11)]);
        assert_eq!(g.output_pin(), Some(Point::new(13, 10)));
        c.component_mut(id).unwrap().rotation = Rotation::R90;
        let g = c.component(id).unwrap();
        // Po otočení o 90° míří výstup dolů.
        assert_eq!(g.output_pin(), Some(Point::new(10, 13)));
    }

    #[test]
    fn auto_labels() {
        let mut c = Circuit::default();
        let a = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let b = c.add_component(ComponentKind::Button, Point::new(0, 4));
        let y = c.add_component(ComponentKind::Led, Point::new(10, 0));
        let g = c.add_component(ComponentKind::gate(GateKind::Or), Point::new(5, 0));
        assert_eq!(c.component(a).unwrap().label, "A");
        assert_eq!(c.component(b).unwrap().label, "B");
        assert_eq!(c.component(y).unwrap().label, "Y1");
        assert_eq!(c.component(g).unwrap().label, "");
    }

    #[test]
    fn wire_rules() {
        let mut c = Circuit::default();
        assert!(
            c.add_wire(Point::new(0, 0), Point::new(3, 3)).is_none(),
            "šikmý vodič"
        );
        assert!(
            c.add_wire(Point::new(0, 0), Point::new(0, 0)).is_none(),
            "nulová délka"
        );
        let w = c.add_wire(Point::new(0, 0), Point::new(5, 0)).unwrap();
        assert!(
            c.add_wire(Point::new(5, 0), Point::new(0, 0)).is_none(),
            "duplicita"
        );
        assert!(c.wire(w).unwrap().contains(Point::new(3, 0)));
        assert!(!c.wire(w).unwrap().contains(Point::new(3, 1)));
    }

    #[test]
    fn junction_at_t_branch() {
        let mut c = Circuit::default();
        c.add_wire(Point::new(0, 0), Point::new(10, 0));
        c.add_wire(Point::new(5, 0), Point::new(5, 5));
        // Křížení bez spoje tečku nemá.
        c.add_wire(Point::new(8, -3), Point::new(8, 3));
        assert_eq!(c.junctions(), vec![Point::new(5, 0)]);
    }

    #[test]
    fn json_roundtrip_and_ids_continue() {
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        c.add_wire(Point::new(2, 0), Point::new(6, 0));
        let text = ProjectFile::to_json(&c);
        let mut loaded = ProjectFile::from_json(&text).unwrap();
        assert_eq!(loaded, c);
        let id = loaded.add_component(ComponentKind::Led, Point::new(8, 0));
        assert!(id.0 >= 2, "nové id nesmí kolidovat s načtenými");
    }

    #[test]
    fn moving_component_reroutes_wire() {
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        let led = c.add_component(ComponentKind::Led, Point::new(12, 0));
        c.add_wire(Point::new(2, 0), Point::new(10, 0));
        let m = c.moved(&[sw], &[], 0, 3);
        assert_eq!(m.component(sw).unwrap().pos, Point::new(0, 3));
        // Vodič se rozdělí na vodorovnou a svislou část a spojení zůstane.
        assert_eq!(m.wires.len(), 2);
        let mut sim = crate::Simulator::new(&m, None);
        sim.run(100);
        assert_eq!(sim.inputs(led)[0], LogicValue::One);
        // Původní obvod zůstal beze změny.
        assert_eq!(c.wires.len(), 1);
    }

    #[test]
    fn moving_component_slides_corner() {
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        let led = c.add_component(ComponentKind::Led, Point::new(12, 6));
        // lomený vodič: (2,0) → (6,0) → (6,6) → (10,6)
        c.add_wire(Point::new(2, 0), Point::new(6, 0));
        c.add_wire(Point::new(6, 0), Point::new(6, 6));
        c.add_wire(Point::new(6, 6), Point::new(10, 6));
        let m = c.moved(&[sw], &[], 0, 2);
        // Žádný nový úsek, roh se jen posunul.
        assert_eq!(m.wires.len(), 3);
        assert!(
            m.wires
                .iter()
                .any(|w| w.a == Point::new(2, 2) && w.b == Point::new(6, 2))
        );
        assert!(
            m.wires
                .iter()
                .any(|w| w.a == Point::new(6, 2) && w.b == Point::new(6, 6))
        );
        let mut sim = crate::Simulator::new(&m, None);
        sim.run(100);
        assert_eq!(sim.inputs(led)[0], LogicValue::One);
    }

    #[test]
    fn moving_selection_with_wires() {
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        let w = c.add_wire(Point::new(2, 0), Point::new(6, 0)).unwrap();
        let m = c.moved(&[sw], &[w], 5, 5);
        assert_eq!(m.wires.len(), 1);
        assert_eq!(m.wires[0].a, Point::new(7, 5));
        assert_eq!(m.wires[0].b, Point::new(11, 5));
    }

    #[test]
    fn duplicate_gets_new_ids_and_labels() {
        let mut c = Circuit::default();
        let a = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let w = c.add_wire(Point::new(2, 0), Point::new(5, 0)).unwrap();
        let (comps, wires) = c.duplicate(&[a], &[w]);
        assert_eq!(comps.len(), 1);
        assert_eq!(wires.len(), 1);
        assert_ne!(comps[0], a);
        assert_eq!(c.component(comps[0]).unwrap().label, "B");
        assert_eq!(c.components.len(), 2);
    }

    #[test]
    fn newer_format_is_rejected() {
        let text = r#"{"format": 99, "app": "x", "circuit": {"components": [], "wires": []}}"#;
        assert!(ProjectFile::from_json(text).is_err());
    }
}
