//! Kreslení značek součástek (ČSN EN 60617, obdélníkové značky).
//!
//! [en] Drawing of component symbols (EN 60617, rectangular symbols).

use crate::theme::{colors, signal_color};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2, pos2, vec2};
use logicflow_core::block::{SEVEN_SEG_HEX, number};
use logicflow_core::{
    BlockKind, Component, ComponentKind, FlipFlopKind, LogicValue, PinSide, Rotation,
};

/// Styl značek: evropský (ČSN EN 60617-12) nebo americký (ANSI/IEEE Std 91).
///
/// [en] Symbol style: European (EN 60617-12) or American (ANSI/IEEE Std 91).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SymbolStyle {
    /// Evropské obdélníkové značky.
    ///
    /// [en] European rectangular symbols.
    #[default]
    Iec,
    /// Americké tvarové značky.
    ///
    /// [en] American distinctive-shape symbols.
    Ansi,
}

/// Klíč, pod kterým je styl uložený v datech egui.
///
/// [en] Key under which the style is stored in egui data.
fn style_id() -> egui::Id {
    egui::Id::unique("logicflow-symbol-style")
}

/// Nastaví styl značek pro kreslení v tomto snímku.
///
/// [en] Sets the symbol style for drawing in this frame.
pub fn set_style(ctx: &egui::Context, style: SymbolStyle) {
    ctx.data_mut(|d| d.insert_temp(style_id(), style));
}

/// Styl značek platný pro tento snímek.
///
/// [en] Symbol style valid for this frame.
fn style(painter: &Painter) -> SymbolStyle {
    painter
        .ctx()
        .data(|d| d.get_temp(style_id()))
        .unwrap_or_default()
}

/// Velikost jedné jednotky mřížky v pixelech při zoomu 100 %.
///
/// [en] Size of one grid unit in pixels at 100 % zoom.
pub const UNIT: f32 = 16.0;

/// Převod mezi světovými (mřížka) a obrazovkovými souřadnicemi.
///
/// [en] Conversion between world (grid) and screen coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Poloha bodu mřížky (0, 0) na obrazovce.
    ///
    /// [en] Screen position of grid point (0, 0).
    pub origin: Pos2,
    /// Počet pixelů na jednotku mřížky.
    ///
    /// [en] Pixels per grid unit.
    pub scale: f32,
}

impl Camera {
    /// Světový bod na obrazovce.
    ///
    /// [en] World point on screen.
    pub fn to_screen(self, p: Pos2) -> Pos2 {
        self.origin + p.to_vec2() * self.scale
    }

    /// Bod obrazovky ve světových souřadnicích.
    ///
    /// [en] Screen point in world coordinates.
    pub fn to_world(self, p: Pos2) -> Pos2 {
        ((p - self.origin) / self.scale).to_pos2()
    }

    /// Délka ve světových jednotkách převedená na pixely.
    ///
    /// [en] Length in world units converted to pixels.
    pub fn len(self, l: f32) -> f32 {
        l * self.scale
    }

    /// Obdélník ve světových souřadnicích převedený na obrazovku.
    ///
    /// [en] Rectangle in world coordinates converted to the screen.
    pub fn rect(self, (x0, y0, x1, y1): (f32, f32, f32, f32)) -> Rect {
        Rect::from_min_max(self.to_screen(pos2(x0, y0)), self.to_screen(pos2(x1, y1)))
    }

    /// Tloušťka čáry podle zvětšení (1–3 px).
    ///
    /// [en] Line width according to the zoom (1–3 px).
    pub fn line_width(self) -> f32 {
        self.len(0.12).clamp(1.0, 3.0)
    }
}

/// Stav, ve kterém se součástka kreslí.
///
/// [en] State in which the component is drawn.
#[derive(Clone, Copy, Default)]
pub struct Look {
    /// Součástka je vybraná.
    ///
    /// [en] The component is selected.
    pub selected: bool,
    /// Ukazatel myši je nad součástkou.
    ///
    /// [en] The mouse pointer is over the component.
    pub hovered: bool,
    /// Poloprůhledný náhled při umisťování.
    ///
    /// [en] Semi-transparent preview while placing.
    pub ghost: bool,
}

/// Hodnoty potřebné ke kreslení (ze simulátoru).
///
/// [en] Values needed for drawing (from the simulator).
#[derive(Default)]
pub struct Values {
    /// Hodnoty na vstupech.
    ///
    /// [en] Values at the inputs.
    pub inputs: Vec<LogicValue>,
    /// Hodnoty výstupů.
    ///
    /// [en] Output values.
    pub outputs: Vec<LogicValue>,
    /// Název a názvy vývodů podobvodu (jen u instancí podobvodů).
    ///
    /// [en] Name and pin names of a subcircuit (only for subcircuit instances).
    pub sub: Option<SubInfo>,
}

/// Popis podobvodu pro kreslení jeho značky.
///
/// [en] Description of a subcircuit for drawing its symbol.
pub struct SubInfo {
    /// Název podobvodu.
    ///
    /// [en] Subcircuit name.
    pub name: String,
    /// Názvy vstupů shora dolů.
    ///
    /// [en] Input names from top to bottom.
    pub inputs: Vec<String>,
    /// Názvy výstupů shora dolů.
    ///
    /// [en] Output names from top to bottom.
    pub outputs: Vec<String>,
}

