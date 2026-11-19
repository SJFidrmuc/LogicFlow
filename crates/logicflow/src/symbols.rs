//! Kreslení značek součástek (ČSN EN 60617, obdélníkové značky).

use crate::theme::{colors, signal_color};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2, pos2, vec2};
use logicflow_core::{Component, ComponentKind, FlipFlopKind, LogicValue, Rotation};

/// Velikost jedné jednotky mřížky v pixelech při zoomu 100 %.
pub const UNIT: f32 = 16.0;

/// Převod mezi světovými (mřížka) a obrazovkovými souřadnicemi.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub origin: Pos2,
    pub scale: f32,
}

impl Camera {
    pub fn to_screen(self, p: Pos2) -> Pos2 {
        self.origin + p.to_vec2() * self.scale
    }

    pub fn to_world(self, p: Pos2) -> Pos2 {
        ((p - self.origin) / self.scale).to_pos2()
    }

    pub fn len(self, l: f32) -> f32 {
        l * self.scale
    }

    pub fn rect(self, (x0, y0, x1, y1): (f32, f32, f32, f32)) -> Rect {
        Rect::from_min_max(self.to_screen(pos2(x0, y0)), self.to_screen(pos2(x1, y1)))
    }

    pub fn line_width(self) -> f32 {
        self.len(0.12).clamp(1.0, 3.0)
    }
}

/// Stav, ve kterém se součástka kreslí.
#[derive(Clone, Copy, Default)]
pub struct Look {
    pub selected: bool,
    pub hovered: bool,
    /// Poloprůhledný náhled při umisťování.
    pub ghost: bool,
}

/// Hodnoty potřebné ke kreslení (ze simulátoru).
pub struct Values {
    pub inputs: Vec<LogicValue>,
    pub outputs: Vec<LogicValue>,
}

pub const SELECTED: Color32 = Color32::from_rgb(0xff, 0xa0, 0x30);

fn fade(c: Color32, look: Look) -> Color32 {
    if look.ghost {
        c.gamma_multiply(0.45)
    } else {
        c
    }
}

fn local(comp: &Component, x: f32, y: f32) -> Pos2 {
    let (rx, ry) = comp.rotation.apply_f(x, y);
    pos2(comp.pos.x as f32 + rx, comp.pos.y as f32 + ry)
}

/// Zarovnání popisku podle směru, kterým od součástky míří.
fn align_for(rot: Rotation, dx: f32) -> Align2 {
    let (x, y) = rot.apply_f(dx, 0.0);
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

pub fn draw(painter: &Painter, cam: Camera, comp: &Component, values: &Values, look: Look) {
    let lw = cam.line_width();
    let outline = if look.selected {
        Stroke::new(lw + 1.0, SELECTED)
    } else if look.hovered {
        Stroke::new(lw, Color32::WHITE)
    } else {
        Stroke::new(lw, fade(colors::SYMBOL, look))
    };
    let (x0, _y0, x1, _y1) = comp.kind.body_local();
    let (ins, outs) = comp.kind.pins_local();
    let inverted = matches!(comp.kind, ComponentKind::Gate { gate, .. } if gate.is_inverted());
    // Klopný obvod spouštěný sestupnou hranou má na hodinovém vstupu kroužek.
    let clock_bubble = match comp.kind {
        ComponentKind::FlipFlop {
            ff, falling: true, ..
        } if ff.is_edge_triggered() => ff.clock_input(),
        _ => None,
    };
    let lead = Stroke::new(lw, fade(colors::SYMBOL, look));

    // Vývody: krátká čára od těla k bodu mřížky a tečka obarvená hodnotou.
    let pin_color = |v: LogicValue| fade(signal_color(v), look);
    for (i, (px, py)) in ins.iter().enumerate() {
        let v = values.inputs.get(i).copied().unwrap_or(LogicValue::Z);
        let start = if clock_bubble == Some(i) {
            x0 - 0.7
        } else {
            x0
        };
        let a = local(comp, start, *py as f32);
        let b = local(comp, *px as f32, *py as f32);
        painter.line_segment([cam.to_screen(a), cam.to_screen(b)], lead);
        painter.circle_filled(cam.to_screen(b), cam.len(0.16).max(2.0), pin_color(v));
    }
    for (k, (px, py)) in outs.iter().enumerate() {
        let v = values.outputs.get(k).copied().unwrap_or(LogicValue::Z);
        let start = if inverted { x1 + 0.7 } else { x1 };
        let a = local(comp, start, *py as f32);
        let b = local(comp, *px as f32, *py as f32);
        painter.line_segment([cam.to_screen(a), cam.to_screen(b)], lead);
        painter.circle_filled(cam.to_screen(b), cam.len(0.16).max(2.0), pin_color(v));
    }

    let body = cam.rect(comp.body());
    let fill = fade(colors::SYMBOL_FILL, look);
    match comp.kind {
        ComponentKind::Gate { gate, inputs } => {
            painter.rect_filled(body, cam.len(0.15), fill);
            painter.rect_stroke(body, cam.len(0.15), outline, egui::StrokeKind::Middle);
            if inverted {
                painter.circle_stroke(
                    cam.to_screen(local(comp, x1 + 0.35, 0.0)),
                    cam.len(0.35),
                    outline,
                );
            }
            // Značka v horní části těla (u otočených hradel uprostřed).
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
    }
}

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

/// Frekvence hodin pro popisek („0,5 Hz“).
pub fn format_hz(hz: f32) -> String {
    if (hz - hz.round()).abs() < 0.01 {
        format!("{} Hz", hz.round())
    } else {
        format!("{hz:.1} Hz").replace('.', ",")
    }
}

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
                painter.circle_stroke(
                    cam.to_screen(local(comp, x0 - 0.35, y)),
                    cam.len(0.35),
                    outline,
                );
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
pub fn hit_rect(comp: &Component) -> Rect {
    let (x0, y0, x1, y1) = comp.body();
    Rect::from_min_max(pos2(x0, y0), pos2(x1, y1)).expand(0.25)
}

pub fn snap(p: Pos2) -> logicflow_core::Point {
    logicflow_core::Point::new(p.x.round() as i32, p.y.round() as i32)
}

pub fn point(p: logicflow_core::Point) -> Pos2 {
    pos2(p.x as f32, p.y as f32)
}

/// Vzdálenost bodu od úsečky.
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
