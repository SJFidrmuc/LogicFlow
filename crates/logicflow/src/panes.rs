//! Oblasti (areas) ve stylu Blenderu.
//!
//! Každá oblast má záhlaví s volbou typu editoru. Oblasti jde přetahovat
//! za záhlaví, měnit jejich velikost a skládat do záložek.
//!
//! [en] Areas in the style of Blender.
//!
//! Each area has a header with a choice of editor type. Areas can be dragged
//! by their header, resized and stacked into tabs.

use crate::i18n::{Lang, T, tr};
use crate::panels::{self, TruthCache};
use crate::project::Project;
use crate::schematic::SchematicView;
use crate::theme::colors;
use crate::timing;
use egui::{RichText, Sense, Ui};
use egui_tiles::{Behavior, SimplificationOptions, TileId, Tiles, Tree, UiResponse};

/// Typ editoru zobrazeného v oblasti.
///
/// [en] Type of editor shown in an area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorKind {
    /// Editor schématu.
    ///
    /// [en] Schematic editor.
    Schematic,
    /// Časový diagram.
    ///
    /// [en] Timing diagram.
    Timing,
    /// Vlastnosti vybrané součástky.
    ///
    /// [en] Properties of the selected component.
    Properties,
    /// Hierarchie: listy projektu a součástky.
    ///
    /// [en] Hierarchy: project sheets and components.
    Hierarchy,
    /// Knihovna součástek.
    ///
    /// [en] Component library.
    Library,
    /// Pravdivostní tabulka.
    ///
    /// [en] Truth table.
    TruthTable,
    /// Paměti (připravuje se).
    ///
    /// [en] Memories (coming later).
    Memory,
    /// Nápověda.
    ///
    /// [en] Help.
    Help,
}

impl EditorKind {
    /// Všechny typy editorů v pořadí nabídky.
    ///
    /// [en] All editor types in menu order.
    pub const ALL: [Self; 8] = [
        Self::Schematic,
        Self::Timing,
        Self::Properties,
        Self::Hierarchy,
        Self::Library,
        Self::TruthTable,
        Self::Memory,
        Self::Help,
    ];

    /// Název typu editoru.
    ///
    /// [en] Name of the editor type.
    pub fn title(self, lang: Lang) -> &'static str {
        tr(
            lang,
            match self {
                Self::Schematic => T::EdSchematic,
                Self::Timing => T::EdTiming,
                Self::Properties => T::EdProperties,
                Self::Hierarchy => T::EdHierarchy,
                Self::Library => T::EdLibrary,
                Self::TruthTable => T::EdTruthTable,
                Self::Memory => T::EdMemory,
                Self::Help => T::EdHelp,
            },
        )
    }
}

/// Obsah jedné oblasti: typ editoru a jeho vlastní stav.
///
/// [en] Content of one area: the editor type and its own state.
pub struct Pane {
    /// Zobrazený typ editoru.
    ///
    /// [en] Editor type shown.
    pub kind: EditorKind,
    /// Pohled editoru schématu (každá oblast má vlastní).
    ///
    /// [en] Schematic editor view (each area has its own).
    pub view: SchematicView,
    /// Text hledání v knihovně součástek.
    ///
    /// [en] Search text in the component library.
    pub search: String,
    /// Kapitola nápovědy zobrazená v oblasti typu Nápověda.
    ///
    /// [en] Help chapter shown in an area of type Help.
    pub help_section: usize,
}

impl Pane {
    /// Oblast s daným typem editoru.
    ///
    /// [en] Area with the given editor type.
    pub fn new(kind: EditorKind) -> Self {
        Self {
            kind,
            view: SchematicView::default(),
            search: String::new(),
            help_section: 0,
        }
    }
}

/// Požadavek na změnu rozložení oblastí (provede se po vykreslení stromu).
///
/// [en] Request to change the area layout (carried out after the tree is drawn).
#[derive(Clone, Copy, Debug)]
pub enum AreaRequest {
    /// Rozdělit oblast v daném směru.
    ///
    /// [en] Split the area in the given direction.
    Split(TileId, egui_tiles::LinearDir),
    /// Zavřít oblast.
    ///
    /// [en] Close the area.
    Close(TileId),
}

/// Sdílený kontext pro vykreslení oblastí v jednom snímku.
///
/// [en] Shared context for drawing the areas in one frame.
pub struct AreaBehavior<'a> {
    /// Otevřený projekt.
    ///
    /// [en] The open project.
    pub project: &'a mut Project,
    /// Mezivýsledek pravdivostní tabulky (přepočítá se při změně obvodu).
    ///
    /// [en] Cached truth table (recomputed when the circuit changes).
    pub truth_cache: &'a mut TruthCache,
    /// Jazyk rozhraní.
    ///
    /// [en] Interface language.
    pub lang: Lang,
    /// Požadavky na rozdělení a zavření oblastí z tohoto snímku.
    ///
    /// [en] Requests to split and close areas from this frame.
    pub requests: Vec<AreaRequest>,
    /// Lze oblast zavřít (není poslední)?
    ///
    /// [en] Can the area be closed (is it not the last one)?
    pub can_close: bool,
}

