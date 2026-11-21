//! Model obvodu: součástky na mřížce a vodiče mezi body mřížky.
//!
//! Souřadnice jsou celočíselné body mřížky. Vodič je vodorovná nebo svislá
//! úsečka. Vodiče se propojují ve svých koncových bodech; konec vodiče, který
//! leží uprostřed jiného vodiče, se s ním také propojí (odbočka „T“).
//! Vodiče, které se jen kříží, propojené nejsou.
//!
//! [en] Circuit model: components on a grid and wires between grid points.
//!
//! Coordinates are integer grid points. A wire is a horizontal or vertical
//! segment. Wires connect at their end points; a wire end lying in the middle
//! of another wire connects to it as well (a “T” junction). Wires that only
//! cross each other are not connected.

use crate::{BlockKind, GateKind, LogicValue};
use serde::{Deserialize, Serialize};

/// Bod mřížky.
///
/// [en] Grid point.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Point {
    /// Vodorovná souřadnice (roste doprava).
    ///
    /// [en] Horizontal coordinate (grows to the right).
    pub x: i32,
    /// Svislá souřadnice (roste dolů).
    ///
    /// [en] Vertical coordinate (grows downwards).
    pub y: i32,
}

impl Point {
    /// Bod se souřadnicemi `x`, `y`.
    ///
    /// [en] Point with coordinates `x`, `y`.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Bod posunutý o `dx`, `dy`.
    ///
    /// [en] Point moved by `dx`, `dy`.
    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// Otočení součástky po 90° ve směru hodinových ručiček.
///
/// [en] Rotation of a component in 90° steps clockwise.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    /// Bez otočení.
    ///
    /// [en] No rotation.
    #[default]
    R0,
    /// Otočení o 90°.
    ///
    /// [en] Rotated by 90°.
    R90,
    /// Otočení o 180°.
    ///
    /// [en] Rotated by 180°.
    R180,
    /// Otočení o 270°.
    ///
    /// [en] Rotated by 270°.
    R270,
}

impl Rotation {
    /// Další otočení o 90° (po R270 následuje R0).
    ///
    /// [en] Next rotation by 90° (R0 follows R270).
    pub const fn next(self) -> Self {
        match self {
            Self::R0 => Self::R90,
            Self::R90 => Self::R180,
            Self::R180 => Self::R270,
            Self::R270 => Self::R0,
        }
    }

    /// Otočí bod `(x, y)` kolem počátku (osa y míří dolů, jako na obrazovce).
    ///
    /// [en] Rotates the point `(x, y)` around the origin (the y axis points down, as on screen).
    pub const fn apply(self, x: i32, y: i32) -> (i32, i32) {
        match self {
            Self::R0 => (x, y),
            Self::R90 => (-y, x),
            Self::R180 => (-x, -y),
            Self::R270 => (y, -x),
        }
    }

    /// Stejné otočení pro desetinné souřadnice (kreslení).
    ///
    /// [en] The same rotation for floating-point coordinates (drawing).
    pub fn apply_f(self, x: f32, y: f32) -> (f32, f32) {
        match self {
            Self::R0 => (x, y),
            Self::R90 => (-y, x),
            Self::R180 => (-x, -y),
            Self::R270 => (y, -x),
        }
    }

    /// Je součástka otočená na výšku (o 90° nebo 270°)?
    ///
    /// [en] Is the component turned upright (by 90° or 270°)?
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::R90 | Self::R270)
    }
}

/// Druh součástky.
///
/// [en] Kind of component.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComponentKind {
    /// Hradlo s daným počtem vstupů (u NOT a BUF vždy 1).
    ///
    /// [en] Gate with the given number of inputs (always 1 for NOT and BUF).
    Gate {
        /// Druh hradla.
        ///
        /// [en] Kind of gate.
        gate: GateKind,
        /// Počet vstupů.
        ///
        /// [en] Number of inputs.
        inputs: u8,
    },
    /// Přepínač – drží hodnotu 0 nebo 1.
    ///
    /// [en] Switch – holds the value 0 or 1.
    Switch {
        /// Zapnuto (výstup 1)?
        ///
        /// [en] Switched on (output 1)?
        on: bool,
    },
    /// Tlačítko – 1 jen po dobu stisku.
    ///
    /// [en] Push button – 1 only while pressed.
    Button,
    /// Konstanta 0 nebo 1.
    ///
    /// [en] Constant 0 or 1.
    Constant {
        /// Hodnota konstanty (true = 1).
        ///
        /// [en] Value of the constant (true = 1).
        value: bool,
    },
    /// Svítivá dioda.
    ///
    /// [en] Light-emitting diode.
    Led,
    /// Klopný obvod s výstupy Q a Q̄. `initial` je stav Q po spuštění simulace.
    ///
    /// [en] Flip-flop with outputs Q and Q̄. `initial` is the state of Q after the simulation starts.
    FlipFlop {
        /// Druh klopného obvodu.
        ///
        /// [en] Kind of flip-flop.
        ff: FlipFlopKind,
        /// Spouštění sestupnou hranou (jinak náběžnou). Jen u obvodů řízených hranou.
        ///
        /// [en] Triggered by the falling edge (otherwise the rising one). Only for edge-triggered flip-flops.
        #[serde(default)]
        falling: bool,
        /// Stav Q po spuštění simulace.
        ///
        /// [en] State of Q after the simulation starts.
        #[serde(default)]
        initial: bool,
    },
    /// Generátor hodinového signálu s periodou v milisekundách.
    ///
    /// [en] Clock generator with the period in milliseconds.
    Clock {
        /// Perioda v milisekundách.
        ///
        /// [en] Period in milliseconds.
        period_ms: u32,
    },
    /// Funkční blok (multiplexor, dekodér, čítač, …).
    ///
    /// [en] Function block (multiplexer, decoder, counter, …).
    Block {
        /// Druh a nastavení bloku.
        ///
        /// [en] Kind and settings of the block.
        block: BlockKind,
    },
    /// Sedmisegmentový zobrazovač se vstupy a–g a desetinnou tečkou.
    ///
    /// [en] Seven-segment display with inputs a–g and a decimal point.
    SevenSegment {
        /// Strana, na které jsou vývody.
        ///
        /// [en] Side on which the pins are.
        #[serde(default)]
        pins: PinSide,
    },
    /// Hexadecimální zobrazovač se 4bitovým vstupem.
    ///
    /// [en] Hexadecimal display with a 4-bit input.
    HexDisplay {
        /// Strana, na které jsou vývody.
        ///
        /// [en] Side on which the pins are.
        #[serde(default)]
        pins: PinSide,
    },
    /// Instance podobvodu (definice `id` v projektu). Počty vývodů jsou kopií
    /// rozhraní definice, aby šlo značku nakreslit a propojit bez ní.
    ///
    /// [en] Instance of a subcircuit (definition `id` in the project). The pin counts are
    /// a copy of the definition's interface so that the symbol can be drawn and
    /// connected without it.
    Subcircuit {
        /// Id definice podobvodu v projektu.
        ///
        /// [en] Id of the subcircuit definition in the project.
        id: u32,
        /// Počet vstupů značky.
        ///
        /// [en] Number of the symbol's inputs.
        inputs: u8,
        /// Počet výstupů značky.
        ///
        /// [en] Number of the symbol's outputs.
        outputs: u8,
    },
}