impl SubInfo {
    /// Popis podobvodu pro instanci druhu `kind` (None, pokud to není podobvod).
    ///
    /// [en] Subcircuit description for an instance of kind `kind` (None if it is not a subcircuit).
    pub fn of(project: &crate::project::Project, kind: ComponentKind) -> Option<Self> {
        let ComponentKind::Subcircuit { id, .. } = kind else {
            return None;
        };
        let def = project.subcircuit(id)?;
        let (inputs, outputs) = def.pin_names();
        Some(Self {
            name: def.name.clone(),
            inputs,
            outputs,
        })
    }
}

/// Barva obrysu vybrané součástky.
///
/// [en] Outline colour of a selected component.
pub const SELECTED: Color32 = Color32::from_rgb(0xff, 0xa0, 0x30);

/// Zeslabí barvu u poloprůhledného náhledu.
///
/// [en] Dims the colour for a semi-transparent preview.
fn fade(c: Color32, look: Look) -> Color32 {
    if look.ghost {
        c.gamma_multiply(0.45)
    } else {
        c
    }
}

/// Místní bod značky ve světových souřadnicích (s otočením).
///
/// [en] Local point of a symbol in world coordinates (with rotation).
fn local(comp: &Component, x: f32, y: f32) -> Pos2 {
    let (rx, ry) = comp.rotation.apply_f(x, y);
    pos2(comp.pos.x as f32 + rx, comp.pos.y as f32 + ry)
}

/// Zarovnání popisku podle směru, kterým od součástky míří.
///
/// [en] Label alignment according to the direction in which it points away from the component.
fn align_for(rot: Rotation, dx: f32) -> Align2 {
    align_dir(rot, dx, 0.0)
}

/// Zarovnání textu, který od bodu pokračuje směrem (`dx`, `dy`) v místních souřadnicích.
///
/// [en] Alignment of text that continues from a point in direction (`dx`, `dy`) in local coordinates.
fn align_dir(rot: Rotation, dx: f32, dy: f32) -> Align2 {
    let (x, y) = rot.apply_f(dx, dy);
    if x < -0.5 {
        Align2::RIGHT_CENTER
    } else if x > 0.5 {
        Align2::LEFT_CENTER
    } else if y < 0.0 {
        Align2::CENTER_BOTTOM
    } else {
        Align2::CENTER_TOP
    }
}

/// Text ve světových souřadnicích; při malém zvětšení se nekreslí.
///
/// [en] Text in world coordinates; not drawn when zoomed out too far.
fn text(
    painter: &Painter,
    cam: Camera,
    p: Pos2,
    s: &str,
    size: f32,
    align: Align2,
    color: Color32,
) {
    if cam.len(size) < 5.0 {
        return;
    }
    painter.text(
        cam.to_screen(p),
        align,
        s,
        FontId::proportional(cam.len(size)),
        color,
    );
}