/// Výška záhlaví oblasti.
///
/// [en] Height of an area header.
const HEADER_H: f32 = 26.0;

impl Behavior<Pane> for AreaBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.kind.title(self.lang).into()
    }

    /// Záhlaví oblasti (typ editoru, list, nabídka Oblast) a obsah editoru.
    ///
    /// [en] Area header (editor type, sheet, Area menu) and the editor content.
    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        let lang = self.lang;
        let mut drag_started = false;

        // Záhlaví oblasti.
        // [en] Area header.
        egui::Frame::new()
            .fill(colors::HEADER)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.set_height(HEADER_H - 4.0);
                ui.horizontal_centered(|ui| {
                    egui::ComboBox::from_id_salt(("editor", tile_id))
                        .selected_text(pane.kind.title(lang))
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            for kind in EditorKind::ALL {
                                ui.selectable_value(&mut pane.kind, kind, kind.title(lang));
                            }
                        })
                        .response
                        .on_hover_text(tr(lang, T::EditorType));

                    if pane.kind == EditorKind::Schematic {
                        if ui
                            .button(tr(lang, T::FitView))
                            .on_hover_text("Home")
                            .clicked()
                        {
                            pane.view.needs_fit = true;
                        }
                        ui.label(
                            RichText::new(format!("{:.0} %", pane.view.zoom * 100.0))
                                .color(colors::TEXT_WEAK),
                        );
                        // Kde právě jsme: Hlavní obvod / podobvod.
                        // [en] Where we are now: Main circuit / subcircuit.
                        if let Some(name) = self.project.sheet_name() {
                            ui.separator();
                            if ui
                                .link(tr(lang, T::MainCircuit))
                                .on_hover_text(tr(lang, T::BackToParent))
                                .clicked()
                            {
                                self.project.open_sheet(None);
                            }
                            ui.label(RichText::new(format!("/  {name}")).color(colors::ACCENT));
                        }
                    }

                    // Nabídka oblasti vpravo, zbytek záhlaví je úchyt pro přetažení.
                    // [en] Area menu on the right, the rest of the header is a handle for dragging.
                    let mut menu = |ui: &mut Ui| self.area_menu(ui, tile_id);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button(tr(lang, T::AreaMenu), &mut menu)
                            .response
                            .on_hover_text(tr(lang, T::AreaMenuHint));
                        let rest = ui.available_rect_before_wrap();
                        let handle = ui.allocate_rect(rest, Sense::click_and_drag());
                        if handle.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                        }
                        drag_started = handle.drag_started();
                        handle.context_menu(&mut menu);
                    });
                });
            });

        // Obsah oblasti.
        // [en] Area content.
        let p = &mut *self.project;
        match pane.kind {
            EditorKind::Schematic => pane.view.ui(ui, p, lang),
            EditorKind::Timing => timing::ui(ui, p, lang),
            EditorKind::Properties => panels::properties(ui, p, lang),
            EditorKind::Hierarchy => panels::hierarchy(ui, p, lang),
            EditorKind::Library => panels::library(ui, &mut pane.search, p, lang),
            EditorKind::TruthTable => panels::truth_table(ui, p, self.truth_cache, lang),
            EditorKind::Memory => panels::memory(ui, lang),
            EditorKind::Help => crate::help::area(ui, &mut pane.help_section, lang),
        }

        if drag_started {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: false,
            ..Default::default()
        }
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        3.0
    }

    fn tab_bar_color(&self, _visuals: &egui::Visuals) -> egui::Color32 {
        colors::GAP
    }
}

impl AreaBehavior<'_> {
    /// Nabídka Oblast: rozdělit vodorovně / svisle, zavřít.
    ///
    /// [en] The Area menu: split horizontally / vertically, close.
    fn area_menu(&mut self, ui: &mut Ui, tile: TileId) {
        use egui_tiles::LinearDir::{Horizontal, Vertical};
        let lang = self.lang;
        if ui.button(tr(lang, T::SplitSide)).clicked() {
            self.requests.push(AreaRequest::Split(tile, Horizontal));
            ui.close();
        }
        if ui.button(tr(lang, T::SplitBelow)).clicked() {
            self.requests.push(AreaRequest::Split(tile, Vertical));
            ui.close();
        }
        ui.separator();
        if ui
            .add_enabled(self.can_close, egui::Button::new(tr(lang, T::CloseArea)))
            .clicked()
        {
            self.requests.push(AreaRequest::Close(tile));
            ui.close();
        }
    }
}

