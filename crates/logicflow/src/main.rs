//! LogicFlow – simulátor logických obvodů.
//!
//! [en] LogicFlow – logic circuit simulator.

// V release verzi na Windows neotevírat okno konzole.
// [en] In the release build on Windows, do not open a console window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod catalog;
mod commands;
mod examples;
mod filedialog;
mod help;
mod i18n;
mod panels;
mod panes;
mod project;
mod schematic;
mod symbols;
mod theme;
mod timing;

/// Spuštění aplikace: okno, ikona, uložené nastavení a soubor z příkazové řádky.
///
/// [en] Application start: window, icon, saved settings and the file from the command line.
fn main() -> eframe::Result {
    // Soubor předaný na příkazové řádce (nebo dvojklikem) se rovnou otevře.
    // [en] A file given on the command line (or by double-click) is opened right away.
    let open = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LogicFlow")
            .with_app_id("logicflow")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(icon()),
        ..Default::default()
    };
    eframe::run_native(
        "LogicFlow",
        options,
        Box::new(|cc| Ok(Box::new(app::LogicFlowApp::new(cc, open)))),
    )
}

/// Ikona aplikace kreslená v kódu: zelený obdélníkový průběh na tmavém pozadí.
///
/// [en] Application icon drawn in code: a green square wave on a dark background.
fn icon() -> egui::IconData {
    /// Velikost ikony v pixelech.
    ///
    /// [en] Icon size in pixels.
    const N: usize = 64;
    let mut rgba = vec![0u8; N * N * 4];
    let wave = |x: usize| if (x / 16).is_multiple_of(2) { 40 } else { 22 };
    for y in 0..N {
        for x in 0..N {
            let i = (y * N + x) * 4;
            // Zaoblený čtverec.
            // [en] Rounded square.
            let (dx, dy) = (x.min(N - 1 - x), y.min(N - 1 - y));
            let corner = dx < 10 && dy < 10 && (10 - dx).pow(2) + (10 - dy).pow(2) > 100;
            if corner {
                continue;
            }
            let mut c = [0x30, 0x30, 0x30, 0xff];
            let level = wave(x);
            let on_h = (y as i32 - level).abs() <= 2 && (6..58).contains(&x);
            let edge = x % 16 == 0 && (6..58).contains(&x) && (22..=40).contains(&y);
            if on_h || edge {
                c = [0x7e, 0xe0, 0x7e, 0xff];
            }
            rgba[i..i + 4].copy_from_slice(&c);
        }
    }
    egui::IconData {
        rgba,
        width: N as u32,
        height: N as u32,
    }
}