/// Nakreslí součástku včetně vývodů, obarvených podle hodnot ze simulátoru.
///
/// [en] Draws a component including its pins, coloured by the values from the simulator.
pub fn draw(painter: &Painter, cam: Camera, comp: &Component, values: &Values, look: Look) {
    let lw = cam.line_width();
    let outline = if look.selected {
        Stroke::new(lw + 1.0, SELECTED)
    } else if look.hovered {
        Stroke::new(lw, Color32::WHITE)
    } else {
        Stroke::new(lw, fade(colors::SYMBOL, look))
    };
    let (x0, y0, x1, y1) = comp.kind.body_local();
    let (ins, outs) = comp.kind.pins_local();
    // Vývod vede od nejbližšího bodu obrysu těla.
    // [en] A pin starts at the nearest point of the body's outline.
    let edge = |px: i32, py: i32| local(comp, (px as f32).clamp(x0, x1), (py as f32).clamp(y0, y1));
    let inverted = matches!(comp.kind, ComponentKind::Gate { gate, .. } if gate.is_inverted());
    let lead = Stroke::new(lw, fade(colors::SYMBOL, look));

    // Vývody: krátká čára od těla k bodu mřížky a tečka obarvená hodnotou.
    // [en] Pins: a short line from the body to the grid point and a dot coloured by the value.
    let pin_color = |v: LogicValue| fade(signal_color(v), look);
    for (i, (px, py)) in ins.iter().enumerate() {
        let v = values.inputs.get(i).copied().unwrap_or(LogicValue::Z);
        let a = edge(*px, *py);
        let b = local(comp, *px as f32, *py as f32);
        painter.line_segment([cam.to_screen(a), cam.to_screen(b)], lead);
        painter.circle_filled(cam.to_screen(b), cam.len(0.16).max(2.0), pin_color(v));
    }
    for (k, (px, py)) in outs.iter().enumerate() {
        let v = values.outputs.get(k).copied().unwrap_or(LogicValue::Z);
        let a = edge(*px, *py);
        let b = local(comp, *px as f32, *py as f32);
        painter.line_segment([cam.to_screen(a), cam.to_screen(b)], lead);
        painter.circle_filled(cam.to_screen(b), cam.len(0.16).max(2.0), pin_color(v));
    }

    let body = cam.rect(comp.body());
    let fill = fade(colors::SYMBOL_FILL, look);
    match comp.kind {
        ComponentKind::Gate { gate, .. } if style(painter) == SymbolStyle::Ansi => {
            draw_ansi_gate(painter, cam, comp, gate, outline, look);
        }
        ComponentKind::Gate { gate, inputs } => {
            painter.rect_filled(body, cam.len(0.15), fill);
            painter.rect_stroke(body, cam.len(0.15), outline, egui::StrokeKind::Middle);
            if inverted {
                negation(painter, cam, comp, x1, 0.0, 1.0, outline);
            }
            // Značka v horní části těla (u otočených hradel uprostřed).
            // [en] Symbol in the upper part of the body (in the middle for rotated gates).
            let label_pos = if gate.is_unary() || inputs <= 1 || comp.rotation.is_vertical() {
                local(comp, 0.0, 0.0)
            } else {
                let (_, y0, _, _) = comp.body();
                pos2(comp.pos.x as f32, y0 + 1.3)
            };
            text(
                painter,
                cam,
                label_pos,
                gate.iec_label(),
                1.4,
                Align2::CENTER_CENTER,
                fade(colors::TEXT, look),
            );
            if !comp.label.is_empty() {
                let (_, y0, _, _) = comp.body();
                text(
                    painter,
                    cam,
                    pos2(comp.pos.x as f32, y0 - 0.4),
                    &comp.label,
                    0.9,
                    Align2::CENTER_BOTTOM,
                    fade(colors::TEXT_WEAK, look),
                );
            }
        }
        ComponentKind::Switch { on } => {
            painter.rect_filled(body, cam.len(0.3), fill);
            painter.rect_stroke(body, cam.len(0.3), outline, egui::StrokeKind::Middle);
            // Jezdec: vpravo (ve směru výstupu) = 1.
            // [en] Slider: to the right (towards the output) = 1.
            let dir = if on { 0.55 } else { -0.55 };
            let c = local(comp, dir, 0.0);
            let size = if comp.rotation.is_vertical() {
                vec2(1.3, 1.0)
            } else {
                vec2(1.0, 1.3)
            };
            let knob = Rect::from_center_size(cam.to_screen(c), size * cam.scale);
            painter.rect_filled(knob, cam.len(0.2), fade(signal_color(on.into()), look));
            text(
                painter,
                cam,
                c,
                if on { "1" } else { "0" },
                0.9,
                Align2::CENTER_CENTER,
                Color32::BLACK,
            );
            draw_label(painter, cam, comp, -1.8, look);
        }
        ComponentKind::Button => {
            painter.rect_filled(body, cam.len(0.3), fill);
            painter.rect_stroke(body, cam.len(0.3), outline, egui::StrokeKind::Middle);
            let c = cam.to_screen(local(comp, 0.0, 0.0));
            let pressed = values.outputs.first() == Some(&LogicValue::One);
            let fill = if pressed {
                signal_color(LogicValue::One)
            } else {
                Color32::from_rgb(0x50, 0x50, 0x50)
            };
            painter.circle_filled(c, cam.len(0.55), fade(fill, look));
            painter.circle_stroke(
                c,
                cam.len(0.55),
                Stroke::new(1.0, fade(colors::SYMBOL, look)),
            );
            draw_label(painter, cam, comp, -1.8, look);
        }
        ComponentKind::Constant { value } => {
            painter.rect_filled(body, cam.len(0.2), fill);
            painter.rect_stroke(body, cam.len(0.2), outline, egui::StrokeKind::Middle);
            let v: LogicValue = value.into();
            text(
                painter,
                cam,
                local(comp, 0.0, 0.0),
                if value { "1" } else { "0" },
                1.1,
                Align2::CENTER_CENTER,
                fade(signal_color(v), look),
            );
        }
        ComponentKind::FlipFlop { ff, falling, .. } => {
            draw_flip_flop(painter, cam, comp, ff, falling, outline, look);
        }
        ComponentKind::Clock { period_ms } => {
            painter.rect_filled(body, cam.len(0.3), fill);
            painter.rect_stroke(body, cam.len(0.3), outline, egui::StrokeKind::Middle);
            // Obdélníkový průběh obarvený aktuální hodnotou.
            // [en] Square wave coloured by the current value.
            let v = values.outputs.first().copied().unwrap_or(LogicValue::Z);
            let wave = [
                (-1.0, 0.45),
                (-0.5, 0.45),
                (-0.5, -0.45),
                (0.0, -0.45),
                (0.0, 0.45),
                (0.5, 0.45),
                (0.5, -0.45),
                (1.0, -0.45),
            ];
            let pts: Vec<Pos2> = wave
                .iter()
                .map(|&(x, y)| cam.to_screen(local(comp, x, y)))
                .collect();
            painter.add(egui::Shape::line(
                pts,
                Stroke::new(lw.max(1.5), fade(signal_color(v), look)),
            ));
            draw_label(painter, cam, comp, -2.0, look);
            let hz = 1000.0 / period_ms.max(1) as f32;
            let (_, _, _, y1) = comp.body();
            text(
                painter,
                cam,
                pos2(comp.pos.x as f32, y1 + 0.3),
                &format_hz(hz),
                0.8,
                Align2::CENTER_TOP,
                fade(colors::TEXT_WEAK, look),
            );
        }
        ComponentKind::Led => {
            let c = cam.to_screen(local(comp, 0.0, 0.0));
            let v = values.inputs.first().copied().unwrap_or(LogicValue::Z);
            let fill = match v {
                LogicValue::One => {
                    painter.circle_filled(
                        c,
                        cam.len(1.5),
                        fade(Color32::from_rgba_unmultiplied(255, 60, 40, 40), look),
                    );
                    Color32::from_rgb(0xff, 0x45, 0x30)
                }
                LogicValue::Zero => Color32::from_rgb(0x55, 0x22, 0x1e),
                LogicValue::X => Color32::from_rgb(0x6a, 0x30, 0x30),
                LogicValue::Z => Color32::from_rgb(0x2a, 0x2a, 0x34),
            };
            painter.circle_filled(c, cam.len(0.9), fade(fill, look));
            painter.circle_stroke(c, cam.len(0.9), outline);
            if matches!(v, LogicValue::X | LogicValue::Z) {
                text(
                    painter,
                    cam,
                    local(comp, 0.0, 0.0),
                    &v.to_string(),
                    0.9,
                    Align2::CENTER_CENTER,
                    colors::TEXT,
                );
            }
            draw_label(painter, cam, comp, 1.6, look);
        }
        ComponentKind::Block { block } => draw_block(painter, cam, comp, block, outline, look),
        ComponentKind::Subcircuit { .. } => {
            draw_subcircuit(painter, cam, comp, values.sub.as_ref(), outline, look);
        }
        ComponentKind::SevenSegment { pins } => {
            draw_display_body(painter, cam, comp, pins, outline, look, &SEG_NAMES);
            let segs: Vec<LogicValue> = (0..8)
                .map(|k| values.inputs.get(k).copied().unwrap_or(LogicValue::Z))
                .collect();
            let geom = match pins {
                PinSide::Left => (0.5, 1.6, 3.4),
                PinSide::Right => (-1.1, 1.6, 3.4),
                PinSide::Top | PinSide::Bottom => (-0.3, 1.6, 3.4),
            };
            draw_digit(painter, cam, comp, geom, &segs, look);
        }
        ComponentKind::HexDisplay { pins } => {
            draw_display_body(
                painter,
                cam,
                comp,
                pins,
                outline,
                look,
                &["D0", "D1", "D2", "D3"],
            );
            let segs: Vec<LogicValue> = match number(&values.inputs) {
                Some(n) if values.inputs.len() == 4 => (0..7)
                    .map(|k| (SEVEN_SEG_HEX[n as usize] >> k & 1 == 1).into())
                    .chain([LogicValue::Zero])
                    .collect(),
                _ => vec![LogicValue::X; 8],
            };
            let geom = match pins {
                PinSide::Left => (0.7, 1.4, 2.8),
                PinSide::Right => (-1.0, 1.4, 2.8),
                PinSide::Top | PinSide::Bottom => (-0.2, 1.2, 2.3),
            };
            draw_digit(painter, cam, comp, geom, &segs, look);
        }
    }
}