/// Provede požadavky na rozdělení a zavření oblastí.
///
/// [en] Carries out the requests to split and close areas.
pub fn apply_requests(tree: &mut Tree<Pane>, requests: Vec<AreaRequest>) {
    for r in requests {
        match r {
            AreaRequest::Split(tile, dir) => split(tree, tile, dir),
            AreaRequest::Close(tile) => {
                if count_panes(tree) > 1 {
                    tree.remove_recursively(tile);
                }
            }
        }
    }
}

/// Počet oblastí ve stromu.
///
/// [en] Number of areas in the tree.
pub fn count_panes(tree: &Tree<Pane>) -> usize {
    tree.tiles
        .tiles()
        .filter(|t| matches!(t, egui_tiles::Tile::Pane(_)))
        .count()
}

/// Rozdělí oblast na dvě; nová oblast zobrazí stejný typ editoru (jako v Blenderu).
///
/// [en] Splits an area in two; the new area shows the same editor type (as in Blender).
fn split(tree: &mut Tree<Pane>, tile: TileId, dir: egui_tiles::LinearDir) {
    let Some(egui_tiles::Tile::Pane(p)) = tree.tiles.get(tile) else {
        return;
    };
    let new = tree.tiles.insert_pane(Pane::new(p.kind));
    let parent = tree.tiles.parent_of(tile);

    // Rodič je řada ve stejném směru: stačí vložit vedle.
    // [en] The parent is a row in the same direction: inserting alongside is enough.
    if let Some(pid) = parent
        && let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(lin))) =
            tree.tiles.get_mut(pid)
        && lin.dir == dir
    {
        let idx = lin
            .children
            .iter()
            .position(|c| *c == tile)
            .unwrap_or(lin.children.len());
        let share = lin.shares[tile];
        lin.children.insert(idx + 1, new);
        lin.shares.set_share(tile, share / 2.0);
        lin.shares.set_share(new, share / 2.0);
        return;
    }

    // Jinak se oblast nahradí novou řadou [oblast, nová].
    // [en] Otherwise the area is replaced by a new row [area, new].
    let pair = tree
        .tiles
        .insert_container(egui_tiles::Linear::new_binary(dir, [tile, new], 0.5));
    match parent {
        None => tree.root = Some(pair),
        Some(pid) => {
            if let Some(egui_tiles::Tile::Container(c)) = tree.tiles.get_mut(pid) {
                match c {
                    egui_tiles::Container::Linear(lin) => {
                        if let Some(slot) = lin.children.iter_mut().find(|c| **c == tile) {
                            *slot = pair;
                        }
                        lin.shares.replace_with(tile, pair);
                    }
                    egui_tiles::Container::Tabs(tabs) => {
                        if let Some(slot) = tabs.children.iter_mut().find(|c| **c == tile) {
                            *slot = pair;
                        }
                        if tabs.active == Some(tile) {
                            tabs.active = Some(pair);
                        }
                    }
                    egui_tiles::Container::Grid(_) => {
                        // V mřížce jen přidáme novou oblast jako další buňku.
                        // [en] In a grid we just add the new area as another cell.
                        tree.tiles.remove(pair);
                        if let Some(egui_tiles::Tile::Container(c)) = tree.tiles.get_mut(pid) {
                            c.add_child(new);
                        }
                    }
                }
            }
        }
    }
}

/// Pracovní plocha = pojmenované rozložení oblastí (jako v Blenderu).
///
/// [en] Workspace = a named layout of areas (as in Blender).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workspace {
    /// Návrh: schéma, knihovna, vlastnosti.
    ///
    /// [en] Design: schematic, library, properties.
    Design,
    /// Simulace: schéma a časový diagram.
    ///
    /// [en] Simulation: schematic and timing diagram.
    Simulation,
    /// Analýza: pravdivostní tabulka.
    ///
    /// [en] Analysis: truth table.
    Analysis,
    /// Paměť (připravuje se).
    ///
    /// [en] Memory (coming later).
    Memory,
}

impl Workspace {
    /// Všechny pracovní plochy v pořadí záložek.
    ///
    /// [en] All workspaces in tab order.
    pub const ALL: [Self; 4] = [Self::Design, Self::Simulation, Self::Analysis, Self::Memory];