/// Strana, na které má zobrazovač vývody (číslice zůstává vždy rovně).
///
/// [en] Side on which a display has its pins (the digit always stays upright).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinSide {
    /// Vývody vlevo.
    ///
    /// [en] Pins on the left.
    #[default]
    Left,
    /// Vývody vpravo.
    ///
    /// [en] Pins on the right.
    Right,
    /// Vývody nahoře.
    ///
    /// [en] Pins on the top.
    Top,
    /// Vývody dole.
    ///
    /// [en] Pins on the bottom.
    Bottom,
}

impl PinSide {
    /// Všechny strany v pořadí nabídky.
    ///
    /// [en] All sides in menu order.
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    /// Tělo a vývody zobrazovače s `n` vstupy. `along` je poloviční délka strany
    /// s vývody, `across` poloviční šířka (u vývodů vlevo/vpravo).
    ///
    /// [en] Body and pins of a display with `n` inputs. `along` is half the length of
    /// the side with pins, `across` half the width (for pins on the left/right).
    fn display_layout(self, n: i32, along: f32, across: f32) -> ((f32, f32, f32, f32), LocalPins) {
        let first = -(n - 1);
        let reach = across as i32 + 1;
        match self {
            Self::Left | Self::Right => {
                let x = if self == Self::Left { -reach } else { reach };
                (
                    (-across, -along, across, along),
                    ((0..n).map(|i| (x, first + 2 * i)).collect(), vec![]),
                )
            }
            Self::Top | Self::Bottom => {
                // Vývody vedle sebe: tělo je široké podle počtu vývodů.
                // [en] Pins side by side: the body is as wide as the number of pins requires.
                let half_w = (n as f32).max(across + 1.0);
                let half_h = (along - 2.0).max(4.0);
                let y = half_h as i32 + 1;
                let y = if self == Self::Top { -y } else { y };
                (
                    (-half_w, -half_h, half_w, half_h),
                    ((0..n).map(|i| (first + 2 * i, y)).collect(), vec![]),
                )
            }
        }
    }
}

/// Druh klopného obvodu.
///
/// [en] Kind of flip-flop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlipFlopKind {
    /// Asynchronní RS (vstupy S, R).
    ///
    /// [en] Asynchronous RS (inputs S, R).
    Rs,
    /// D řízený úrovní (vstupy D, C) – „průhledný“, dokud je C = 1.
    ///
    /// [en] Level-triggered D latch (inputs D, C) – “transparent” while C = 1.
    DLatch,
    /// D řízený hranou (vstupy D, C).
    ///
    /// [en] Edge-triggered D (inputs D, C).
    D,
    /// JK řízený hranou (vstupy J, C, K).
    ///
    /// [en] Edge-triggered JK (inputs J, C, K).
    Jk,
    /// T řízený hranou (vstupy T, C).
    ///
    /// [en] Edge-triggered T (inputs T, C).
    T,
}

impl FlipFlopKind {
    /// Všechny klopné obvody v pořadí knihovny.
    ///
    /// [en] All flip-flops in library order.
    pub const ALL: [Self; 5] = [Self::Rs, Self::DLatch, Self::D, Self::Jk, Self::T];

    /// Krátký název (RS, D, JK, …).
    ///
    /// [en] Short name (RS, D, JK, …).
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
    ///
    /// [en] Input names from top to bottom.
    pub const fn input_names(self) -> &'static [&'static str] {
        match self {
            Self::Rs => &["S", "R"],
            Self::DLatch | Self::D => &["D", "C"],
            Self::Jk => &["J", "C", "K"],
            Self::T => &["T", "C"],
        }
    }

    /// Index hodinového (řídicího) vstupu.
    ///
    /// [en] Index of the clock (control) input.
    pub const fn clock_input(self) -> Option<usize> {
        match self {
            Self::Rs => None,
            Self::DLatch | Self::D | Self::T => Some(1),
            Self::Jk => Some(1),
        }
    }

    /// Řízený hranou hodinového signálu?
    ///
    /// [en] Triggered by a clock edge?
    pub const fn is_edge_triggered(self) -> bool {
        matches!(self, Self::D | Self::Jk | Self::T)
    }
}

impl ComponentKind {
    /// Nejmenší počet vstupů vícevstupového hradla.
    ///
    /// [en] Smallest number of inputs of a multi-input gate.
    pub const MIN_INPUTS: u8 = 2;
    /// Největší počet vstupů vícevstupového hradla.
    ///
    /// [en] Largest number of inputs of a multi-input gate.
    pub const MAX_INPUTS: u8 = 8;

    /// Hradlo s výchozím počtem vstupů (1 u NOT a BUF, jinak 2).
    ///
    /// [en] Gate with the default number of inputs (1 for NOT and BUF, otherwise 2).
    pub fn gate(gate: GateKind) -> Self {
        let inputs = if gate.is_unary() { 1 } else { 2 };
        Self::Gate { gate, inputs }
    }

    /// Klopný obvod řízený náběžnou hranou s počátečním stavem 0.
    ///
    /// [en] Flip-flop triggered by the rising edge with initial state 0.
    pub fn flip_flop(ff: FlipFlopKind) -> Self {
        Self::FlipFlop {
            ff,
            falling: false,
            initial: false,
        }
    }

    /// Výchozí generátor hodin (1 Hz).
    ///
    /// [en] Default clock generator (1 Hz).
    pub const DEFAULT_CLOCK: Self = Self::Clock { period_ms: 1000 };

    /// Počet vstupů.
    ///
    /// [en] Number of inputs.
    pub fn input_count(&self) -> usize {
        self.pins_local().0.len()
    }

    /// Počet výstupů.
    ///
    /// [en] Number of outputs.
    pub fn output_count(&self) -> usize {
        self.pins_local().1.len()
    }

    /// Má součástka aspoň jeden výstup?
    ///
    /// [en] Does the component have at least one output?
    pub fn has_output(&self) -> bool {
        self.output_count() > 0
    }

    /// Obsahuje součástka paměť (stav)?
    ///
    /// [en] Does the component contain memory (state)?
    pub fn is_sequential(&self) -> bool {
        match self {
            Self::FlipFlop { .. } => true,
            Self::Block { block } => block.is_sequential(),
            _ => false,
        }
    }