/// Názvy vstupů sedmisegmentovky.
///
/// [en] Input names of the seven-segment display.
const SEG_NAMES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "dp"];

/// Barva segmentu zobrazovače podle hodnoty.
///
/// [en] Display segment colour according to the value.
fn segment_color(v: LogicValue) -> Color32 {
    match v {
        LogicValue::One => Color32::from_rgb(0xff, 0x45, 0x30),
        LogicValue::Zero => Color32::from_rgb(0x3a, 0x1c, 0x1a),
        LogicValue::X => Color32::from_rgb(0x5a, 0x3a, 0x3a),
        LogicValue::Z => Color32::from_rgb(0x2a, 0x2a, 0x34),
    }
}

/// Tmavé tělo zobrazovače s názvy vstupů a popiskem nad ním.
///
/// [en] Dark display body with input names and a label above it.
fn draw_display_body(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    side: PinSide,
    outline: Stroke,
    look: Look,
    names: &[&str],
) {
    let body = cam.rect(comp.body());
    painter.rect_filled(
        body,
        cam.len(0.3),
        fade(Color32::from_rgb(0x16, 0x16, 0x1a), look),
    );
    painter.rect_stroke(body, cam.len(0.3), outline, egui::StrokeKind::Middle);
    let (x0, y0, x1, y1) = comp.kind.body_local();
    let (ins, _) = comp.kind.pins_local();
    for ((px, py), name) in ins.iter().zip(names) {
        // Název vstupu uvnitř těla u jeho vývodu, text míří dovnitř.
        // [en] Input name inside the body at its pin, the text points inwards.
        let (x, y, dx, dy) = match side {
            PinSide::Left => (x0 + 0.3, *py as f32, 1.0, 0.0),
            PinSide::Right => (x1 - 0.3, *py as f32, -1.0, 0.0),
            PinSide::Top => (*px as f32, y0 + 0.2, 0.0, 1.0),
            PinSide::Bottom => (*px as f32, y1 - 0.2, 0.0, -1.0),
        };
        text(
            painter,
            cam,
            local(comp, x, y),
            name,
            0.7,
            align_dir(comp.rotation, dx, dy),
            fade(colors::TEXT_WEAK, look),
        );
    }
    if side == PinSide::Top && !comp.label.is_empty() {
        // Nahoře jsou vývody – popisek patří pod tělo.
        // [en] The pins are on top – the label goes below the body.
        let (_, _, _, by1) = comp.body();
        text(
            painter,
            cam,
            pos2(comp.pos.x as f32, by1 + 0.4),
            &comp.label,
            0.9,
            Align2::CENTER_TOP,
            fade(colors::TEXT_WEAK, look),
        );
    } else {
        top_label(painter, cam, comp, look);
    }
}

