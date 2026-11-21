//! Tmavé téma inspirované Blenderem.
//!
//! [en] Dark theme inspired by Blender.

use egui::{Color32, CornerRadius, Stroke, Visuals};
use logicflow_core::LogicValue;

/// Barvy rozhraní.
///
/// [en] Interface colours.
pub mod colors {
    use egui::Color32;

    /// Horní lišta a záhlaví oblastí.
    ///
    /// [en] Top bar and area headers.
    pub const HEADER: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
    /// Pozadí panelů (vlastnosti, hierarchie…).
    ///
    /// [en] Panel background (properties, hierarchy…).
    pub const PANEL: Color32 = Color32::from_rgb(0x28, 0x28, 0x28);
    /// Mezery mezi oblastmi.
    ///
    /// [en] Gaps between areas.
    pub const GAP: Color32 = Color32::from_rgb(0x16, 0x16, 0x16);
    /// Pozadí plátna schématu.
    ///
    /// [en] Background of the schematic canvas.
    pub const CANVAS: Color32 = Color32::from_rgb(0x39, 0x39, 0x39);
    /// Jemná mřížka schématu.
    ///
    /// [en] Fine grid of the schematic.
    pub const GRID_MINOR: Color32 = Color32::from_rgb(0x41, 0x41, 0x41);
    /// Hrubá mřížka schématu (každý pátý bod).
    ///
    /// [en] Coarse grid of the schematic (every fifth point).
    pub const GRID_MAJOR: Color32 = Color32::from_rgb(0x4b, 0x4b, 0x4b);
    /// Tlačítka a pole.
    ///
    /// [en] Buttons and fields.
    pub const WIDGET: Color32 = Color32::from_rgb(0x54, 0x54, 0x54);
    /// Tlačítka a pole pod ukazatelem myši.
    ///
    /// [en] Buttons and fields under the mouse pointer.
    pub const WIDGET_HOVER: Color32 = Color32::from_rgb(0x65, 0x65, 0x65);
    /// Modrá pro výběr, jako v Blenderu.
    ///
    /// [en] Blue for the selection, as in Blender.
    pub const ACCENT: Color32 = Color32::from_rgb(0x47, 0x72, 0xb3);
    /// Běžný text.
    ///
    /// [en] Regular text.
    pub const TEXT: Color32 = Color32::from_rgb(0xe6, 0xe6, 0xe6);
    /// Méně důležitý text (popisky, nápovědy).
    ///
    /// [en] Less important text (captions, hints).
    pub const TEXT_WEAK: Color32 = Color32::from_rgb(0xa0, 0xa0, 0xa0);
    /// Obrys součástek.
    ///
    /// [en] Outline of components.
    pub const SYMBOL: Color32 = Color32::from_rgb(0xdc, 0xdc, 0xdc);
    /// Výplň značek.
    ///
    /// [en] Fill of symbols.
    pub const SYMBOL_FILL: Color32 = Color32::from_rgb(0x2c, 0x2c, 0x2c);
}

/// Barva vodiče podle logické hodnoty.
///
/// [en] Wire colour according to the logic value.
pub fn signal_color(v: LogicValue) -> Color32 {
    match v {
        LogicValue::Zero => Color32::from_rgb(0x2f, 0x7a, 0x3c),
        LogicValue::One => Color32::from_rgb(0x7e, 0xe0, 0x7e),
        LogicValue::X => Color32::from_rgb(0xe0, 0x52, 0x52),
        LogicValue::Z => Color32::from_rgb(0x5b, 0x9b, 0xd5),
    }
}

/// Nastaví vzhled celé aplikace.
///
/// [en] Sets the look of the whole application.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    let r = CornerRadius::same(4);

    visuals.panel_fill = colors::PANEL;
    visuals.window_fill = colors::HEADER;
    visuals.extreme_bg_color = Color32::from_rgb(0x1d, 0x1d, 0x1d);
    visuals.faint_bg_color = Color32::from_rgb(0x2d, 0x2d, 0x2d);
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.menu_corner_radius = CornerRadius::same(4);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(0x1a, 0x1a, 0x1a));
    visuals.selection.bg_fill = colors::ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, colors::TEXT);
    visuals.hyperlink_color = Color32::from_rgb(0x7a, 0xa6, 0xe8);
    visuals.override_text_color = Some(colors::TEXT);

    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = colors::PANEL;
    w.noninteractive.weak_bg_fill = colors::PANEL;
    w.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x3a, 0x3a, 0x3a));
    w.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT);
    for (state, fill) in [
        (&mut w.inactive, colors::WIDGET),
        (&mut w.hovered, colors::WIDGET_HOVER),
        (&mut w.active, colors::ACCENT),
        (&mut w.open, colors::WIDGET_HOVER),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.corner_radius = r;
        state.fg_stroke = Stroke::new(1.0, colors::TEXT);
        state.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x3d, 0x3d, 0x3d));
    }
    // Neaktivní tlačítka v liště jsou tmavší, jako v Blenderu.
    // [en] Inactive buttons in the bar are darker, as in Blender.
    w.inactive.weak_bg_fill = Color32::from_rgb(0x3d, 0x3d, 0x3d);

    ctx.set_visuals_of(egui::Theme::Dark, visuals);
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 3.0);
        style.spacing.interact_size.y = 22.0;
    });
}
