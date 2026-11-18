//! Časový diagram (logický analyzátor) vstupů a výstupů obvodu.

use crate::i18n::{Lang, T, tr};
use crate::project::Project;
use crate::theme::{colors, signal_color};
use egui::{Align2, FontId, Rect, Sense, Stroke, Ui, pos2};
use logicflow_core::LogicValue;

/// Kolik sekund historie je vidět.
const WINDOW: f64 = 20.0;
const NAME_W: f32 = 70.0;
const ROW_H: f32 = 30.0;

pub fn ui(ui: &mut Ui, project: &Project, lang: Lang) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, colors::PANEL);

    if project.signals.is_empty() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            tr(lang, T::TimingEmpty),
            FontId::proportional(13.0),
            colors::TEXT_WEAK,
        );
        return;
    }

    let now = project.time;
    let t0 = (now - WINDOW).max(0.0);
    let t1 = t0 + WINDOW;
    let plot = Rect::from_min_max(
        pos2(rect.left() + NAME_W, rect.top() + 8.0),
        pos2(rect.right() - 12.0, rect.bottom() - 24.0),
    );
    if plot.width() < 20.0 || plot.height() < 20.0 {
        return;
    }
    let x_of = |t: f64| plot.left() + ((t - t0) / WINDOW) as f32 * plot.width();

    // Svislé čáry po sekundách, popisky po pěti.
    for s in t0.ceil() as i64..=t1.floor() as i64 {
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

    for (row, signal) in project.signals.iter().enumerate() {
        let top = plot.top() + row as f32 * ROW_H;
        if top + ROW_H > plot.bottom() + 1.0 {
            break;
        }
        let hi = top + 7.0;
        let lo = top + ROW_H - 7.0;
        painter.text(
            pos2(rect.left() + 12.0, (hi + lo) / 2.0),
            Align2::LEFT_CENTER,
            &signal.label,
            FontId::monospace(13.0),
            colors::TEXT,
        );

        let mut prev_y: Option<f32> = None;
        for (i, &(from, v)) in signal.changes.iter().enumerate() {
            let to = signal.changes.get(i + 1).map_or(now, |n| n.0);
            if to < t0 {
                continue;
            }
            let (x0, x1) = (x_of(from.max(t0)), x_of(to));
            let y = match v {
                LogicValue::One => hi,
                LogicValue::Zero => lo,
                _ => (hi + lo) / 2.0,
            };
            if let Some(py) = prev_y
                && (py - y).abs() > 0.5
            {
                painter.line_segment(
                    [pos2(x0, py), pos2(x0, y)],
                    Stroke::new(1.5, colors::TEXT_WEAK),
                );
            }
            if matches!(v, LogicValue::One | LogicValue::X) {
                let alpha = if v == LogicValue::One { 0.12 } else { 0.25 };
                painter.rect_filled(
                    Rect::from_min_max(pos2(x0, hi), pos2(x1, lo)),
                    0.0,
                    signal_color(v).gamma_multiply(alpha),
                );
            }
            painter.line_segment(
                [pos2(x0, y), pos2(x1, y)],
                Stroke::new(2.0, signal_color(v)),
            );
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
    let x = x_of(now);
    painter.line_segment(
        [pos2(x, plot.top()), pos2(x, plot.bottom())],
        Stroke::new(1.0, colors::ACCENT),
    );
}