/// Sedmisegmentová číslice. `geom` = (střed x, poloviční šířka, poloviční výška),
/// `segs` = hodnoty segmentů a, b, c, d, e, f, g, dp.
///
/// [en] Seven-segment digit. `geom` = (centre x, half width, half height),
/// `segs` = values of segments a, b, c, d, e, f, g, dp.
fn draw_digit(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    (cx, hw, hh): (f32, f32, f32),
    segs: &[LogicValue],
    look: Look,
) {
    let t = 0.45;
    // mezera mezi segmenty
    // [en] gap between segments
    let g = 0.06;
    let (l, r) = (cx - hw, cx + hw);
    let rects = [
        (l + t + g, -hh, r - t - g, -hh + t),      // a
        (r - t, -hh + t + g, r, -t / 2.0 - g),     // b
        (r - t, t / 2.0 + g, r, hh - t - g),       // c
        (l + t + g, hh - t, r - t - g, hh),        // d
        (l, t / 2.0 + g, l + t, hh - t - g),       // e
        (l, -hh + t + g, l + t, -t / 2.0 - g),     // f
        (l + t + g, -t / 2.0, r - t - g, t / 2.0), // g
    ];
    for (k, (x0, y0, x1, y1)) in rects.into_iter().enumerate() {
        let v = segs.get(k).copied().unwrap_or(LogicValue::Z);
        let pts = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
            .map(|(x, y)| cam.to_screen(local(comp, x, y)))
            .to_vec();
        painter.add(egui::Shape::convex_polygon(
            pts,
            fade(segment_color(v), look),
            Stroke::NONE,
        ));
    }
    let dp = segs.get(7).copied().unwrap_or(LogicValue::Z);
    painter.circle_filled(
        cam.to_screen(local(comp, r + 0.55, hh - 0.25)),
        cam.len(0.28),
        fade(segment_color(dp), look),
    );
}

/// Popisek součástky nad tělem.
///
/// [en] Component label above the body.
fn top_label(painter: &Painter, cam: Camera, comp: &Component, look: Look) {
    if comp.label.is_empty() {
        return;
    }
    let (_, y0, _, _) = comp.body();
    text(
        painter,
        cam,
        pos2(comp.pos.x as f32, y0 - 0.4),
        &comp.label,
        0.9,
        Align2::CENTER_BOTTOM,
        fade(colors::TEXT_WEAK, look),
    );
}

/// Dynamický vstup (trojúhelník u okraje těla) pro hodinový vstup na výšce `y`.
///
/// [en] Dynamic input (triangle at the body edge) for the clock input at height `y`.
fn clock_triangle(painter: &Painter, cam: Camera, comp: &Component, x0: f32, y: f32, ink: Color32) {
    let tri = [
        local(comp, x0, y - 0.45),
        local(comp, x0 + 0.6, y),
        local(comp, x0, y + 0.45),
    ]
    .map(|p| cam.to_screen(p));
    painter.add(egui::Shape::closed_line(
        tri.to_vec(),
        Stroke::new(cam.line_width(), ink),
    ));
}

/// Značka podobvodu: obdélník s jeho názvem a popisky vývodů. Dvojitá horní
/// čára odlišuje podobvod od vestavěných bloků.
///
/// [en] Subcircuit symbol: a rectangle with its name and pin labels. A double top
/// line distinguishes a subcircuit from built-in blocks.
fn draw_subcircuit(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    info: Option<&SubInfo>,
    outline: Stroke,
    look: Look,
) {
    let body = cam.rect(comp.body());
    painter.rect_filled(body, cam.len(0.15), fade(colors::SYMBOL_FILL, look));
    painter.rect_stroke(body, cam.len(0.15), outline, egui::StrokeKind::Middle);
    let (x0, y0, x1, _) = comp.kind.body_local();
    painter.line_segment(
        [
            cam.to_screen(local(comp, x0, y0 + 2.0)),
            cam.to_screen(local(comp, x1, y0 + 2.0)),
        ],
        Stroke::new(1.0, fade(colors::SYMBOL, look)),
    );
    let (ins, outs) = comp.kind.pins_local();
    let ink = fade(colors::TEXT, look);
    let name_of = |names: Option<&Vec<String>>, k: usize| {
        names.and_then(|n| n.get(k)).cloned().unwrap_or_default()
    };
    for (k, (_, py)) in ins.iter().enumerate() {
        text(
            painter,
            cam,
            local(comp, x0 + 0.35, *py as f32),
            &name_of(info.map(|i| &i.inputs), k),
            0.75,
            align_for(comp.rotation, 1.0),
            ink,
        );
    }
    for (k, (_, py)) in outs.iter().enumerate() {
        text(
            painter,
            cam,
            local(comp, x1 - 0.35, *py as f32),
            &name_of(info.map(|i| &i.outputs), k),
            0.75,
            align_for(comp.rotation, -1.0),
            ink,
        );
    }
    let name = info.map_or("?", |i| i.name.as_str());
    let accent = fade(colors::ACCENT, look);
    let lines = name_lines(name, 11);
    let (size, step) = if lines.len() == 1 {
        (0.85, 0.0)
    } else {
        (0.7, 0.75)
    };
    for (k, line) in lines.iter().enumerate() {
        let y = y0 + 1.0 + (k as f32 - (lines.len() as f32 - 1.0) / 2.0) * step;
        text(
            painter,
            cam,
            local(comp, 0.0, y),
            line,
            size,
            Align2::CENTER_CENTER,
            accent,
        );
    }
    top_label(painter, cam, comp, look);
}

