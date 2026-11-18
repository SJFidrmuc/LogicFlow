//! Editor schématu: nekonečné plátno s mřížkou, zoomem a posunem.
//!
//! V milníku M0 zobrazuje ukázkový obvod; kreslení vlastních obvodů přijde ve verzi 0.1.

use crate::demo::{DemoCircuit, Item};
use crate::i18n::{Lang, T, tr};
use crate::theme::{colors, signal_color};
use egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2, pos2, vec2,
};
use logicflow_core::{GateKind, LogicValue};

/// Velikost jedné jednotky mřížky v pixelech při zoomu 100 %.
const UNIT: f32 = 16.0;
const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 6.0;

/// Ohraničení ukázkového obvodu ve světových souřadnicích.
const DEMO_BOUNDS: Rect = Rect {
    min: pos2(-16.0, -8.0),
    max: pos2(18.0, 8.0),
};

/// Kamera jednoho editoru schématu (každá oblast má vlastní).
#[derive(Clone, Debug)]
pub struct SchematicView {
    pub offset: Vec2,
    pub zoom: f32,
    pub needs_fit: bool,
}

impl Default for SchematicView {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
            needs_fit: true,
        }
    }
}

/// Převod mezi světovými a obrazovkovými souřadnicemi.
#[derive(Clone, Copy)]
struct Camera {
    origin: Pos2,
    scale: f32,
}

impl Camera {
    fn to_screen(self, p: Pos2) -> Pos2 {
        self.origin + p.to_vec2() * self.scale
    }

    fn to_world(self, p: Pos2) -> Pos2 {
        ((p - self.origin) / self.scale).to_pos2()
    }

    fn len(self, l: f32) -> f32 {
        l * self.scale
    }
}

impl SchematicView {
    fn camera(&self, rect: Rect) -> Camera {
        Camera {
            origin: rect.center() + self.offset,
            scale: UNIT * self.zoom,
        }
    }

    fn fit(&mut self, rect: Rect) {
        let size = DEMO_BOUNDS.size() * UNIT;
        self.zoom = (rect.width() / size.x)
            .min(rect.height() / size.y)
            .clamp(MIN_ZOOM, MAX_ZOOM);
        self.offset = -DEMO_BOUNDS.center().to_vec2() * UNIT * self.zoom;
        self.needs_fit = false;
    }

    pub fn ui(&mut self, ui: &mut Ui, circuit: &mut DemoCircuit, lang: Lang) {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        if self.needs_fit && rect.width() > 10.0 {
            self.fit(rect);
        }

        // Posun prostředním nebo pravým tlačítkem myši.
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            self.offset += response.drag_delta();
        }

        if response.hovered() {
            let (scroll, pinch, home) = ui.input(|i| {
                (
                    i.smooth_scroll_delta.y,
                    i.zoom_delta(),
                    i.key_pressed(egui::Key::Home),
                )
            });
            let factor = (scroll * 0.0025).exp() * pinch;
            if (factor - 1.0).abs() > f32::EPSILON
                && let Some(mouse) = response.hover_pos()
            {
                // Zoom kolem kurzoru: bod pod myší zůstane na místě.
                let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
                let world = self.camera(rect).to_world(mouse);
                self.zoom = new_zoom;
                let cam = self.camera(rect);
                self.offset += mouse - cam.to_screen(world);
            }
            if home {
                self.fit(rect);
            }
        }

        let cam = self.camera(rect);

        // Výběr a přepínání kliknutím.
        if response.clicked()
            && let Some(p) = response.interact_pointer_pos()
        {
            let hit = hit_test(cam.to_world(p));
            circuit.selected = hit;
            if let Some(item) = hit {
                circuit.toggle(item);
            }
        }

        painter.rect_filled(rect, 0.0, colors::CANVAS);
        draw_grid(&painter, rect, cam);
        draw_demo(&painter, cam, circuit);