    /// Je to výstup obvodu (zobrazovací prvek)?
    ///
    /// [en] Is it a circuit output (display element)?
    pub fn is_display(&self) -> bool {
        matches!(
            self,
            Self::Led | Self::SevenSegment { .. } | Self::HexDisplay { .. }
        )
    }

    /// Je to vstup obvodu (zdroj signálu ovládaný uživatelem)?
    ///
    /// [en] Is it a circuit input (signal source controlled by the user)?
    pub fn is_input(&self) -> bool {
        matches!(self, Self::Switch { .. } | Self::Button)
    }

    /// Předpona pro automatické pojmenování.
    ///
    /// [en] Prefix for automatic naming.
    pub fn label_prefix(&self) -> &'static str {
        match self {
            Self::Gate { .. } => "G",
            Self::Switch { .. } | Self::Button => "",
            Self::Constant { .. } => "K",
            Self::Led | Self::SevenSegment { .. } | Self::HexDisplay { .. } => "Y",
            Self::Block { .. } => "IO",
            Self::Subcircuit { .. } => "P",
            Self::FlipFlop { .. } => "KO",
            Self::Clock { .. } => "CLK",
        }
    }

    /// Ohraničení těla v místních souřadnicích (bez vývodů), pro otočení R0.
    ///
    /// [en] Bounds of the body in local coordinates (without pins), for rotation R0.
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
            Self::SevenSegment { pins } => pins.display_layout(8, 8.0, 3.0).0,
            Self::HexDisplay { pins } => pins.display_layout(4, 4.0, 3.0).0,
            Self::Subcircuit {
                inputs, outputs, ..
            } => crate::block::rect_body(*inputs as usize, *outputs as usize),
        }
    }

    /// Vývody v místních souřadnicích: (vstupy, výstupy).
    ///
    /// [en] Pins in local coordinates: (inputs, outputs).
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
            Self::SevenSegment { pins } => pins.display_layout(8, 8.0, 3.0).1,
            Self::HexDisplay { pins } => pins.display_layout(4, 4.0, 3.0).1,
            Self::Subcircuit {
                inputs, outputs, ..
            } => crate::block::rect_pins(*inputs as usize, *outputs as usize),
        }
    }
}

/// Místní souřadnice vývodů: (vstupy, výstupy).
///
/// [en] Local pin coordinates: (inputs, outputs).
pub type LocalPins = (Vec<(i32, i32)>, Vec<(i32, i32)>);

/// Identifikátor součástky (stabilní v rámci souboru).
///
/// [en] Component identifier (stable within a file).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CompId(pub u32);

/// Identifikátor vodiče.
///
/// [en] Wire identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WireId(pub u32);

/// Součástka umístěná ve schématu.
///
/// [en] Component placed in the schematic.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Component {
    /// Identifikátor.
    ///
    /// [en] Identifier.
    pub id: CompId,
    /// Druh součástky a jeho nastavení.
    ///
    /// [en] Kind of component and its settings.
    pub kind: ComponentKind,
    /// Poloha počátku značky na mřížce.
    ///
    /// [en] Position of the symbol's origin on the grid.
    pub pos: Point,
    /// Otočení.
    ///
    /// [en] Rotation.
    #[serde(default)]
    pub rotation: Rotation,
    /// Popisek (A, B, Y1…); u podobvodu tvoří názvy vývodů.
    ///
    /// [en] Label (A, B, Y1…); inside a subcircuit it forms the pin names.
    #[serde(default)]
    pub label: String,
    /// Stav tlačítka (neukládá se).
    ///
    /// [en] Push button state (not saved).
    #[serde(skip)]
    pub pressed: bool,
}

impl Component {
    /// Převede místní souřadnice vývodu na světové (otočení a posun).
    ///
    /// [en] Converts local pin coordinates to world coordinates (rotation and offset).
    fn to_world(&self, (x, y): (i32, i32)) -> Point {
        let (rx, ry) = self.rotation.apply(x, y);
        self.pos.offset(rx, ry)
    }

    /// Vstupní vývody ve světových souřadnicích.
    ///
    /// [en] Input pins in world coordinates.
    pub fn input_pins(&self) -> Vec<Point> {
        self.kind
            .pins_local()
            .0
            .into_iter()
            .map(|p| self.to_world(p))
            .collect()
    }

    /// Výstupní vývody ve světových souřadnicích.
    ///
    /// [en] Output pins in world coordinates.
    pub fn output_pins(&self) -> Vec<Point> {
        self.kind
            .pins_local()
            .1
            .into_iter()
            .map(|p| self.to_world(p))
            .collect()
    }

    /// První (hlavní) výstup.
    ///
    /// [en] First (main) output.
    pub fn output_pin(&self) -> Option<Point> {
        self.output_pins().first().copied()
    }

    /// Všechny vývody (vstupy, pak výstupy).
    ///
    /// [en] All pins (inputs, then outputs).
    pub fn pins(&self) -> Vec<Point> {
        let mut v = self.input_pins();
        v.extend(self.output_pins());
        v
    }

    /// Ohraničení těla ve světových souřadnicích (min_x, min_y, max_x, max_y).
    ///
    /// [en] Bounds of the body in world coordinates (min_x, min_y, max_x, max_y).
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
    ///
    /// [en] Value the component produces by itself (signal sources).
    pub fn source_value(&self) -> Option<LogicValue> {
        match self.kind {
            ComponentKind::Switch { on } => Some(on.into()),
            ComponentKind::Button => Some(self.pressed.into()),
            ComponentKind::Constant { value } => Some(value.into()),
            // Hodiny řídí aplikace podle času; na začátku dávají 0.
            // [en] The application drives clocks by time; they start at 0.
            ComponentKind::Clock { .. } => Some(LogicValue::Zero),
            _ => None,
        }
    }
}

/// Vodič – vodorovná nebo svislá úsečka.
///
/// [en] Wire – a horizontal or vertical segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wire {
    /// Identifikátor.
    ///
    /// [en] Identifier.
    pub id: WireId,
    /// První konec.
    ///
    /// [en] First end.
    pub a: Point,
    /// Druhý konec.
    ///
    /// [en] Second end.
    pub b: Point,
}

impl Wire {
    /// Leží bod na vodiči (včetně konců)?
    ///
    /// [en] Does the point lie on the wire (ends included)?
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

    /// Je bod jedním z konců vodiče?
    ///
    /// [en] Is the point one of the wire's ends?
    pub fn is_endpoint(&self, p: Point) -> bool {
        p == self.a || p == self.b
    }

    /// Délka vodiče v bodech mřížky.
    ///
    /// [en] Length of the wire in grid points.
    pub fn len(&self) -> i32 {
        (self.a.x - self.b.x).abs() + (self.a.y - self.b.y).abs()
    }