/// Rozdělí název do nejvýše dvou řádků po `max` znacích (zalomí u mezery,
/// příliš dlouhá slova zkrátí).
///
/// [en] Splits a name into at most two lines of `max` characters (breaks at a space,
/// shortens words that are too long).
fn name_lines(name: &str, max: usize) -> Vec<String> {
    let cut = |s: &str| -> String {
        if s.chars().count() > max {
            s.chars().take(max - 1).chain(['.']).collect()
        } else {
            s.to_owned()
        }
    };
    if name.chars().count() <= max {
        return vec![name.to_owned()];
    }
    // Mezera nejblíž polovině názvu.
    // [en] The space nearest to the middle of the name.
    let half = name.len() / 2;
    let split = name
        .match_indices(' ')
        .map(|(i, _)| i)
        .min_by_key(|i| i.abs_diff(half));
    match split {
        Some(i) => vec![cut(&name[..i]), cut(name[i + 1..].trim())],
        None => vec![cut(name)],
    }
}

/// Obdélníková značka funkčního bloku s kvalifikátorem a názvy vývodů.
///
/// [en] Rectangular symbol of a function block with a qualifier and pin names.
fn draw_block(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    block: BlockKind,
    outline: Stroke,
    look: Look,
) {
    let body = cam.rect(comp.body());
    painter.rect_filled(body, cam.len(0.15), fade(colors::SYMBOL_FILL, look));
    painter.rect_stroke(body, cam.len(0.15), outline, egui::StrokeKind::Middle);
    let (x0, y0, x1, _) = block.body_local();
    let (ins, outs) = block.pins_local();
    let ink = fade(colors::TEXT, look);
    let size = 0.75;
    let clock = block.clock_input();
    for (i, ((_, py), name)) in ins.iter().zip(block.input_names()).enumerate() {
        let y = *py as f32;
        let mut x = x0 + 0.35;
        let mut name = name;
        if clock == Some(i) {
            clock_triangle(painter, cam, comp, x0, y, ink);
            x = x0 + 0.8;
            if let BlockKind::Counter { down, .. } = block {
                name = if down { "-".into() } else { "+".into() };
            }
        }
        text(
            painter,
            cam,
            local(comp, x, y),
            &name,
            size,
            align_for(comp.rotation, 1.0),
            ink,
        );
    }
    for ((_, py), name) in outs.iter().zip(block.output_names()) {
        text(
            painter,
            cam,
            local(comp, x1 - 0.35, *py as f32),
            &name,
            size,
            align_for(comp.rotation, -1.0),
            ink,
        );
    }
    text(
        painter,
        cam,
        local(comp, 0.0, y0 + 1.1),
        &block.qualifier(),
        0.95,
        Align2::CENTER_CENTER,
        fade(colors::TEXT_WEAK, look),
    );
    top_label(painter, cam, comp, look);
}

/// Popisek vedle značky (přepínač, LED…), posunutý o `dx` od středu.
///
/// [en] Label next to the symbol (switch, LED…), shifted by `dx` from the centre.
fn draw_label(painter: &Painter, cam: Camera, comp: &Component, dx: f32, look: Look) {
    if comp.label.is_empty() {
        return;
    }
    text(
        painter,
        cam,
        local(comp, dx, 0.0),
        &comp.label,
        1.2,
        align_for(comp.rotation, dx),
        fade(colors::TEXT, look),
    );
}

/// Indikátor negace podle ČSN EN 60617-12: pravoúhlý trojúhelník na vývodu.
/// Pravý úhel leží na obrysu těla (`edge`), přepona klesá k vodiči směrem od těla
/// (`dir` = +1 pro výstup vpravo, −1 pro vstup vlevo).
///
/// [en] Negation indicator according to EN 60617-12: a right triangle at the pin.
/// The right angle lies on the body outline (`edge`), the hypotenuse falls
/// towards the wire away from the body (`dir` = +1 for an output on the
/// right, −1 for an input on the left).
fn negation(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    edge: f32,
    y: f32,
    dir: f32,
    stroke: Stroke,
) {
    let tri = [
        local(comp, edge, y - 0.45),
        local(comp, edge, y),
        local(comp, edge + dir * 0.7, y),
    ]
    .map(|p| cam.to_screen(p));
    painter.add(egui::Shape::closed_line(tri.to_vec(), stroke));
}

/// Kroužek negace (americké značky) se středem v `(x, y)`.
///
/// [en] Negation circle (American symbols) centred at `(x, y)`.
fn bubble(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    x: f32,
    y: f32,
    stroke: Stroke,
    look: Look,
) {
    let c = cam.to_screen(local(comp, x, y));
    painter.circle_filled(c, cam.len(0.35), fade(colors::SYMBOL_FILL, look));
    painter.circle_stroke(c, cam.len(0.35), stroke);
}

/// Body kvadratické Bézierovy křivky (bez počátečního bodu).
///
/// [en] Points of a quadratic Bézier curve (without the start point).
fn bezier(a: (f32, f32), c: (f32, f32), b: (f32, f32), n: usize) -> Vec<(f32, f32)> {
    (1..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            (
                u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0,
                u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1,
            )
        })
        .collect()
}

