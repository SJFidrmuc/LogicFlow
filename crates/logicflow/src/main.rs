//! LogicFlow – simulátor logických obvodů.

// V release verzi na Windows neotevírat okno konzole.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod demo;
mod i18n;
mod panels;
mod panes;
mod schematic;
mod theme;
mod timing;

fn main() -> eframe::Result {
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
        Box::new(|cc| Ok(Box::new(app::LogicFlowApp::new(cc)))),
    )
}

/// Ikona aplikace kreslená v kódu: zelený obdélníkový průběh na tmavém pozadí.
fn icon() -> egui::IconData {
    const N: usize = 64;
    let mut rgba = vec![0u8; N * N * 4];
    let wave = |x: usize| if (x / 16).is_multiple_of(2) { 40 } else { 22 };
    for y in 0..N {
        for x in 0..N {
            let i = (y * N + x) * 4;
            // Zaoblený čtverec.
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