    /// Má vodič nulovou délku?
    ///
    /// [en] Does the wire have zero length?
    pub fn is_empty(&self) -> bool {
        self.a == self.b
    }
}

/// Celý obvod (jeden list schématu).
///
/// [en] The whole circuit (one schematic sheet).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    /// Součástky.
    ///
    /// [en] Components.
    pub components: Vec<Component>,
    /// Vodiče.
    ///
    /// [en] Wires.
    pub wires: Vec<Wire>,
    /// Poslední přidělené id (součástky i vodiče sdílejí jednu řadu).
    ///
    /// [en] Last allocated id (components and wires share one sequence).
    #[serde(default)]
    next_id: u32,
}

impl Circuit {
    /// Přidělí nové id.
    ///
    /// [en] Allocates a new id.
    fn alloc_id(&mut self) -> u32 {
        // Po načtení souboru se pokračuje za nejvyšším existujícím id.
        // [en] After loading a file, numbering continues after the highest existing id.
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
    ///
    /// [en] Adds a component and names it automatically (A, B, … / Y1, Y2, …).
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

    /// První volný popisek pro daný druh součástky.
    ///
    /// [en] First free label for the given kind of component.
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
    ///
    /// [en] Adds a wire; rejects a zero-length or diagonal segment.
    pub fn add_wire(&mut self, a: Point, b: Point) -> Option<WireId> {
        if a == b || (a.x != b.x && a.y != b.y) {
            return None;
        }
        // Stejný vodič už existuje.
        // [en] The same wire already exists.
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

    /// Součástka podle id.
    ///
    /// [en] Component by id.
    pub fn component(&self, id: CompId) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }

    /// Součástka podle id pro úpravu.
    ///
    /// [en] Component by id for editing.
    pub fn component_mut(&mut self, id: CompId) -> Option<&mut Component> {
        self.components.iter_mut().find(|c| c.id == id)
    }

    /// Vodič podle id.
    ///
    /// [en] Wire by id.
    pub fn wire(&self, id: WireId) -> Option<&Wire> {
        self.wires.iter().find(|w| w.id == id)
    }

    /// Odstraní součástku (vodiče zůstanou).
    ///
    /// [en] Removes a component (wires stay).
    pub fn remove_component(&mut self, id: CompId) {
        self.components.retain(|c| c.id != id);
    }

    /// Odstraní vodič.
    ///
    /// [en] Removes a wire.
    pub fn remove_wire(&mut self, id: WireId) {
        self.wires.retain(|w| w.id != id);
    }

    /// Vrátí kopii obvodu, ve které jsou vybrané součástky a vodiče posunuté o (dx, dy).
    ///
    /// Nevybrané vodiče připojené k vývodu posunuté součástky se přepojí: konec
    /// u vývodu se posune a vodič se zalomí. Místo zalomení se volí tak, aby se
    /// vodič nedotkl žádné jiné sítě (nejdřív uprostřed, pak dál od středu).
    /// Když takové místo není, vodič zůstane na místě a od vývodu se odpojí –
    /// raději viditelně volný konec než skrytý zkrat.
    ///
    /// [en] Returns a copy of the circuit in which the selected components and wires are
    /// moved by (dx, dy).
    ///
    /// Unselected wires connected to a pin of a moved component are rerouted: the
    /// end at the pin moves and the wire gets a bend. The bend position is chosen so
    /// that the wire touches no other net (first in the middle, then further from
    /// the middle). If there is no such place, the wire stays and is disconnected
    /// from the pin – a visibly loose end is better than a hidden short circuit.
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
        let mut stretched = Vec::new();
        for (i, w) in self.wires.iter().enumerate() {
            match ends_move(w) {
                (true, true) => {
                    c.wires[i].a = w.a.offset(dx, dy);
                    c.wires[i].b = w.b.offset(dx, dy);
                }
                (false, false) => {}
                (a_side, _) => stretched.push((i, a_side)),
            }
        }
        if stretched.is_empty() {
            return c;
        }
        // Sítě před přesunem: k nim se kontroluje, zda nová cesta nespojí cizí síť.
        // [en] Nets before the move: a new route is checked against them so that it does not join a foreign net.
        let old = crate::sim::Netlist::build(self);
        let mut net_of_wire = old.wire_net.clone();
        let pin_nets: Vec<(Point, usize)> = c
            .components
            .iter()
            .enumerate()
            .flat_map(|(ci, k)| {
                let ins = k.input_pins().into_iter().zip(old.input_nets[ci].clone());
                let outs = k.output_pins().into_iter().zip(old.output_nets[ci].clone());
                ins.chain(outs).collect::<Vec<_>>()
            })
            .collect();
        // Vodiče, které na přepojení teprve čekají, se kontrolují v původní
        // poloze – tam v nejhorším případě zůstanou. Kdo se nepřepojí napoprvé,
        // zkusí to znovu, až se ostatní vodiče uhnou (dokud se daří).
        // [en] Wires still waiting to be rerouted are checked in their original position –
        // in the worst case they stay there. A wire that fails the first time tries
        // again after the others have moved out of the way (as long as it helps).
        let mut todo = stretched;
        loop {
            let before = todo.len();
            todo.retain(|&(i, a_side)| {
                !self.reroute(
                    &mut c,
                    i,
                    a_side,
                    (dx, dy),
                    &ends_move,
                    &mut net_of_wire,
                    &pin_nets,
                )
            });
            if todo.is_empty() || todo.len() == before {
                break;
            }
        }
        c.wires.retain(|w| !w.is_empty());
        c.merge_collinear();
        c
    }