/// Vyplněný obrys (hvězdicovitý vzhledem k bodu `center`) s okrajem.
///
/// [en] Filled outline (star-shaped with respect to point `center`) with a border.
fn filled_outline(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    center: (f32, f32),
    outline: &[(f32, f32)],
    fill: Color32,
    stroke: Stroke,
) {
    let pts: Vec<Pos2> = outline
        .iter()
        .map(|&(x, y)| cam.to_screen(local(comp, x, y)))
        .collect();
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(cam.to_screen(local(comp, center.0, center.1)), fill);
    for p in &pts {
        mesh.colored_vertex(*p, fill);
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    painter.add(egui::Shape::mesh(mesh));
    painter.add(egui::Shape::closed_line(pts, stroke));
}

/// Americká tvarová značka hradla (ANSI/IEEE Std 91).
///
/// [en] American distinctive-shape gate symbol (ANSI/IEEE Std 91).
fn draw_ansi_gate(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    gate: logicflow_core::GateKind,
    outline: Stroke,
    look: Look,
) {
    use logicflow_core::GateKind as G;
    let (x0, y0, x1, y1) = comp.kind.body_local();
    let fill = fade(colors::SYMBOL_FILL, look);
    let inverted = gate.is_inverted();
    // U negovaných hradel zůstane vpravo místo na kroužek.
    // [en] Negated gates keep room for the circle on the right.
    let right = if inverted { x1 - 0.7 } else { x1 };
    let h = if gate.is_unary() {
        1.3
    } else {
        y1.min(-y0) - 0.2
    };
    let (ins, _) = comp.kind.pins_local();
    let lead = Stroke::new(cam.line_width(), fade(colors::SYMBOL, look));
    // Zadní oblouk OR/XOR: x podle y.
    // [en] Back arc of OR/XOR: x as a function of y.
    let depth = 0.55;
    let back = |x: f32, y: f32| x + depth * (1.0 - (y / h).powi(2)).max(0.0);
    match gate {
        G::Not | G::Buffer => {
            let tri = [(x0, -h), (right, 0.0), (x0, h)];
            filled_outline(painter, cam, comp, (x0 + 0.5, 0.0), &tri, fill, outline);
        }
        G::And | G::Nand => {
            let xc = x0 + (right - x0) * 0.35;
            let (rx, ry) = (right - xc, h);
            let mut pts = vec![(x0, -h), (xc, -h)];
            pts.extend((1..24).map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 24.0;
                (xc + rx * a.cos(), ry * a.sin())
            }));
            pts.extend([(xc, h), (x0, h)]);
            filled_outline(painter, cam, comp, (x0 + 0.5, 0.0), &pts, fill, outline);
        }
        G::Or | G::Nor | G::Xor | G::Xnor => {
            let exclusive = matches!(gate, G::Xor | G::Xnor);
            let bx = if exclusive { x0 + 0.45 } else { x0 };
            let w = right - bx;
            let mut pts = vec![(bx, -h)];
            pts.extend(bezier((bx, -h), (bx + w * 0.6, -h), (right, 0.0), 16));
            pts.extend(bezier((right, 0.0), (bx + w * 0.6, h), (bx, h), 16));
            // zadní oblouk zdola nahoru
            // [en] back arc from bottom to top
            pts.extend((1..16).map(|i| {
                let y = h - 2.0 * h * i as f32 / 16.0;
                (back(bx, y), y)
            }));
            filled_outline(
                painter,
                cam,
                comp,
                (bx + depth + 0.4, 0.0),
                &pts,
                fill,
                outline,
            );
            if exclusive {
                let arc: Vec<Pos2> = (0..=16)
                    .map(|i| {
                        let y = -h + 2.0 * h * i as f32 / 16.0;
                        cam.to_screen(local(comp, back(x0, y), y))
                    })
                    .collect();
                painter.add(egui::Shape::line(arc, outline));
            }
            // Vstupy prodloužené až k zadnímu oblouku.
            // [en] Inputs extended up to the back arc.
            for (_, py) in &ins {
                let y = *py as f32;
                let a = cam.to_screen(local(comp, x0, y));
                let b = cam.to_screen(local(comp, back(x0, y), y));
                painter.line_segment([a, b], lead);
            }
        }
    }
    if inverted {
        bubble(painter, cam, comp, x1 - 0.35, 0.0, outline, look);
    }
    top_label(painter, cam, comp, look);
}

/// Frekvence hodin pro popisek („0,5 Hz“).
///
/// [en] Clock frequency for a label (“0.5 Hz”).
pub fn format_hz(hz: f32) -> String {
    if (hz - hz.round()).abs() < 0.01 {
        format!("{} Hz", hz.round())
    } else {
        format!("{hz:.1} Hz").replace('.', ",")
    }
}

