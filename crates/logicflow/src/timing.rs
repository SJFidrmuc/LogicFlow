//! Časový diagram (logický analyzátor).

use crate::demo::{DemoCircuit, Sample};
use crate::i18n::{Lang, T, tr};
use crate::theme::{colors, signal_color};
use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2};
use logicflow_core::LogicValue;

/// Kolik sekund historie je vidět.
const WINDOW: f64 = 20.0;
const NAME_W: f32 = 70.0;
const ROW_H: f32 = 34.0;

type Getter = fn(&Sample) -> LogicValue;

const SIGNALS: [(&str, Getter); 4] = [
    ("A", |s| s.a),
    ("B", |s| s.b),
    ("Y1", |s| s.y1),
    ("Y2", |s| s.y2),
];

pub fn ui(ui: &mut Ui, circuit: &DemoCircuit, lang: Lang) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, colors::PANEL);

    let now = circuit.time.max(WINDOW);
    let t0 = now - WINDOW;
    let plot = Rect::from_min_max(
        pos2(rect.left() + NAME_W, rect.top() + 8.0),
        pos2(rect.right() - 12.0, rect.bottom() - 24.0),
    );
    if plot.width() < 20.0 || plot.height() < 20.0 {
        return;
    }
    let x_of = |t: f64| plot.left() + ((t - t0) / WINDOW) as f32 * plot.width();

    // Svislé čáry po sekundách a popisky po pěti.
    let first = t0.ceil() as i64;
    for s in first..=now.floor() as i64 {
        let x = x_of(s as f64);
        let major = s % 5 == 0;
        let c = if major {
            colors::GRID_MAJOR
        } else {
            colors::GRID_MINOR
        };
        painter.line_segment(
            [pos2(x, plot.top()), pos2(x, plot.bottom())],
            Stroke::new(1.0, c),
        );
        if major {
            painter.text(
                pos2(x, plot.bottom() + 4.0),
                Align2::CENTER_TOP,
                format!("{s}"),
                FontId::proportional(11.0),
                colors::TEXT_WEAK,
            );
        }
    }
    painter.text(
        pos2(rect.left() + 12.0, rect.bottom() - 4.0),
        Align2::LEFT_BOTTOM,
        tr(lang, T::TimeAxis),
        FontId::proportional(11.0),
        colors::TEXT_WEAK,
    );

    for (row, (name, get)) in SIGNALS.iter().enumerate() {
        let top = plot.top() + row as f32 * ROW_H;
        if top + ROW_H > plot.bottom() + 1.0 {
            break;
        }
        let hi = top + 8.0;
        let lo = top + ROW_H - 8.0;
        painter.text(
            pos2(rect.left() + 12.0, (hi + lo) / 2.0),
            Align2::LEFT_CENTER,
            *name,
            FontId::monospace(13.0),
            colors::TEXT,
        );

        // Úseky mezi změnami: (od, do, hodnota).
        let mut segments: Vec<(f64, f64, LogicValue)> = Vec::new();
        for (i, s) in circuit.history.iter().enumerate() {
            let end = circuit.history.get(i + 1).map_or(circuit.time, |n| n.t);
            if end < t0 {
                continue;
            }
            segments.push((s.t.max(t0), end, get(s)));
        }

        let mut prev_y: Option<f32> = None;
        for (from, to, v) in segments {
            let y = match v {
                LogicValue::One => hi,
                LogicValue::Zero => lo,
                _ => (hi + lo) / 2.0,
            };
            let (x0, x1) = (x_of(from), x_of(to));
            let stroke = Stroke::new(2.0, signal_color(v));
            if let Some(py) = prev_y
                && (py - y).abs() > 0.5
            {
                painter.line_segment(
                    [pos2(x0, py), pos2(x0, y)],
                    Stroke::new(1.5, colors::TEXT_WEAK),
                );
            }
            if v == LogicValue::One {
                // Jemná výplň pod úrovní 1 zlepšuje čitelnost.
                painter.rect_filled(
                    Rect::from_min_max(pos2(x0, hi), pos2(x1, lo)),
                    0.0,
                    signal_color(v).gamma_multiply(0.12),
                );
            }
            painter.add(Shape::line(
                vec![Pos2::new(x0, y), Pos2::new(x1, y)],
                stroke,
            ));
            prev_y = Some(y);
        }
        painter.line_segment(
            [
                pos2(plot.left(), top + ROW_H),
                pos2(plot.right(), top + ROW_H),
            ],
            Stroke::new(1.0, colors::GAP),
        );
    }

    // Kurzor „teď“.
    let x = x_of(circuit.time.max(t0));
    painter.line_segment(
        [pos2(x, plot.top()), pos2(x, plot.bottom())],
        Stroke::new(1.0, colors::ACCENT),
    );
}