    /// Přepojí vodič `i` (v `c`) k posunutému konci. Vrací `false`, pokud se
    /// nenašla cesta, která by se nedotkla cizí sítě (vodič pak zůstane, kde byl).
    ///
    /// [en] Reroutes wire `i` (in `c`) to the moved end. Returns `false` if no route
    /// was found that touches no foreign net (the wire then stays where it was).
    #[allow(clippy::too_many_arguments)]
    fn reroute(
        &self,
        c: &mut Circuit,
        i: usize,
        a_side: bool,
        (dx, dy): (i32, i32),
        ends_move: &impl Fn(&Wire) -> (bool, bool),
        net_of_wire: &mut std::collections::HashMap<WireId, usize>,
        pin_nets: &[(Point, usize)],
    ) -> bool {
        let w = self.wires[i];
        let Some(&net) = net_of_wire.get(&w.id) else {
            return true;
        };
        let (moving, fixed) = if a_side { (w.a, w.b) } else { (w.b, w.a) };
        let new = moving.offset(dx, dy);
        let horizontal = moving.y == fixed.y;
        let check = |c: &Circuit, segs: &[(Point, Point)], skip: &[WireId], nets: &_| {
            route_is_clean(c, segs, net, skip, nets, pin_nets)
        };
        // 1) Pevný konec je roh lomeného vodiče: roh se posune s ním.
        // [en] 1) The fixed end is the corner of a bent wire: the corner moves with it.
        if let Some(j) = self.sole_partner(i, fixed, ends_move) {
            let corner = if horizontal {
                Point::new(fixed.x, new.y)
            } else {
                Point::new(new.x, fixed.y)
            };
            let pw = c.wires[j];
            let far = if pw.a == fixed { pw.b } else { pw.a };
            let segs = [(new, corner), (corner, far)];
            if check(c, &segs, &[w.id, pw.id], net_of_wire) {
                c.wires[i].a = new;
                c.wires[i].b = corner;
                let pw = &mut c.wires[j];
                if pw.a == fixed {
                    pw.a = corner;
                } else {
                    pw.b = corner;
                }
                return true;
            }
        }
        // 2) Lomená cesta se zalomením v různých místech.
        // [en] 2) A bent route with the bend at different places.
        let Some(segs) = route_candidates(new, fixed, horizontal)
            .into_iter()
            .find(|segs| check(c, segs, &[w.id], net_of_wire))
        else {
            return false;
        };
        c.wires[i].a = segs[0].0;
        c.wires[i].b = segs[0].1;
        for &(a, b) in &segs[1..] {
            if let Some(id) = c.add_wire(a, b) {
                net_of_wire.insert(id, net);
            }
        }
        true
    }

    /// Je bod `p` rohem, kde se vodič `i` stýká s jediným dalším kolmým vodičem
    /// (a s ničím jiným)? Pak vrátí index toho vodiče.
    ///
    /// [en] Is point `p` a corner where wire `i` meets exactly one other perpendicular
    /// wire (and nothing else)? Then returns the index of that wire.
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
    ///
    /// [en] Copies the selected components and wires (moved by 2 points) and returns the ids of the copies.
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

    /// Kopie vybraných součástek a vodičů jako samostatný obvod (pro schránku).
    /// Přibere i vodiče, které propojují jen vybrané součástky – celou skupinu
    /// spojených vodičů, jejíž všechny připojené vývody patří vybraným součástkám.
    ///
    /// [en] A copy of the selected components and wires as a separate circuit (for the
    /// clipboard). It also takes the wires that connect only selected components – a
    /// whole group of joined wires whose connected pins all belong to selected components.
    pub fn extract(&self, comps: &[CompId], wires: &[WireId]) -> Circuit {
        let components: Vec<Component> = self
            .components
            .iter()
            .filter(|c| comps.contains(&c.id))
            .cloned()
            .collect();
        let selected_pins: Vec<Point> = components.iter().flat_map(|c| c.pins()).collect();
        let other_pins: Vec<Point> = self
            .components
            .iter()
            .filter(|c| !comps.contains(&c.id))
            .flat_map(|c| c.pins())
            .collect();
        // Skupiny spojených vodičů (konec jednoho leží na druhém).
        // [en] Groups of joined wires (the end of one lies on the other).
        let n = self.wires.len();
        let mut group: Vec<usize> = (0..n).collect();
        fn root(g: &mut [usize], mut i: usize) -> usize {
            while g[i] != i {
                g[i] = g[g[i]];
                i = g[i];
            }
            i
        }
        for i in 0..n {
            for j in (i + 1)..n {
                let (a, b) = (&self.wires[i], &self.wires[j]);
                if a.contains(b.a) || a.contains(b.b) || b.contains(a.a) || b.contains(a.b) {
                    let (ri, rj) = (root(&mut group, i), root(&mut group, j));
                    group[ri] = rj;
                }
            }
        }
        let mut take = vec![false; n];
        for (i, t) in take.iter_mut().enumerate() {
            let r = root(&mut group, i);
            let members: Vec<&Wire> = (0..n)
                .filter(|&k| root(&mut group, k) == r)
                .map(|k| &self.wires[k])
                .collect();
            let touches = |pins: &[Point]| {
                members
                    .iter()
                    .any(|w| pins.contains(&w.a) || pins.contains(&w.b))
            };
            *t = wires.contains(&self.wires[i].id)
                || (touches(&selected_pins) && !touches(&other_pins));
        }
        Circuit {
            components,
            wires: (0..n).filter(|&i| take[i]).map(|i| self.wires[i]).collect(),
            next_id: 0,
        }
    }

    /// Vloží kopii jiného obvodu posunutou o (dx, dy) a vrátí id nových prvků.
    /// Popisky se zachovají, pokud ještě nejsou obsazené; jinak dostanou nový
    /// automatický popisek (jako při duplikaci).
    ///
    /// [en] Inserts a copy of another circuit moved by (dx, dy) and returns the ids of
    /// the new items. Labels are kept if they are not taken yet; otherwise they get a
    /// new automatic label (as when duplicating).
    pub fn insert(&mut self, other: &Circuit, dx: i32, dy: i32) -> (Vec<CompId>, Vec<WireId>) {
        let mut comps = Vec::new();
        for k in &other.components {
            let label_free =
                !k.label.is_empty() && !self.components.iter().any(|c| c.label == k.label);
            let id = self.add_component(k.kind, k.pos.offset(dx, dy));
            let copy = self.component_mut(id).expect("právě přidáno");
            copy.rotation = k.rotation;
            if label_free || matches!(k.kind, ComponentKind::Gate { .. }) {
                copy.label = k.label.clone();
            }
            comps.push(id);
        }
        let wires = other
            .wires
            .iter()
            .filter_map(|w| self.add_wire(w.a.offset(dx, dy), w.b.offset(dx, dy)))
            .collect();
        (comps, wires)
    }