/// Obdélníková značka klopného obvodu s Q a Q̄ a dynamickým vstupem.
///
/// [en] Rectangular flip-flop symbol with Q and Q̄ and a dynamic input.
fn draw_flip_flop(
    painter: &Painter,
    cam: Camera,
    comp: &Component,
    ff: FlipFlopKind,
    falling: bool,
    outline: Stroke,
    look: Look,
) {
    let body = cam.rect(comp.body());
    painter.rect_filled(body, cam.len(0.15), fade(colors::SYMBOL_FILL, look));
    painter.rect_stroke(body, cam.len(0.15), outline, egui::StrokeKind::Middle);
    let (x0, _, x1, _) = comp.kind.body_local();
    let (ins, outs) = comp.kind.pins_local();
    let ink = fade(colors::TEXT, look);
    let size = 0.85;

    for (i, ((_, py), name)) in ins.iter().zip(ff.input_names()).enumerate() {
        let y = *py as f32;
        let mut x = x0 + 0.4;
        if ff.is_edge_triggered() && ff.clock_input() == Some(i) {
            // Dynamický vstup: trojúhelník u okraje těla.
            // [en] Dynamic input: a triangle at the body edge.
            let tri = [
                local(comp, x0, y - 0.45),
                local(comp, x0 + 0.6, y),
                local(comp, x0, y + 0.45),
            ]
            .map(|p| cam.to_screen(p));
            painter.add(egui::Shape::closed_line(
                tri.to_vec(),
                Stroke::new(cam.line_width(), ink),
            ));
            if falling {
                if style(painter) == SymbolStyle::Ansi {
                    bubble(painter, cam, comp, x0 - 0.35, y, outline, look);
                } else {
                    negation(painter, cam, comp, x0, y, -1.0, outline);
                }
            }
            x = x0 + 0.85;
        }
        text(
            painter,
            cam,
            local(comp, x, y),
            name,
            size,
            align_for(comp.rotation, 1.0),
            ink,
        );
    }
    for (k, (_, py)) in outs.iter().enumerate() {
        let p = local(comp, x1 - 0.4, *py as f32);
        text(
            painter,
            cam,
            p,
            "Q",
            size,
            align_for(comp.rotation, -1.0),
            ink,
        );
        if k == 1 && cam.len(size) >= 5.0 {
            // Negovaný výstup: čára nad Q.
            // [en] Negated output: a bar over Q.
            let (lx0, lx1, ly) = (x1 - 1.05, x1 - 0.45, *py as f32 - 0.55);
            painter.line_segment(
                [
                    cam.to_screen(local(comp, lx0, ly)),
                    cam.to_screen(local(comp, lx1, ly)),
                ],
                Stroke::new(1.0, ink),
            );
        }
    }
    let short = match ff {
        FlipFlopKind::DLatch => "D",
        other => other.name(),
    };
    // U JK je uprostřed hodinový vstup, název typu proto o řádek výš.
    // [en] JK has the clock input in the middle, so the type name goes one line higher.
    let center = if ff == FlipFlopKind::Jk {
        local(comp, 0.3, -1.0)
    } else {
        local(comp, 0.0, 0.0)
    };
    text(
        painter,
        cam,
        center,
        short,
        1.1,
        Align2::CENTER_CENTER,
        fade(colors::TEXT_WEAK, look),
    );
    if !comp.label.is_empty() {
        let (_, y0, _, _) = comp.body();
        text(
            painter,
            cam,
            pos2(comp.pos.x as f32, y0 - 0.4),
            &comp.label,
            0.9,
            Align2::CENTER_BOTTOM,
            fade(colors::TEXT_WEAK, look),
        );
    }
}

/// Obdélník, na který jde kliknout (tělo s malou rezervou), ve světových souřadnicích.
///
/// [en] Clickable rectangle (the body with a small margin) in world coordinates.
pub fn hit_rect(comp: &Component) -> Rect {
    let (x0, y0, x1, y1) = comp.body();
    Rect::from_min_max(pos2(x0, y0), pos2(x1, y1)).expand(0.25)
}

/// Zaokrouhlí světový bod na nejbližší bod mřížky.
///
/// [en] Rounds a world point to the nearest grid point.
pub fn snap(p: Pos2) -> logicflow_core::Point {
    logicflow_core::Point::new(p.x.round() as i32, p.y.round() as i32)
}

/// Bod mřížky jako světový bod.
///
/// [en] Grid point as a world point.
pub fn point(p: logicflow_core::Point) -> Pos2 {
    pos2(p.x as f32, p.y as f32)
}

/// Vzdálenost bodu od úsečky.
///
/// [en] Distance of a point from a segment.
pub fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab: Vec2 = b - a;
    let t = if ab.length_sq() < 1e-6 {
        0.0
    } else {
        ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
    };
    (a + ab * t).distance(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_roundtrip() {
        let cam = Camera {
            origin: pos2(100.0, 50.0),
            scale: 24.0,
        };
        let p = pos2(-3.5, 7.25);
        assert!((cam.to_world(cam.to_screen(p)) - p).length() < 1e-4);
    }

    #[test]
    fn segment_distance() {
        assert!(
            (dist_to_segment(pos2(5.0, 1.0), pos2(0.0, 0.0), pos2(10.0, 0.0)) - 1.0).abs() < 1e-6
        );
        assert!(
            (dist_to_segment(pos2(-3.0, 4.0), pos2(0.0, 0.0), pos2(10.0, 0.0)) - 5.0).abs() < 1e-6
        );
    }

    #[test]
    fn long_names_wrap() {
        assert_eq!(name_lines("NAND", 11), ["NAND"]);
        assert_eq!(
            name_lines("Poloviční sčítačka", 11),
            ["Poloviční", "sčítačka"]
        );
        assert_eq!(name_lines("Dekodérosmisegment", 11), ["Dekodérosm."]);
    }

    #[test]
    fn hz_format() {
        assert_eq!(format_hz(1.0), "1 Hz");
        assert_eq!(format_hz(0.5), "0,5 Hz");
    }

    #[test]
    fn label_alignment_follows_rotation() {
        assert_eq!(align_for(Rotation::R0, -1.0), Align2::RIGHT_CENTER);
        assert_eq!(align_for(Rotation::R180, -1.0), Align2::LEFT_CENTER);
        assert_eq!(align_for(Rotation::R90, -1.0), Align2::CENTER_BOTTOM);
    }
}