        // Nápověda v dolní části plátna.
        painter.text(
            rect.left_bottom() + vec2(10.0, -10.0),
            Align2::LEFT_BOTTOM,
            tr(lang, T::SchematicHint),
            FontId::proportional(12.0),
            colors::TEXT_WEAK,
        );
        painter.text(
            rect.left_top() + vec2(10.0, 10.0),
            Align2::LEFT_TOP,
            tr(lang, T::DemoNote),
            FontId::proportional(12.0),
            colors::TEXT_WEAK,
        );
    }
}

fn draw_grid(painter: &Painter, rect: Rect, cam: Camera) {
    let min = cam.to_world(rect.min);
    let max = cam.to_world(rect.max);
    let minor_visible = cam.scale >= 7.0;
    let (x0, x1) = (min.x.floor() as i32, max.x.ceil() as i32);
    let (y0, y1) = (min.y.floor() as i32, max.y.ceil() as i32);
    for x in x0..=x1 {
        let major = x % 5 == 0;
        if !major && !minor_visible {
            continue;
        }
        let sx = cam.to_screen(pos2(x as f32, 0.0)).x;
        let c = if major {
            colors::GRID_MAJOR
        } else {
            colors::GRID_MINOR
        };
        painter.line_segment(
            [pos2(sx, rect.top()), pos2(sx, rect.bottom())],
            Stroke::new(1.0, c),
        );
    }
    for y in y0..=y1 {
        let major = y % 5 == 0;
        if !major && !minor_visible {
            continue;
        }
        let sy = cam.to_screen(pos2(0.0, y as f32)).y;
        let c = if major {
            colors::GRID_MAJOR
        } else {
            colors::GRID_MINOR
        };
        painter.line_segment(
            [pos2(rect.left(), sy), pos2(rect.right(), sy)],
            Stroke::new(1.0, c),
        );
    }
}

// --- Rozmístění ukázkového obvodu (světové souřadnice) ---

const SW_X: f32 = -12.0;
const A_Y: f32 = -5.0;
const B_Y: f32 = 5.0;
const GATE_X: f32 = 0.0;
const AND_Y: f32 = -4.0;
const OR_Y: f32 = 4.0;
const LED_X: f32 = 9.0;

fn switch_rect(y: f32) -> Rect {
    Rect::from_center_size(pos2(SW_X, y), vec2(2.4, 1.6))
}

fn gate_rect(y: f32) -> Rect {
    Rect::from_center_size(pos2(GATE_X, y), vec2(4.0, 5.0))
}

const LED_R: f32 = 0.9;

fn hit_test(p: Pos2) -> Option<Item> {
    let led_hit = |y: f32| pos2(LED_X, y).distance(p) <= LED_R + 0.3;
    if switch_rect(A_Y).expand(0.3).contains(p) {
        Some(Item::SwitchA)
    } else if switch_rect(B_Y).expand(0.3).contains(p) {
        Some(Item::SwitchB)
    } else if gate_rect(AND_Y).contains(p) {
        Some(Item::GateAnd)
    } else if gate_rect(OR_Y).contains(p) {
        Some(Item::GateOr)
    } else if led_hit(AND_Y) {
        Some(Item::LedY1)
    } else if led_hit(OR_Y) {
        Some(Item::LedY2)
    } else {
        None
    }
}

fn wire(painter: &Painter, cam: Camera, points: &[Pos2], v: LogicValue) {
    let pts: Vec<Pos2> = points.iter().map(|&p| cam.to_screen(p)).collect();
    let width = cam.len(0.16).clamp(1.5, 4.0);
    painter.add(Shape::line(pts, Stroke::new(width, signal_color(v))));
}

fn junction(painter: &Painter, cam: Camera, p: Pos2, v: LogicValue) {
    painter.circle_filled(cam.to_screen(p), cam.len(0.3).max(3.0), signal_color(v));
}

fn label(
    painter: &Painter,
    cam: Camera,
    p: Pos2,
    text: &str,
    size: f32,
    align: Align2,
    color: Color32,
) {
    painter.text(
        cam.to_screen(p),
        align,
        text,
        FontId::proportional(cam.len(size).max(8.0)),
        color,
    );
}