    /// Sloučí úseky vodiče, které leží v jedné přímce a stýkají se v bodě, kde
    /// není nic jiného (vývod, další vodič). Rovný vodič pak nemá skrytý styčný
    /// bod, na který by se nechtěně napojil jiný vodič. Zapojení se nemění.
    ///
    /// [en] Merges wire segments that lie on one line and meet at a point where there is
    /// nothing else (pin, another wire). A straight wire then has no hidden joint
    /// another wire could unintentionally connect to. The connections do not change.
    pub fn merge_collinear(&mut self) {
        let pins: Vec<Point> = self.components.iter().flat_map(|c| c.pins()).collect();
        'outer: loop {
            for i in 0..self.wires.len() {
                for j in i + 1..self.wires.len() {
                    let (a, b) = (self.wires[i], self.wires[j]);
                    let same_line = (a.a.y == a.b.y && b.a.y == b.b.y && a.a.y == b.a.y)
                        || (a.a.x == a.b.x && b.a.x == b.b.x && a.a.x == b.a.x);
                    if !same_line {
                        continue;
                    }
                    let Some(p) = [a.a, a.b].into_iter().find(|p| b.is_endpoint(*p)) else {
                        continue;
                    };
                    let others = self.wires.iter().filter(|w| w.contains(p)).count();
                    if others != 2 || pins.contains(&p) {
                        continue;
                    }
                    let far_a = if a.a == p { a.b } else { a.a };
                    let far_b = if b.a == p { b.b } else { b.a };
                    // Úseky se nesmí překrývat (musí ležet na opačných stranách bodu).
                    // [en] The segments must not overlap (they must lie on opposite sides of the point).
                    let opposite = (far_a.x - p.x).signum() != (far_b.x - p.x).signum()
                        || (far_a.y - p.y).signum() != (far_b.y - p.y).signum();
                    if !opposite {
                        continue;
                    }
                    self.wires[i].a = far_a;
                    self.wires[i].b = far_b;
                    self.wires.remove(j);
                    continue 'outer;
                }
            }
            break;
        }
    }

    /// Body, ve kterých by se nový vodič z úseků `segs` spojil s existujícími
    /// vodiči nebo vývody (podle stejných pravidel jako netlist): konec cizího
    /// vodiče na novém úseku, konec nového úseku na cizím vodiči nebo vývodu.
    ///
    /// [en] Points where a new wire made of segments `segs` would connect to existing
    /// wires or pins (by the same rules as the netlist): the end of a foreign wire
    /// on a new segment, the end of a new segment on a foreign wire or pin.
    pub fn wire_contacts(&self, segs: &[(Point, Point)]) -> Vec<Point> {
        let mut out = Vec::new();
        let pins: Vec<Point> = self.components.iter().flat_map(|c| c.pins()).collect();
        for &(a, b) in segs {
            let s = Wire {
                id: WireId(u32::MAX),
                a,
                b,
            };
            for w in &self.wires {
                out.extend([w.a, w.b].into_iter().filter(|p| s.contains(*p)));
                out.extend([a, b].into_iter().filter(|p| w.contains(*p)));
            }
            out.extend([a, b].into_iter().filter(|p| pins.contains(p)));
        }
        out.sort();
        out.dedup();
        out
    }

    /// Body, ve kterých se kreslí spojovací tečka: setkávají se zde aspoň tři větve.
    ///
    /// [en] Points where a junction dot is drawn: at least three branches meet there.
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
    ///
    /// [en] Bounds of the whole circuit (min, max) in grid points.
    pub fn bounds(&self) -> Option<(f32, f32, f32, f32)> {
        let mut it = self
            .components
            .iter()
            .map(|c| {
                let (x0, y0, x1, y1) = c.body();
                // Rezerva na vývody a popisky.
                // [en] Margin for pins and labels.
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

/// Možné cesty z nového konce `new` k pevnému konci `fixed`: zalomení
/// postupně uprostřed, dál od středu, u pevného a nakonec u posunutého konce.
/// `horizontal` říká, jakým směrem vodič vede (zachová se u obou konců).
///
/// [en] Possible routes from the new end `new` to the fixed end `fixed`: the bend
/// first in the middle, then further from the middle, at the fixed end and
/// finally at the moved end. `horizontal` tells which way the wire runs (kept
/// at both ends).
fn route_candidates(new: Point, fixed: Point, horizontal: bool) -> Vec<Vec<(Point, Point)>> {
    let (from, to) = if horizontal {
        (new.x, fixed.x)
    } else {
        (new.y, fixed.y)
    };
    let (lo, hi) = (from.min(to), from.max(to));
    let mut cuts: Vec<i32> = (lo + 1..hi).collect();
    cuts.sort_by_key(|&v| ((2 * v - lo - hi).abs(), (v - to).abs()));
    cuts.extend([to, from]);
    cuts.dedup();
    let mut routes: Vec<Vec<(Point, Point)>> = Vec::new();
    for v in cuts {
        let (p, q) = if horizontal {
            (Point::new(v, new.y), Point::new(v, fixed.y))
        } else {
            (Point::new(new.x, v), Point::new(fixed.x, v))
        };
        let segs: Vec<(Point, Point)> = [(new, p), (p, q), (q, fixed)]
            .into_iter()
            .filter(|(a, b)| a != b)
            .collect();
        if !segs.is_empty() && !routes.contains(&segs) {
            routes.push(segs);
        }
    }
    routes
}

/// Nedotkne se cesta `segs` (síť `net`) žádného vývodu ani vodiče jiné sítě?
/// Pouhé křížení vodičů nevadí – nepropojí se.
///
/// [en] Does the route `segs` (net `net`) avoid touching any pin or wire of another
/// net? Merely crossing wires is fine – they do not connect.
fn route_is_clean(
    c: &Circuit,
    segs: &[(Point, Point)],
    net: usize,
    skip: &[WireId],
    net_of_wire: &std::collections::HashMap<WireId, usize>,
    pin_nets: &[(Point, usize)],
) -> bool {
    let seg = |a: Point, b: Point| Wire {
        id: WireId(u32::MAX),
        a,
        b,
    };
    segs.iter().all(|&(a, b)| {
        let s = seg(a, b);
        let pins_ok = pin_nets.iter().all(|&(p, m)| m == net || !s.contains(p));
        let wires_ok = c.wires.iter().all(|o| {
            skip.contains(&o.id)
                || net_of_wire.get(&o.id) == Some(&net)
                || !(o.contains(a) || o.contains(b) || s.contains(o.a) || s.contains(o.b))
        });
        pins_ok && wires_ok
    })
}

impl Circuit {
    /// Vstupy (přepínače, tlačítka) a výstupy (LED) obvodu seřazené podle popisku.
    /// U podobvodu tvoří jeho rozhraní – vývody značky shora dolů.
    ///
    /// [en] Inputs (switches, buttons) and outputs (LEDs) of the circuit sorted by label.
    /// For a subcircuit they form its interface – the symbol's pins from top to bottom.
    pub fn io(&self) -> (Vec<&Component>, Vec<&Component>) {
        let mut ins: Vec<&Component> = self
            .components
            .iter()
            .filter(|c| c.kind.is_input())
            .collect();
        let mut outs: Vec<&Component> = self
            .components
            .iter()
            .filter(|c| matches!(c.kind, ComponentKind::Led))
            .collect();
        ins.sort_by(|a, b| natural_cmp(&a.label, &b.label));
        outs.sort_by(|a, b| natural_cmp(&a.label, &b.label));
        (ins, outs)
    }
}

/// Porovnání popisků „přirozeně“: A2 < A10.
///
/// [en] “Natural” comparison of labels: A2 < A10.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let split = |s: &str| {
        let i = s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len());
        (s[..i].to_owned(), s[i..].parse::<u64>().unwrap_or(0))
    };
    split(a).cmp(&split(b)).then_with(|| a.cmp(b))
}