    /// Název pracovní plochy.
    ///
    /// [en] Workspace name.
    pub fn title(self, lang: Lang) -> &'static str {
        tr(
            lang,
            match self {
                Self::Design => T::WsDesign,
                Self::Simulation => T::WsSimulation,
                Self::Analysis => T::WsAnalysis,
                Self::Memory => T::WsMemory,
            },
        )
    }

    /// Výchozí rozložení oblastí pracovní plochy.
    ///
    /// [en] Default area layout of the workspace.
    pub fn default_tree(self) -> Tree<Pane> {
        use EditorKind as E;
        use egui_tiles::LinearDir::{Horizontal, Vertical};
        let mut tiles: Tiles<Pane> = Tiles::default();
        let mut pane = |k: E| tiles.insert_pane(Pane::new(k));
        let root = match self {
            Self::Design => {
                let hier = pane(E::Hierarchy);
                let lib = pane(E::Library);
                let schem = pane(E::Schematic);
                let props = pane(E::Properties);
                let left = linear(&mut tiles, Vertical, &[(hier, 1.0), (lib, 1.4)]);
                linear(
                    &mut tiles,
                    Horizontal,
                    &[(left, 1.0), (schem, 4.0), (props, 1.2)],
                )
            }
            Self::Simulation => {
                let schem = pane(E::Schematic);
                let props = pane(E::Properties);
                let timing = pane(E::Timing);
                let top = linear(&mut tiles, Horizontal, &[(schem, 4.0), (props, 1.2)]);
                linear(&mut tiles, Vertical, &[(top, 2.0), (timing, 1.0)])
            }
            Self::Analysis => {
                let schem = pane(E::Schematic);
                let table = pane(E::TruthTable);
                linear(&mut tiles, Horizontal, &[(schem, 2.0), (table, 1.0)])
            }
            Self::Memory => {
                let schem = pane(E::Schematic);
                let mem = pane(E::Memory);
                linear(&mut tiles, Horizontal, &[(schem, 2.0), (mem, 1.0)])
            }
        };
        Tree::new(
            egui::Id::unique(("logicflow-workspace", self as u8)),
            root,
            tiles,
        )
    }
}

/// Řada oblastí v daném směru s poměry velikostí.
///
/// [en] Row of areas in the given direction with size shares.
fn linear(
    tiles: &mut Tiles<Pane>,
    dir: egui_tiles::LinearDir,
    children: &[(TileId, f32)],
) -> TileId {
    let mut lin = egui_tiles::Linear::new(dir, children.iter().map(|(id, _)| *id).collect());
    for &(id, share) in children {
        lin.shares.set_share(id, share);
    }
    tiles.insert_container(lin)
}

/// Po otevření jiného obvodu: všechny editory schématu se přizpůsobí novému obsahu.
///
/// [en] After opening another circuit: all schematic editors adapt to the new content.
pub fn reset_views(tree: &mut Tree<Pane>) {
    for tile in tree.tiles.tiles_mut() {
        if let egui_tiles::Tile::Pane(p) = tile {
            p.view = SchematicView::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_pane(tree: &Tree<Pane>, kind: EditorKind) -> TileId {
        tree.tiles
            .iter()
            .find_map(|(id, t)| match t {
                egui_tiles::Tile::Pane(p) if p.kind == kind => Some(*id),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn split_and_close_areas() {
        let mut tree = Workspace::Analysis.default_tree();
        assert_eq!(count_panes(&tree), 2);
        let schem = first_pane(&tree, EditorKind::Schematic);
        // Ve stejném směru jako rodič.
        // [en] In the same direction as the parent.
        apply_requests(
            &mut tree,
            vec![AreaRequest::Split(schem, egui_tiles::LinearDir::Horizontal)],
        );
        assert_eq!(count_panes(&tree), 3);
        // Kolmo na rodiče – vznikne nová řada.
        // [en] Perpendicular to the parent – a new row appears.
        apply_requests(
            &mut tree,
            vec![AreaRequest::Split(schem, egui_tiles::LinearDir::Vertical)],
        );
        assert_eq!(count_panes(&tree), 4);
        assert!(tree.tiles.parent_of(schem).is_some());
        apply_requests(&mut tree, vec![AreaRequest::Close(schem)]);
        assert_eq!(count_panes(&tree), 3);
    }

    #[test]
    fn last_area_cannot_be_closed() {
        let mut tiles: Tiles<Pane> = Tiles::default();
        let only = tiles.insert_pane(Pane::new(EditorKind::Schematic));
        let mut tree = Tree::new(egui::Id::unique("t"), only, tiles);
        apply_requests(&mut tree, vec![AreaRequest::Close(only)]);
        assert_eq!(count_panes(&tree), 1);
        // Rozdělení jediné (kořenové) oblasti.
        // [en] Splitting the only (root) area.
        apply_requests(
            &mut tree,
            vec![AreaRequest::Split(only, egui_tiles::LinearDir::Vertical)],
        );
        assert_eq!(count_panes(&tree), 2);
    }
}