fn draw_demo(painter: &Painter, cam: Camera, c: &DemoCircuit) {
    let (a, b, y1, y2) = (c.va(), c.vb(), c.y1(), c.y2());
    let out_x = SW_X + 1.2;
    let in_x = GATE_X - 3.0;

    // Vodiče ze vstupů do hradel. A se větví u x = -8, B u x = -6.
    wire(painter, cam, &[pos2(out_x, A_Y), pos2(in_x, A_Y)], a);
    wire(
        painter,
        cam,
        &[
            pos2(-8.0, A_Y),
            pos2(-8.0, OR_Y - 1.0),
            pos2(in_x, OR_Y - 1.0),
        ],
        a,
    );
    junction(painter, cam, pos2(-8.0, A_Y), a);
    wire(painter, cam, &[pos2(out_x, B_Y), pos2(in_x, B_Y)], b);
    wire(
        painter,
        cam,
        &[
            pos2(-6.0, B_Y),
            pos2(-6.0, AND_Y + 1.0),
            pos2(in_x, AND_Y + 1.0),
        ],
        b,
    );
    junction(painter, cam, pos2(-6.0, B_Y), b);

    // Výstupy do LED.
    wire(
        painter,
        cam,
        &[pos2(GATE_X + 3.0, AND_Y), pos2(LED_X - LED_R, AND_Y)],
        y1,
    );
    wire(
        painter,
        cam,
        &[pos2(GATE_X + 3.0, OR_Y), pos2(LED_X - LED_R, OR_Y)],
        y2,
    );

    draw_switch(
        painter,
        cam,
        A_Y,
        "A",
        c.a,
        c.selected == Some(Item::SwitchA),
    );
    draw_switch(
        painter,
        cam,
        B_Y,
        "B",
        c.b,
        c.selected == Some(Item::SwitchB),
    );
    draw_gate(
        painter,
        cam,
        AND_Y,
        GateKind::And,
        c.selected == Some(Item::GateAnd),
    );
    draw_gate(
        painter,
        cam,
        OR_Y,
        GateKind::Or,
        c.selected == Some(Item::GateOr),
    );
    draw_led(
        painter,
        cam,
        AND_Y,
        "Y1 = A · B",
        y1,
        c.selected == Some(Item::LedY1),
    );
    draw_led(
        painter,
        cam,
        OR_Y,
        "Y2 = A + B",
        y2,
        c.selected == Some(Item::LedY2),
    );
}

fn outline(selected: bool, cam: Camera) -> Stroke {
    let w = cam.len(0.12).clamp(1.0, 3.0);
    if selected {
        Stroke::new(w + 1.0, Color32::from_rgb(0xff, 0xa0, 0x30))
    } else {
        Stroke::new(w, colors::SYMBOL)
    }
}

fn draw_switch(painter: &Painter, cam: Camera, y: f32, name: &str, on: bool, selected: bool) {
    let r = switch_rect(y);
    let sr = Rect::from_min_max(cam.to_screen(r.min), cam.to_screen(r.max));
    painter.rect_filled(sr, cam.len(0.3), colors::SYMBOL_FILL);
    painter.rect_stroke(
        sr,
        cam.len(0.3),
        outline(selected, cam),
        egui::StrokeKind::Middle,
    );
    // Jezdec přepínače: vpravo = 1, vlevo = 0.
    let knob_w = r.width() / 2.0 - 0.2;
    let knob_x = if on {
        r.max.x - 0.1 - knob_w / 2.0
    } else {
        r.min.x + 0.1 + knob_w / 2.0
    };
    let knob = Rect::from_center_size(pos2(knob_x, y), vec2(knob_w, r.height() - 0.3));
    let fill = signal_color(on.into());
    painter.rect_filled(
        Rect::from_min_max(cam.to_screen(knob.min), cam.to_screen(knob.max)),
        cam.len(0.2),
        fill,
    );
    label(
        painter,
        cam,
        pos2(knob_x, y),
        if on { "1" } else { "0" },
        0.9,
        Align2::CENTER_CENTER,
        Color32::BLACK,
    );
    label(
        painter,
        cam,
        pos2(r.min.x - 0.6, y),
        name,
        1.2,
        Align2::RIGHT_CENTER,
        colors::TEXT,
    );
}