/// Nejvyšší počet vývodů podobvodu na každé straně.
///
/// [en] Largest number of subcircuit pins on each side.
pub const MAX_SUBCIRCUIT_PINS: usize = 32;

/// Definice podobvodu: pojmenovaný obvod, který lze vložit jako součástku.
///
/// [en] Subcircuit definition: a named circuit that can be inserted as a component.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubcircuitDef {
    /// Identifikátor definice (na něj odkazují instance).
    ///
    /// [en] Definition identifier (instances refer to it).
    pub id: u32,
    /// Název podobvodu zobrazený na značce.
    ///
    /// [en] Subcircuit name shown on the symbol.
    pub name: String,
    /// Zapojení podobvodu.
    ///
    /// [en] The subcircuit's circuit.
    pub circuit: Circuit,
}

impl SubcircuitDef {
    /// Názvy vstupů a výstupů značky (popisky přepínačů a LED uvnitř).
    ///
    /// [en] Names of the symbol's inputs and outputs (labels of the switches and LEDs inside).
    pub fn pin_names(&self) -> (Vec<String>, Vec<String>) {
        let (ins, outs) = self.circuit.io();
        let names = |v: Vec<&Component>| {
            v.into_iter()
                .take(MAX_SUBCIRCUIT_PINS)
                .map(|c| c.label.clone())
                .collect()
        };
        (names(ins), names(outs))
    }

    /// Druh součástky pro vložení instance.
    ///
    /// [en] Kind of component for inserting an instance.
    pub fn kind(&self) -> ComponentKind {
        let (ins, outs) = self.pin_names();
        ComponentKind::Subcircuit {
            id: self.id,
            inputs: ins.len() as u8,
            outputs: outs.len() as u8,
        }
    }

    /// Obsahuje tento podobvod (i nepřímo) instanci podobvodu `target`?
    ///
    /// [en] Does this subcircuit contain (even indirectly) an instance of subcircuit `target`?
    pub fn uses(&self, target: u32, defs: &[SubcircuitDef]) -> bool {
        circuit_uses(&self.circuit, target, defs, 0)
    }
}

/// Obsahuje obvod (i nepřímo) instanci podobvodu `target`?
///
/// [en] Does the circuit contain (even indirectly) an instance of subcircuit `target`?
pub fn circuit_uses(c: &Circuit, target: u32, defs: &[SubcircuitDef], depth: usize) -> bool {
    depth < 32
        && c.components.iter().any(|k| match k.kind {
            ComponentKind::Subcircuit { id, .. } => {
                id == target
                    || defs
                        .iter()
                        .find(|d| d.id == id)
                        .is_some_and(|d| circuit_uses(&d.circuit, target, defs, depth + 1))
            }
            _ => false,
        })
}

/// Upraví počty vývodů instancí podobvodů v obvodu podle aktuálních definic.
/// Vrací `true`, pokud se něco změnilo.
///
/// [en] Updates the pin counts of subcircuit instances in the circuit according to
/// the current definitions. Returns `true` if anything changed.
pub fn sync_instances(circuit: &mut Circuit, defs: &[SubcircuitDef]) -> bool {
    let mut changed = false;
    for c in &mut circuit.components {
        if let ComponentKind::Subcircuit { id, .. } = c.kind
            && let Some(d) = defs.iter().find(|d| d.id == id)
        {
            let kind = d.kind();
            if c.kind != kind {
                c.kind = kind;
                changed = true;
            }
        }
    }
    changed
}

/// Formát souboru projektu `.logicflow`.
///
/// [en] Format of the `.logicflow` project file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Verze formátu – zvýší se při nekompatibilní změně.
    ///
    /// [en] Format version – increased on an incompatible change.
    pub format: u32,
    /// Název a verze aplikace, která soubor uložila.
    ///
    /// [en] Name and version of the application that saved the file.
    pub app: String,
    /// Hlavní obvod.
    ///
    /// [en] Main circuit.
    pub circuit: Circuit,
    /// Definice podobvodů (od verze 0.3).
    ///
    /// [en] Subcircuit definitions (since version 0.3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subcircuits: Vec<SubcircuitDef>,
}

impl ProjectFile {
    /// Aktuální verze formátu.
    ///
    /// [en] Current format version.
    pub const FORMAT: u32 = 1;
    /// Přípona souboru.
    ///
    /// [en] File extension.
    pub const EXTENSION: &'static str = "logicflow";

    /// Uloží obvod a podobvody jako formátovaný JSON.
    ///
    /// [en] Saves the circuit and subcircuits as formatted JSON.
    pub fn to_json(circuit: &Circuit, subcircuits: &[SubcircuitDef]) -> String {
        let file = ProjectFile {
            format: Self::FORMAT,
            app: format!("LogicFlow {}", crate::VERSION),
            circuit: circuit.clone(),
            subcircuits: subcircuits.to_vec(),
        };
        serde_json::to_string_pretty(&file).expect("serializace obvodu nemůže selhat")
    }

    /// Načte obvod a podobvody z JSON; odmítne soubor z novější verze formátu.
    ///
    /// [en] Loads the circuit and subcircuits from JSON; rejects a file in a newer format version.
    pub fn from_json(text: &str) -> Result<(Circuit, Vec<SubcircuitDef>), String> {
        let file: ProjectFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if file.format > Self::FORMAT {
            return Err(format!(
                "Soubor je z novější verze aplikace (formát {}), tato verze umí formát {}.",
                file.format,
                Self::FORMAT
            ));
        }
        Ok((file.circuit, file.subcircuits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_takes_wires_between_selected() {
        // A → AND ← B, AND → LED; vybrané jen A, B a hradlo (bez vodičů).
        // [en] A → AND ← B, AND → LED; only A, B and the gate selected (no wires).
        let mut c = Circuit::default();
        let a = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let b = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 4));
        let g = c.add_component(ComponentKind::gate(GateKind::And), Point::new(10, 1));
        c.add_component(ComponentKind::Led, Point::new(20, 1));
        // A (2,0) → (7,0); B (2,4) → (5,4) → (5,2) → (7,2); hradlo (13,1) → LED (18,1)
        // [en] A (2,0) → (7,0); B (2,4) → (5,4) → (5,2) → (7,2); gate (13,1) → LED (18,1)
        c.add_wire(Point::new(2, 0), Point::new(7, 0)).unwrap();
        c.add_wire(Point::new(2, 4), Point::new(5, 4)).unwrap();
        c.add_wire(Point::new(5, 4), Point::new(5, 2)).unwrap();
        c.add_wire(Point::new(5, 2), Point::new(7, 2)).unwrap();
        c.add_wire(Point::new(13, 1), Point::new(18, 1)).unwrap();
        let clip = c.extract(&[a, b, g], &[]);
        assert_eq!(clip.components.len(), 3);
        // Vodič k LED vede k nevybrané součástce – nepřibere se.
        // [en] The wire to the LED leads to an unselected component – it is not taken.
        assert_eq!(clip.wires.len(), 4);
        // Vložení: popisky A, B jsou obsazené → nové; posun o 30.
        // [en] Insert: labels A, B are taken → new ones; offset 30.
        let (comps, wires) = c.insert(&clip, 30, 0);
        assert_eq!((comps.len(), wires.len()), (3, 4));
        let labels: Vec<&str> = comps
            .iter()
            .map(|id| c.component(*id).unwrap().label.as_str())
            .collect();
        assert_eq!(labels, ["C", "D", ""]);
        assert_eq!(c.component(comps[0]).unwrap().pos, Point::new(30, 0));
        // Do prázdného obvodu se popisky zachovají.
        // [en] Into an empty circuit the labels are kept.
        let mut empty = Circuit::default();
        let (comps, _) = empty.insert(&clip, 0, 0);
        assert_eq!(empty.component(comps[1]).unwrap().label, "B");
    }

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
        // [en] After rotating by 90° the output points down.
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
        // [en] A crossing without a connection has no dot.
        c.add_wire(Point::new(8, -3), Point::new(8, 3));
        assert_eq!(c.junctions(), vec![Point::new(5, 0)]);
    }