fn draw_gate(painter: &Painter, cam: Camera, y: f32, kind: GateKind, selected: bool) {
    let r = gate_rect(y);
    // Vývody.
    let pin = Stroke::new(cam.len(0.12).clamp(1.0, 3.0), colors::SYMBOL);
    for dy in [-1.0, 1.0] {
        painter.line_segment(
            [
                cam.to_screen(pos2(r.min.x - 1.0, y + dy)),
                cam.to_screen(pos2(r.min.x, y + dy)),
            ],
            pin,
        );
    }
    let out_start = if kind.is_inverted() {
        r.max.x + 0.7
    } else {
        r.max.x
    };
    painter.line_segment(
        [
            cam.to_screen(pos2(out_start, y)),
            cam.to_screen(pos2(r.max.x + 1.0, y)),
        ],
        pin,
    );

    let sr = Rect::from_min_max(cam.to_screen(r.min), cam.to_screen(r.max));
    painter.rect_filled(sr, cam.len(0.15), colors::SYMBOL_FILL);
    painter.rect_stroke(
        sr,
        cam.len(0.15),
        outline(selected, cam),
        egui::StrokeKind::Middle,
    );
    if kind.is_inverted() {
        painter.circle_stroke(
            cam.to_screen(pos2(r.max.x + 0.35, y)),
            cam.len(0.35),
            outline(selected, cam),
        );
    }
    label(
        painter,
        cam,
        pos2(r.center().x, r.min.y + 1.2),
        kind.iec_label(),
        1.4,
        Align2::CENTER_CENTER,
        colors::TEXT,
    );
}

fn draw_led(painter: &Painter, cam: Camera, y: f32, text: &str, v: LogicValue, selected: bool) {
    let c = cam.to_screen(pos2(LED_X, y));
    let lit = v == LogicValue::One;
    if lit {
        // Jemná záře rozsvícené LED.
        painter.circle_filled(
            c,
            cam.len(LED_R * 1.6),
            Color32::from_rgba_unmultiplied(255, 60, 40, 40),
        );
    }
    let fill = if lit {
        Color32::from_rgb(0xff, 0x45, 0x30)
    } else {
        Color32::from_rgb(0x55, 0x22, 0x1e)
    };
    painter.circle_filled(c, cam.len(LED_R), fill);
    painter.circle_stroke(c, cam.len(LED_R), outline(selected, cam));
    label(
        painter,
        cam,
        pos2(LED_X + LED_R + 0.6, y),
        text,
        1.0,
        Align2::LEFT_CENTER,
        colors::TEXT,
    );
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
        let back = cam.to_world(cam.to_screen(p));
        assert!((back - p).length() < 1e-4);
    }

    #[test]
    fn hit_switches_and_empty_space() {
        assert_eq!(hit_test(pos2(SW_X, A_Y)), Some(Item::SwitchA));
        assert_eq!(hit_test(pos2(SW_X, B_Y)), Some(Item::SwitchB));
        assert_eq!(hit_test(pos2(GATE_X, OR_Y)), Some(Item::GateOr));
        assert_eq!(hit_test(pos2(LED_X, AND_Y)), Some(Item::LedY1));
        assert_eq!(hit_test(pos2(-14.0, 0.0)), None);
    }

    #[test]
    fn demo_fits_inside_bounds() {
        for r in [
            switch_rect(A_Y),
            switch_rect(B_Y),
            gate_rect(AND_Y),
            gate_rect(OR_Y),
        ] {
            assert!(DEMO_BOUNDS.contains_rect(r));
        }
    }
}