    #[test]
    fn json_roundtrip_and_ids_continue() {
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        c.add_wire(Point::new(2, 0), Point::new(6, 0));
        let text = ProjectFile::to_json(&c, &[]);
        let (mut loaded, _) = ProjectFile::from_json(&text).unwrap();
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
        // Vodič se zalomí uprostřed (vodorovná – svislá – vodorovná část)
        // a spojení zůstane.
        // [en] The wire bends in the middle (horizontal – vertical – horizontal part)
        // and the connection stays.
        assert_eq!(m.wires.len(), 3);
        assert!(
            m.wires
                .iter()
                .any(|w| (w.a, w.b) == (Point::new(6, 3), Point::new(6, 0)))
        );
        let mut sim = crate::Simulator::new(&m, None);
        sim.run(100);
        assert_eq!(sim.inputs(led)[0], LogicValue::One);
        // Původní obvod zůstal beze změny.
        // [en] The original circuit stayed unchanged.
        assert_eq!(c.wires.len(), 1);
    }

    #[test]
    fn display_pin_sides() {
        // Soubor z verze bez volby strany se načte s vývody vlevo.
        // [en] A file from a version without the side option loads with pins on the left.
        let old: ComponentKind = serde_json::from_str(r#"{"type":"seven_segment"}"#).unwrap();
        assert_eq!(
            old,
            ComponentKind::SevenSegment {
                pins: PinSide::Left
            }
        );
        for side in PinSide::ALL {
            for kind in [
                ComponentKind::SevenSegment { pins: side },
                ComponentKind::HexDisplay { pins: side },
            ] {
                let (x0, y0, x1, y1) = kind.body_local();
                let (ins, _) = kind.pins_local();
                for &(x, y) in &ins {
                    let (x, y) = (x as f32, y as f32);
                    let on_side = match side {
                        PinSide::Left => x < x0 && y > y0 && y < y1,
                        PinSide::Right => x > x1 && y > y0 && y < y1,
                        PinSide::Top => y < y0 && x > x0 && x < x1,
                        PinSide::Bottom => y > y1 && x > x0 && x < x1,
                    };
                    assert!(on_side, "{kind:?} vývod ({x},{y})");
                }
            }
        }
    }

    #[test]
    fn collinear_pieces_merge() {
        let mut c = Circuit::default();
        c.add_wire(Point::new(0, 0), Point::new(4, 0));
        c.add_wire(Point::new(4, 0), Point::new(9, 0));
        // Odbočka v (9,0) zůstane – tam se stýkají tři větve.
        // [en] The branch at (9,0) stays – three branches meet there.
        c.add_wire(Point::new(9, 0), Point::new(12, 0));
        c.add_wire(Point::new(9, 0), Point::new(9, 5));
        c.merge_collinear();
        assert_eq!(c.wires.len(), 3);
        assert!(
            c.wires
                .iter()
                .any(|w| (w.a, w.b) == (Point::new(0, 0), Point::new(9, 0)))
        );
    }

    #[test]
    fn rerouting_avoids_other_pins() {
        // Přepínač → vstup C čítače; sousední vstup EN je o 2 body výš.
        // [en] Switch → input C of the counter; the neighbouring input EN is 2 points higher.
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 1));
        let block = crate::BlockKind::Counter {
            bits: 4,
            down: false,
        };
        let ctr = c.add_component(ComponentKind::Block { block }, Point::new(12, 0));
        c.add_wire(Point::new(2, 1), Point::new(8, 1));
        let m = c.moved(&[sw], &[], 0, -2);
        // Zalomení nesmí vést přes vstup EN (8,-1).
        // [en] The bend must not lead over input EN (8,-1).
        let en = m.component(ctr).unwrap().input_pins()[1];
        assert!(m.wires.iter().all(|w| !w.contains(en)));
        let nl = crate::sim::Netlist::build(&m);
        let i = m.components.iter().position(|k| k.id == ctr).unwrap();
        assert_ne!(
            nl.input_nets[i][1], nl.input_nets[i][2],
            "EN a C zůstanou oddělené"
        );
    }

    #[test]
    fn wire_over_pin_does_not_connect() {
        // Vodič vede přes vývod LED, ale nekončí v něm.
        // [en] The wire runs over the LED's pin but does not end there.
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        let led = c.add_component(ComponentKind::Led, Point::new(6, 0));
        c.add_wire(Point::new(2, 0), Point::new(2, 3));
        c.add_wire(Point::new(4, -2), Point::new(4, 2));
        let mut sim = crate::Simulator::new(&c, None);
        sim.run(100);
        assert_eq!(sim.inputs(led)[0], LogicValue::Z);
        let _ = sw;
    }

    #[test]
    fn moving_component_slides_corner() {
        let mut c = Circuit::default();
        let sw = c.add_component(ComponentKind::Switch { on: true }, Point::new(0, 0));
        let led = c.add_component(ComponentKind::Led, Point::new(12, 6));
        // lomený vodič: (2,0) → (6,0) → (6,6) → (10,6)
        // [en] bent wire: (2,0) → (6,0) → (6,6) → (10,6)
        c.add_wire(Point::new(2, 0), Point::new(6, 0));
        c.add_wire(Point::new(6, 0), Point::new(6, 6));
        c.add_wire(Point::new(6, 6), Point::new(10, 6));
        let m = c.moved(&[sw], &[], 0, 2);
        // Žádný nový úsek, roh se jen posunul.
        // [en] No new segment, the corner just moved.
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
