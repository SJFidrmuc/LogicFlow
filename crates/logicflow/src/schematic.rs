//! Editor schématu: nekonečné plátno s mřížkou, přidávání a úpravy součástek,
//! kreslení vodičů. Ovládání vychází z Blenderu (Shift+A, G, R, X, Shift+D, A).
//!
//! [en] Schematic editor: an endless canvas with a grid, adding and editing components,
//! drawing wires. The controls follow Blender (Shift+A, G, R, X, Shift+D, A).

use crate::catalog;
use crate::commands::{self, Command, Scope};
use crate::i18n::{Lang, T, tr};
use crate::project::Project;
use crate::symbols::{self, Camera, Look, UNIT, Values, dist_to_segment, point, snap};
use crate::theme::{colors, signal_color};
use egui::{
    Align2, Color32, FontId, Painter, PointerButton, Pos2, Rect, Sense, Stroke, Ui, Vec2, pos2,
    vec2,
};
use logicflow_core::{Circuit, CompId, Component, ComponentKind, Point, Rotation, WireId};

/// Barva podbarvení sítě se zkratem napájení.
///
/// [en] Highlight colour of a net with a power short.
const POWER_SHORT: Color32 = Color32::from_rgba_premultiplied(0x90, 0x20, 0x18, 0xa0);
/// Nejmenší zvětšení.
///
/// [en] Smallest zoom.
const MIN_ZOOM: f32 = 0.15;
/// Největší zvětšení.
///
/// [en] Largest zoom.
const MAX_ZOOM: f32 = 6.0;

/// Nejdelší odstup dvou stisků dvojkliku na tlačítku v obvodu (s).
///
/// [en] Longest gap between the two presses of a double-click on a push button (s).
const DOUBLE_CLICK_S: f64 = 0.4;
/// Jak dlouho svítí hláška „Pro úpravy zastavte simulaci“ (s).
///
/// [en] How long the message "Stop the simulation to edit" is shown (s).
const LOCKED_HINT_S: f64 = 2.5;

/// Právě používaný nástroj editoru – stavový automat ovládání myší a klávesnicí.
///
/// [en] Editor tool currently in use – the state machine of mouse and keyboard control.
#[derive(Clone, Debug)]
enum Tool {
    /// Žádná rozdělaná akce.
    ///
    /// [en] No action in progress.
    Idle,
    /// Umisťování nové součástky (náhled sleduje myš).
    ///
    /// [en] Placing a new component (the preview follows the mouse).
    Place {
        /// Druh vkládané součástky.
        ///
        /// [en] Kind of the component being placed.
        kind: ComponentKind,
        /// Otočení náhledu (R).
        ///
        /// [en] Rotation of the preview (R).
        rotation: Rotation,
    },
    /// Přesun výběru. `origin` je stav před přesunem.
    ///
    /// [en] Moving the selection. `origin` is the state before the move.
    Grab {
        origin: Box<Circuit>,
        /// Bod mřížky, odkud přesun začal.
        ///
        /// [en] Grid point where the move started.
        start: Point,
        /// Aktuální posun v bodech mřížky.
        ///
        /// [en] Current offset in grid points.
        delta: (i32, i32),
        /// Přesouvané součástky.
        ///
        /// [en] Components being moved.
        comps: Vec<CompId>,
        /// Přesouvané vodiče.
        ///
        /// [en] Wires being moved.
        wires: Vec<WireId>,
        /// Přesun začal klávesou G (končí kliknutím), ne tažením myší.
        ///
        /// [en] The move started with the G key (ends with a click), not by dragging.
        by_key: bool,
        /// Stav pro „Zpět“ už je uložený (např. u Shift+D).
        ///
        /// [en] The “Undo” state has already been saved (e.g. for Shift+D).
        undo_pushed: bool,
        /// Byl projekt změněný už před přesunem (pro zrušení přesunu).
        ///
        /// [en] Was the project modified before the move (for cancelling the move)?
        was_modified: bool,
    },
    /// Kreslení vodiče z bodu `start`.
    ///
    /// [en] Drawing a wire from point `start`.
    Wire {
        start: Point,
        /// Směr prvního úseku (None = podle pohybu myši).
        ///
        /// [en] Direction of the first segment (None = according to the mouse movement).
        horizontal_first: Option<bool>,
    },
    /// Výběr obdélníkem (ve světových souřadnicích).
    ///
    /// [en] Box selection (in world coordinates).
    BoxSelect {
        start: Pos2,
        /// Se Shift se přidává k dosavadnímu výběru.
        ///
        /// [en] With Shift the box adds to the existing selection.
        additive: bool,
    },
    /// Držené tlačítko.
    ///
    /// [en] Button held down.
    Press(CompId),
}

/// Kamera a stav nástroje jednoho editoru schématu (každá oblast má vlastní).
///
/// [en] Camera and tool state of one schematic editor (each area has its own).
#[derive(Clone, Debug)]
pub struct SchematicView {
    /// Posun pohledu v pixelech.
    ///
    /// [en] View offset in pixels.
    pub offset: Vec2,
    /// Zvětšení (1 = 100 %).
    ///
    /// [en] Zoom (1 = 100 %).
    pub zoom: f32,
    /// Při příštím snímku přizpůsobit pohled celému schématu.
    ///
    /// [en] Fit the view to the whole schematic in the next frame.
    pub needs_fit: bool,
    /// Aktuální nástroj.
    ///
    /// [en] Current tool.
    tool: Tool,
    /// Poloha otevřené nabídky Přidat (Shift+A), jinak None.
    ///
    /// [en] Position of the open Add menu (Shift+A), otherwise None.
    add_menu: Option<Pos2>,
    /// List, který byl zobrazený naposledy (při změně se pohled přizpůsobí).
    ///
    /// [en] Sheet shown last time (the view adapts when it changes).
    shown_sheet: Option<(bool, Option<u32>)>,
    /// Do kdy (čas egui v s) ukazovat hlášku „Pro úpravy zastavte simulaci“.
    ///
    /// [en] Until when (egui time in s) to show the "Stop the simulation to edit" message.
    locked_hint_until: f64,
    /// Poslední stisk na tlačítku v obvodu (id, čas) – v režimu úprav se tlačítko
    /// stiskne až druhým stiskem dvojkliku.
    ///
    /// [en] Last press on a push button in the circuit (id, time) – in edit mode the
    /// button is pressed only by the second press of a double-click.
    last_button_press: Option<(CompId, f64)>,
    /// Na co se kliklo pravým tlačítkem – podle toho se plní kontextová nabídka.
    ///
    /// [en] What was right-clicked – the context menu is filled accordingly.
    menu_target: Option<MenuTarget>,
    /// Bod mřížky, kam se kliklo pravým tlačítkem (sem se vloží součástka z nabídky).
    ///
    /// [en] Grid point that was right-clicked (a component from the menu is placed here).
    menu_point: Point,
}

/// Cíl kontextové nabídky.
///
/// [en] Target of the context menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuTarget {
    /// Jedna součástka.
    ///
    /// [en] One component.
    Component(CompId),
    /// Jeden vodič.
    ///
    /// [en] One wire.
    Wire(WireId),
    /// Výběr více součástek nebo vodičů.
    ///
    /// [en] A selection of several components or wires.
    Selection,
    /// Prázdné místo schématu.
    ///
    /// [en] An empty place of the schematic.
    Empty,
}

impl Default for SchematicView {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
            needs_fit: true,
            tool: Tool::Idle,
            add_menu: None,
            shown_sheet: None,
            locked_hint_until: 0.0,
            last_button_press: None,
            menu_target: None,
            menu_point: Point::new(0, 0),
        }
    }
}

impl SchematicView {
    /// Kamera pro oblast `rect` podle posunu a zvětšení.
    ///
    /// [en] Camera for area `rect` according to the offset and zoom.
    fn camera(&self, rect: Rect) -> Camera {
        Camera {
            origin: rect.center() + self.offset,
            scale: UNIT * self.zoom,
        }
    }

    /// Přizpůsobí posun a zvětšení tak, aby bylo vidět celé schéma.
    ///
    /// [en] Adjusts the offset and zoom so that the whole schematic is visible.
    fn fit(&mut self, rect: Rect, circuit: &Circuit) {
        let (x0, y0, x1, y1) = circuit.bounds().unwrap_or((-20.0, -12.0, 20.0, 12.0));
        let size = vec2((x1 - x0).max(10.0), (y1 - y0).max(8.0)) * UNIT;
        let margin = 0.9;
        self.zoom = (rect.width() * margin / size.x)
            .min(rect.height() * margin / size.y)
            .clamp(MIN_ZOOM, 2.5);
        let center = vec2((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        self.offset = -center * UNIT * self.zoom;
        self.needs_fit = false;
    }

    /// Vykreslí editor a zpracuje vstupy myši a klávesnice.
    ///
    /// [en] Draws the editor and handles mouse and keyboard input.
    pub fn ui(&mut self, ui: &mut Ui, project: &mut Project, lang: Lang) {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        if self.shown_sheet != Some(project.sheet_key()) {
            if self.shown_sheet.is_some() {
                self.tool = Tool::Idle;
                self.add_menu = None;
            }
            self.shown_sheet = Some(project.sheet_key());
            self.needs_fit = true;
        }
        if self.needs_fit && rect.width() > 10.0 {
            self.fit(rect, &project.circuit);
        }
        // „Najít ve schématu“ z Hierarchie: pohled se vystředí na součástku.
        // [en] "Find in schematic" from the Hierarchy: the view is centred on the component.
        if let Some(c) = project
            .focus_request
            .take()
            .and_then(|id| project.circuit.component(id))
        {
            self.offset = -vec2(c.pos.x as f32, c.pos.y as f32) * UNIT * self.zoom;
        }

        // Posun prostředním tlačítkem (nebo tažením pravým).
        // [en] Panning with the middle button (or by dragging with the right one).
        if response.dragged_by(PointerButton::Middle)
            || response.dragged_by(PointerButton::Secondary)
        {
            self.offset += response.drag_delta();
        }
        if response.hovered() {
            let (scroll, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
            let factor = (scroll * 0.0025).exp() * pinch;
            if (factor - 1.0).abs() > f32::EPSILON
                && let Some(mouse) = response.hover_pos()
            {
                let world = self.camera(rect).to_world(mouse);
                self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
                self.offset += mouse - self.camera(rect).to_screen(world);
            }
        }

        let cam = self.camera(rect);
        let mouse_screen = ui
            .input(|i| i.pointer.hover_pos())
            .filter(|p| rect.contains(*p));
        let mouse = mouse_screen.map(|p| cam.to_world(p));
        let mouse_pt = mouse.map(snap);

        // V režimu simulace se rozdělané úpravy zruší (např. F5 během přesunu).
        // [en] In simulation mode, edits in progress are cancelled (e.g. F5 during a move).
        if !project.can_edit() {
            self.add_menu = None;
            if matches!(
                self.tool,
                Tool::Place { .. } | Tool::Grab { .. } | Tool::Wire { .. }
            ) {
                self.cancel(project);
            }
            project.pending_place = None;
            project.pending_paste = None;
        }

        // Součástka vybraná v knihovně se umístí do editoru, nad kterým je myš.
        // [en] A component chosen in the library is placed in the editor the mouse is over.
        if response.hovered()
            && let Some(kind) = project.pending_place.take()
        {
            self.tool = Tool::Place {
                kind,
                rotation: Rotation::R0,
            };
        }
        // Prvek knihovny se vloží jako Ctrl+V: objeví se u ukazatele myši a klik ho položí.
        // [en] A library element is inserted like Ctrl+V: it appears at the mouse pointer
        // and a click drops it.
        if response.hovered()
            && let Some(start) = mouse_pt
            && let Some((_, text)) = project.pending_paste.take()
            && project.paste_text(&text, Some(start))
        {
            self.start_grab(project, start, true, true);
        }

        if response.hovered() && !ui.ctx().egui_wants_keyboard_input() {
            self.keyboard(ui, rect, project, mouse_pt, lang);
        }
        // Pravé tlačítko během rozdělané akce ji ruší (viz `pointer`) – nabídka se neotevře.
        // [en] The right button during an action in progress cancels it (see `pointer`) – no menu opens.
        let menu_allowed = matches!(self.tool, Tool::Idle) || !response.secondary_clicked();
        self.pointer(ui, &response, cam, project, mouse, mouse_pt);
        if menu_allowed {
            if response.secondary_clicked()
                && let Some(m) = mouse
            {
                self.open_menu_at(project, m);
            }
            response.context_menu(|ui| self.context_menu_ui(ui, rect, project, lang));
        }

        // --- Kreslení ---
        // [en]
        //
        // --- Drawing ---
        painter.rect_filled(rect, 0.0, colors::CANVAS);
        draw_grid(&painter, rect, cam);
        self.draw_circuit(&painter, cam, project, mouse);
        self.draw_tool(&painter, cam, mouse_pt, project);

        if project.circuit.components.is_empty() && project.circuit.wires.is_empty() {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                tr(lang, T::EmptySchematic),
                FontId::proportional(15.0),
                colors::TEXT_WEAK,
            );
        }
        // Varování vlevo nahoře pod sebou: kmitání, zkrat napájení.
        // [en] Warnings at the top left, one below the other: oscillation, power short.
        let warnings = [
            (project.sim.is_oscillating(), T::Oscillating),
            (project.sim.has_power_short(), T::PowerShort),
        ];
        for (i, (_, key)) in warnings.iter().filter(|(on, _)| *on).enumerate() {
            painter.text(
                rect.left_top() + vec2(10.0, 10.0 + 20.0 * i as f32),
                Align2::LEFT_TOP,
                tr(lang, *key),
                FontId::proportional(13.0),
                Color32::from_rgb(0xff, 0x70, 0x60),
            );
        }
        self.draw_mode(ui, &painter, rect, project, lang);
        let hint = match self.tool {
            _ if project.is_simulating() && matches!(self.tool, Tool::Idle | Tool::Press(_)) => {
                T::HintSim
            }
            Tool::Idle | Tool::BoxSelect { .. } | Tool::Press(_) => T::HintIdle,
            Tool::Place { .. } => T::HintPlace,
            Tool::Grab { .. } => T::HintGrab,
            Tool::Wire { .. } => T::HintWire,
        };
        painter.text(
            rect.left_bottom() + vec2(10.0, -8.0),
            Align2::LEFT_BOTTOM,
            tr(lang, hint),
            FontId::proportional(12.0),
            colors::TEXT_WEAK,
        );

        self.add_menu_ui(ui, lang, catalog::subcircuit_items(project));
    }

    /// Klávesové zkratky editoru (Shift+A, G, R, X, Shift+D, A, Tab, Esc, Home) –
    /// seznam a zkratky jsou v `commands.rs`.
    ///
    /// [en] Editor keyboard shortcuts (Shift+A, G, R, X, Shift+D, A, Tab, Esc, Home) –
    /// the list and the shortcuts are in `commands.rs`.
    fn keyboard(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        project: &mut Project,
        mouse_pt: Option<Point>,
        lang: Lang,
    ) {
        let mut pressed = commands::take_pressed(ui.ctx(), Scope::Schematic);
        // Tab přichází zvlášť (raw_input_hook v app.rs), aby ho egui nepoužilo k přesunu fokusu.
        // [en] Tab arrives separately (raw_input_hook in app.rs) so that egui does not use it to move the focus.
        if take_tab(ui.ctx()) {
            pressed.push(Command::Subcircuit);
        }
        for cmd in pressed {
            self.run_command(cmd, ui, rect, project, mouse_pt, lang);
        }
    }

    /// Provede příkaz editoru schématu. Během rozdělané akce (přesun, vodič, vkládání)
    /// fungují jen Esc, Home a otočení vkládané součástky.
    ///
    /// [en] Carries out a schematic editor command. While an action is in progress (move,
    /// wire, placing) only Esc, Home and rotating the component being placed work.
    pub fn run_command(
        &mut self,
        cmd: Command,
        ui: &Ui,
        rect: Rect,
        project: &mut Project,
        mouse_pt: Option<Point>,
        lang: Lang,
    ) {
        if cmd.blocked_by_simulation(project) {
            self.show_locked_hint(ui);
            return;
        }
        let busy = !matches!(self.tool, Tool::Idle);
        match cmd {
            Command::Cancel => {
                if self.add_menu.take().is_some() {
                    // jen zavřít nabídku
                    // [en] just close the menu
                } else if busy {
                    self.cancel(project);
                } else if project.is_simulating() {
                    // Esc v simulaci: zpět k úpravám (stav obvodu zůstává).
                    // [en] Esc in simulation: back to editing (the circuit state stays).
                    project.stop_simulation();
                } else {
                    project.selection.clear();
                }
            }
            Command::FitView => self.fit(rect, &project.circuit),
            Command::RunPause => project.toggle_run(),
            Command::Start => project.start_simulation(),
            Command::Stop => project.stop_simulation(),
            Command::Step => project.step(),
            Command::StepEdge => project.step_to_edge(false),
            Command::StepCycle => project.step_to_edge(true),
            Command::ResetSim => project.reset_simulation(),
            Command::Rotate => match &mut self.tool {
                Tool::Place { rotation, .. } => *rotation = rotation.next(),
                Tool::Idle => project.rotate_selection(),
                _ => {}
            },
            _ if busy => {}
            // Tab: do vybraného podobvodu a zpět (jako skupiny uzlů v Blenderu).
            // [en] Tab: into the selected subcircuit and back (like node groups in Blender).
            Command::Subcircuit => project.toggle_subcircuit(lang),
            Command::Add => self.add_menu = ui.input(|i| i.pointer.hover_pos()),
            Command::SelectAll => project.select_all(),
            Command::SelectNone => project.selection.clear(),
            Command::Delete => project.delete_selection(),
            Command::Duplicate => {
                if let Some(start) = mouse_pt
                    && !project.selection.is_empty()
                {
                    let before = project.circuit.clone();
                    let (comps, wires) = project
                        .circuit
                        .duplicate(&project.selection.comp_vec(), &project.selection.wire_vec());
                    project.push_undo(before);
                    project.selection.comps = comps.iter().copied().collect();
                    project.selection.wires = wires.iter().copied().collect();
                    project.after_structure_change();
                    self.start_grab(project, start, true, true);
                }
            }
            Command::Grab => {
                if let Some(start) = mouse_pt
                    && !project.selection.is_empty()
                {
                    self.start_grab(project, start, true, false);
                }
            }
            Command::Copy => {
                if let Some(text) = project.copy_selection() {
                    commands::set_clipboard(ui.ctx(), text);
                }
            }
            Command::Cut => {
                if let Some(text) = project.cut_selection() {
                    commands::set_clipboard(ui.ctx(), text);
                }
            }
            // Vložená část se objeví u ukazatele myši a jde s ní hned posunout (klik ji položí).
            // [en] The pasted part appears at the mouse pointer and can be moved at once (a click drops it).
            Command::Paste => {
                if let Some(text) = commands::take_paste(ui.ctx())
                    && project.paste_text(&text, mouse_pt)
                    && let Some(start) = mouse_pt
                {
                    self.start_grab(project, start, true, true);
                }
            }
            _ => {}
        }
    }

    /// Ukáže na chvíli hlášku „Pro úpravy zastavte simulaci (Esc)“.
    ///
    /// [en] Shows the message "Stop the simulation to edit (Esc)" for a moment.
    fn show_locked_hint(&mut self, ui: &Ui) {
        self.locked_hint_until = ui.input(|i| i.time) + LOCKED_HINT_S;
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(LOCKED_HINT_S));
    }

    /// Indikace režimu simulace: barevný rámeček a štítek SIMULACE, ruka nad
    /// ovládacími prvky, hláška po pokusu o úpravu.
    ///
    /// [en] Simulation mode indication: coloured frame and SIMULATION badge, a hand over
    /// the controls, a message after an attempt to edit.
    fn draw_mode(&self, ui: &Ui, painter: &Painter, rect: Rect, project: &Project, lang: Lang) {
        if project.is_simulating() {
            painter.rect_stroke(
                rect.shrink(1.5),
                0.0,
                Stroke::new(3.0, colors::SIMULATION),
                egui::StrokeKind::Inside,
            );
            let badge = tr(
                lang,
                if project.running {
                    T::SimBadge
                } else {
                    T::SimPausedBadge
                },
            );
            let galley = painter.layout_no_wrap(
                badge.to_owned(),
                FontId::proportional(12.0),
                Color32::BLACK,
            );
            let pos = rect.right_top() + vec2(-12.0 - galley.size().x, 10.0);
            painter.rect_filled(
                Rect::from_min_size(pos, galley.size()).expand2(vec2(7.0, 3.0)),
                4.0,
                colors::SIMULATION,
            );
            painter.galley(pos, galley, Color32::BLACK);
            // Ruka nad přepínači a tlačítky – na ně se v simulaci kliká.
            // [en] A hand over switches and buttons – they are clicked in simulation.
            if let Some(m) = ui
                .input(|i| i.pointer.hover_pos())
                .filter(|p| rect.contains(*p))
                && let Some(id) = component_at(&project.circuit, self.camera(rect).to_world(m))
                && matches!(
                    project.circuit.component(id).map(|c| c.kind),
                    Some(ComponentKind::Switch { .. } | ComponentKind::Button)
                )
            {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
        // Náhled vnitřního zapojení bloku: štítek jen pro čtení vlevo nahoře.
        // [en] Internal circuit of a block: a read-only badge at the top left.
        if let Some(block) = project.viewed_block() {
            let text = format!(
                "{} – {}",
                crate::catalog::block_name(block, lang),
                tr(lang, T::BlockViewTitle)
            );
            let galley = painter.layout_no_wrap(text, FontId::proportional(12.0), Color32::BLACK);
            let pos = rect.left_top() + vec2(12.0, 10.0);
            painter.rect_filled(
                Rect::from_min_size(pos, galley.size()).expand2(vec2(7.0, 3.0)),
                4.0,
                colors::ACCENT,
            );
            painter.galley(pos, galley, Color32::BLACK);
        }
        if ui.input(|i| i.time) < self.locked_hint_until {
            let galley = painter.layout_no_wrap(
                tr(lang, project.locked_hint()).to_owned(),
                FontId::proportional(14.0),
                colors::TEXT,
            );
            let pos = pos2(rect.center().x - galley.size().x / 2.0, rect.top() + 40.0);
            painter.rect_filled(
                Rect::from_min_size(pos, galley.size()).expand2(vec2(12.0, 7.0)),
                6.0,
                colors::HEADER,
            );
            painter.rect_stroke(
                Rect::from_min_size(pos, galley.size()).expand2(vec2(12.0, 7.0)),
                6.0,
                Stroke::new(1.0, colors::SIMULATION),
                egui::StrokeKind::Outside,
            );
            painter.galley(pos, galley, colors::TEXT);
        }
    }

    /// Zahájí přesun vybraných součástek a vodičů.
    ///
    /// [en] Starts moving the selected components and wires.
    fn start_grab(&mut self, project: &Project, start: Point, by_key: bool, undo_pushed: bool) {
        self.tool = Tool::Grab {
            origin: Box::new(project.circuit.clone()),
            start,
            delta: (0, 0),
            comps: project.selection.comp_vec(),
            wires: project.selection.wire_vec(),
            by_key,
            undo_pushed,
            was_modified: project.modified,
        };
    }

    /// Zruší rozdělanou akci (přesun vrátí do původního stavu, uvolní tlačítko).
    ///
    /// [en] Cancels the action in progress (a move returns to the original state, a button is released).
    fn cancel(&mut self, project: &mut Project) {
        match std::mem::replace(&mut self.tool, Tool::Idle) {
            Tool::Grab {
                origin,
                was_modified,
                ..
            } => {
                project.circuit = *origin;
                project.after_structure_change();
                project.modified = was_modified;
            }
            Tool::Press(id) => project.set_button(id, false),
            _ => {}
        }
    }

    /// Dokončí přesun a uloží stav pro „Zpět“.
    ///
    /// [en] Finishes the move and saves the state for “Undo”.
    fn finish_grab(&mut self, project: &mut Project) {
        if let Tool::Grab {
            origin,
            delta,
            undo_pushed,
            was_modified,
            ..
        } = std::mem::replace(&mut self.tool, Tool::Idle)
        {
            if delta != (0, 0) {
                if !undo_pushed {
                    project.push_undo(*origin);
                }
            } else if !undo_pushed {
                project.modified = was_modified;
            }
        }
    }

    /// Ovládání myší podle aktuálního nástroje.
    ///
    /// [en] Mouse control according to the current tool.
    fn pointer(
        &mut self,
        ui: &Ui,
        response: &egui::Response,
        cam: Camera,
        project: &mut Project,
        mouse: Option<Pos2>,
        mouse_pt: Option<Point>,
    ) {
        let shift = ui.input(|i| i.modifiers.shift);
        let ctrl = ui.input(|i| i.modifiers.command);
        let primary_down = ui.input(|i| i.pointer.primary_down());

        if response.secondary_clicked() && !matches!(self.tool, Tool::Idle) {
            self.cancel(project);
            return;
        }

        match &mut self.tool {
            Tool::Idle => {
                // Stisk tlačítka v obvodu: v režimu simulace hned, v režimu úprav až
                // druhým stiskem dvojkliku (první stisk tlačítko jen vybere / přesouvá).
                // [en] Pressing a button in the circuit: in simulation mode at once, in edit
                // mode only with the second press of a double-click (the first press only
                // selects / moves the button).
                let (pressed_now, now) = ui.input(|i| (i.pointer.primary_pressed(), i.time));
                if response.is_pointer_button_down_on()
                    && primary_down
                    && let Some(m) = mouse
                    && let Some(id) = component_at(&project.circuit, m)
                    && matches!(
                        project.circuit.component(id).map(|c| c.kind),
                        Some(ComponentKind::Button)
                    )
                {
                    let double = pressed_now
                        && self
                            .last_button_press
                            .is_some_and(|(last, t)| last == id && now - t < DOUBLE_CLICK_S);
                    if pressed_now {
                        self.last_button_press = Some((id, now));
                    }
                    if project.is_simulating() || double {
                        project.set_button(id, true);
                        project.selection.clear();
                        project.selection.comps.insert(id);
                        self.tool = Tool::Press(id);
                        return;
                    }
                }
                if response.drag_started_by(PointerButton::Primary) {
                    let origin = ui
                        .input(|i| i.pointer.press_origin())
                        .map(|p| cam.to_world(p));
                    if let Some(p0) = origin {
                        self.begin_drag(ui, project, p0, shift, ctrl);
                    }
                } else if response.double_clicked() && !project.is_simulating() {
                    // Režim úprav: přepínač se přepne dvojklikem.
                    // [en] Edit mode: a switch is toggled by a double-click.
                    if let Some(m) = mouse
                        && let Some(id) = component_at(&project.circuit, m)
                        && matches!(
                            project.circuit.component(id).map(|c| c.kind),
                            Some(ComponentKind::Switch { .. })
                        )
                    {
                        project.toggle_switch(id);
                    }
                } else if response.clicked()
                    && let Some(m) = mouse
                {
                    click(project, m, shift);
                }
            }
            Tool::Place { kind, rotation } => {
                if response.clicked()
                    && let Some(p) = mouse_pt
                {
                    let (kind, rotation) = (*kind, *rotation);
                    project.add_component_at(kind, p, rotation);
                    // Se Shift zůstane nástroj aktivní pro další kus.
                    // [en] With Shift the tool stays active for another piece.
                    if !shift {
                        self.tool = Tool::Idle;
                    }
                }
            }
            Tool::Grab {
                origin,
                start,
                delta,
                comps,
                wires,
                by_key,
                ..
            } => {
                if let Some(p) = mouse_pt {
                    let d = (p.x - start.x, p.y - start.y);
                    if d != *delta {
                        *delta = d;
                        project.circuit = origin.moved(comps, wires, d.0, d.1);
                        project.after_structure_change();
                    }
                }
                let done = if *by_key {
                    response.clicked()
                } else {
                    !primary_down
                };
                if done {
                    self.finish_grab(project);
                }
            }
            Tool::Wire {
                start,
                horizontal_first,
            } => {
                if let Some(p) = mouse_pt {
                    if p == *start {
                        *horizontal_first = None;
                    } else if horizontal_first.is_none() {
                        *horizontal_first = Some((p.x - start.x).abs() >= (p.y - start.y).abs());
                    }
                }
                if !primary_down {
                    let start = *start;
                    let hf = horizontal_first.unwrap_or(true);
                    self.tool = Tool::Idle;
                    if let Some(end) = mouse_pt
                        && end != start
                    {
                        project.checkpoint();
                        let (segs, _) = wire_route(&project.circuit, start, end, hf);
                        for (a, b) in segs {
                            project.circuit.add_wire(a, b);
                        }
                        project.circuit.merge_collinear();
                        project.after_structure_change();
                    }
                }
            }
            Tool::BoxSelect { start, additive } => {
                if !primary_down {
                    if let Some(m) = mouse {
                        let r = Rect::from_two_pos(*start, m);
                        if !*additive {
                            project.selection.clear();
                        }
                        for c in &project.circuit.components {
                            if r.intersects(symbols::hit_rect(c)) {
                                project.selection.comps.insert(c.id);
                            }
                        }
                        for w in &project.circuit.wires {
                            if r.contains(point(w.a)) && r.contains(point(w.b)) {
                                project.selection.wires.insert(w.id);
                            }
                        }
                    }
                    self.tool = Tool::Idle;
                }
            }
            Tool::Press(id) => {
                if !primary_down {
                    project.set_button(*id, false);
                    self.tool = Tool::Idle;
                }
            }
        }
    }

    /// Začátek tažení: z vývodu nebo vodiče kreslí vodič, ze součástky přesouvá, jinde vybírá obdélníkem.
    ///
    /// [en] Start of a drag: from a pin or wire it draws a wire, from a component it moves, elsewhere it box-selects.
    fn begin_drag(&mut self, ui: &Ui, project: &mut Project, p0: Pos2, shift: bool, ctrl: bool) {
        let circuit = &project.circuit;
        if !project.can_edit() {
            // Zamčené schéma: jen výběr obdélníkem; vodič ani přesun nejde.
            // [en] Locked schematic: box selection only; no wire and no move.
            if pin_at(circuit, p0).is_some()
                || component_at(circuit, p0).is_some()
                || ctrl
                || wire_at(circuit, p0).is_some()
            {
                self.show_locked_hint(ui);
            } else {
                self.tool = Tool::BoxSelect {
                    start: p0,
                    additive: shift,
                };
            }
            return;
        }
        if let Some(pin) = pin_at(circuit, p0) {
            self.tool = Tool::Wire {
                start: pin,
                horizontal_first: None,
            };
        } else if let Some(id) = component_at(circuit, p0) {
            if !project.selection.comps.contains(&id) {
                if !shift {
                    project.selection.clear();
                }
                project.selection.comps.insert(id);
            }
            self.start_grab(project, snap(p0), false, false);
        } else if ctrl || wire_at(circuit, p0).is_some() {
            // Vodič jde začít i z libovolného bodu vodiče, nebo s Ctrl kdekoli.
            // [en] A wire can also start from any point of a wire, or anywhere with Ctrl.
            self.tool = Tool::Wire {
                start: snap(p0),
                horizontal_first: None,
            };
        } else {
            self.tool = Tool::BoxSelect {
                start: p0,
                additive: shift,
            };
        }
    }

    /// Nakreslí vodiče, spojovací tečky a součástky s hodnotami ze simulace.
    ///
    /// [en] Draws wires, junction dots and components with values from the simulation.
    fn draw_circuit(&self, painter: &Painter, cam: Camera, project: &Project, mouse: Option<Pos2>) {
        let c = &project.circuit;
        let lw = cam.len(0.16).clamp(1.5, 4.0);
        for w in &c.wires {
            let (a, b) = (cam.to_screen(point(w.a)), cam.to_screen(point(w.b)));
            if project.selection.wires.contains(&w.id) {
                painter.line_segment(
                    [a, b],
                    Stroke::new(lw + 4.0, symbols::SELECTED.gamma_multiply(0.6)),
                );
            }
            // Zkrat napájení: síť výrazně podbarvená červeně.
            // [en] Power short: the net strongly highlighted in red.
            if project.sim.wire_in_power_short(w.id) {
                painter.line_segment([a, b], Stroke::new(lw + 7.0, POWER_SHORT));
            }
            painter.line_segment(
                [a, b],
                Stroke::new(lw, signal_color(project.sim.wire_value(w.id))),
            );
        }
        for j in c.junctions() {
            let v = c
                .wires
                .iter()
                .find(|w| w.contains(j))
                .map_or(logicflow_core::LogicValue::Z, |w| {
                    project.sim.wire_value(w.id)
                });
            painter.circle_filled(
                cam.to_screen(point(j)),
                cam.len(0.3).max(3.0),
                signal_color(v),
            );
        }
        let hovered = if matches!(self.tool, Tool::Idle) {
            mouse.and_then(|m| component_at(c, m))
        } else {
            None
        };
        for comp in &c.components {
            let values = Values {
                inputs: project.sim.inputs(comp.id),
                outputs: project.sim.output_net_values(comp.id),
                sub: symbols::SubInfo::of(project, comp.kind),
            };
            let look = Look {
                selected: project.selection.comps.contains(&comp.id),
                hovered: hovered == Some(comp.id),
                ghost: false,
            };
            symbols::draw(painter, cam, comp, &values, look);
        }
        // Zvýraznění vývodu pod myší – odtud jde táhnout vodič.
        // [en] Highlighting the pin under the mouse – a wire can be dragged from it.
        if matches!(self.tool, Tool::Idle)
            && let Some(m) = mouse
            && let Some(pin) = pin_at(c, m)
        {
            painter.circle_stroke(
                cam.to_screen(point(pin)),
                cam.len(0.45).max(5.0),
                Stroke::new(1.5, Color32::WHITE),
            );
        }
    }

    /// Nakreslí náhled nástroje (umisťovaná součástka, nový vodič, obdélník výběru).
    ///
    /// [en] Draws the tool preview (component being placed, new wire, selection box).
    fn draw_tool(
        &self,
        painter: &Painter,
        cam: Camera,
        mouse_pt: Option<Point>,
        project: &Project,
    ) {
        match &self.tool {
            Tool::Place { kind, rotation } => {
                if let Some(p) = mouse_pt {
                    let ghost = Component {
                        id: CompId(u32::MAX),
                        kind: *kind,
                        pos: p,
                        rotation: *rotation,
                        label: String::new(),
                        pressed: false,
                    };
                    let values = Values {
                        sub: symbols::SubInfo::of(project, *kind),
                        ..Default::default()
                    };
                    let look = Look {
                        ghost: true,
                        ..Default::default()
                    };
                    symbols::draw(painter, cam, &ghost, &values, look);
                }
            }
            Tool::Wire {
                start,
                horizontal_first,
            } => {
                if let Some(end) = mouse_pt {
                    let lw = cam.len(0.16).clamp(1.5, 4.0);
                    let (segs, warn) = wire_route(
                        &project.circuit,
                        *start,
                        end,
                        horizontal_first.unwrap_or(true),
                    );
                    for (a, b) in segs {
                        painter.line_segment(
                            [cam.to_screen(point(a)), cam.to_screen(point(b))],
                            Stroke::new(lw, Color32::WHITE.gamma_multiply(0.8)),
                        );
                    }
                    painter.circle_filled(cam.to_screen(point(end)), 3.5, Color32::WHITE);
                    // Místa, kde by se vodič napojil i jinde než na začátku a konci.
                    // [en] Places where the wire would connect other than at its start and end.
                    for p in warn {
                        painter.circle_stroke(
                            cam.to_screen(point(p)),
                            cam.len(0.5).max(6.0),
                            Stroke::new(2.0, WARN),
                        );
                    }
                }
            }
            Tool::BoxSelect { start, .. } => {
                if let Some(m) = mouse_pt.map(point) {
                    let r = Rect::from_two_pos(cam.to_screen(*start), cam.to_screen(m));
                    painter.rect_filled(r, 0.0, colors::ACCENT.gamma_multiply(0.15));
                    painter.rect_stroke(
                        r,
                        0.0,
                        Stroke::new(1.0, colors::ACCENT),
                        egui::StrokeKind::Middle,
                    );
                }
            }
            _ => {}
        }
    }

    /// Nabídka Přidat (Shift+A) u ukazatele myši.
    ///
    /// [en] The Add menu (Shift+A) at the mouse pointer.
    /// Zapamatuje si cíl kontextové nabídky podle místa kliknutí pravým tlačítkem.
    /// Nevybraná součástka nebo vodič se nejdřív vybere.
    ///
    /// [en] Remembers the context menu target according to the right-click position.
    /// A component or wire that is not selected gets selected first.
    fn open_menu_at(&mut self, project: &mut Project, m: Pos2) {
        self.add_menu = None;
        self.menu_point = snap(m);
        let total = |p: &Project| p.selection.comps.len() + p.selection.wires.len();
        self.menu_target = Some(if let Some(id) = component_at(&project.circuit, m) {
            if !project.selection.comps.contains(&id) {
                project.selection.clear();
                project.selection.comps.insert(id);
            }
            if total(project) > 1 {
                MenuTarget::Selection
            } else {
                MenuTarget::Component(id)
            }
        } else if let Some(w) = wire_at(&project.circuit, m) {
            if !project.selection.wires.contains(&w) {
                project.selection.clear();
                project.selection.wires.insert(w);
            }
            if total(project) > 1 {
                MenuTarget::Selection
            } else {
                MenuTarget::Wire(w)
            }
        } else {
            MenuTarget::Empty
        });
    }

    /// Obsah kontextové nabídky podle cíle. Položky volají stejné příkazy jako klávesy
    /// a hlavní nabídky; úpravy jsou v režimu simulace zašedlé s vysvětlením.
    ///
    /// [en] Context menu content according to the target. Items call the same commands
    /// as keys and main menus; edits are greyed out with an explanation in simulation mode.
    fn context_menu_ui(&mut self, ui: &mut Ui, rect: Rect, project: &mut Project, lang: Lang) {
        let Some(target) = self.menu_target else {
            ui.close();
            return;
        };
        ui.set_min_width(210.0);
        match target {
            MenuTarget::Component(id) => {
                let Some(comp) = project.circuit.component(id).cloned() else {
                    ui.close();
                    return;
                };
                let title = if comp.label.is_empty() {
                    catalog::kind_name(&comp.kind, lang)
                } else {
                    format!(
                        "{}  ·  {}",
                        comp.label,
                        catalog::kind_name(&comp.kind, lang)
                    )
                };
                ui.label(egui::RichText::new(title).strong());
                ui.separator();
                self.kind_items(ui, project, lang, id, comp.kind);
                // Popisek přímo v nabídce.
                // [en] The label right in the menu.
                ui.horizontal(|ui| {
                    ui.label(tr(lang, T::Label));
                    let mut label = comp.label.clone();
                    let r = ui.add_enabled(
                        project.can_edit(),
                        egui::TextEdit::singleline(&mut label).desired_width(110.0),
                    );
                    if r.gained_focus() {
                        project.checkpoint();
                    }
                    if r.changed()
                        && let Some(c) = project.circuit.component_mut(id)
                    {
                        c.label = label;
                        project.touch();
                    }
                });
                ui.separator();
                for cmd in [
                    Command::Rotate,
                    Command::Duplicate,
                    Command::Grab,
                    Command::Copy,
                    Command::Cut,
                    Command::Delete,
                ] {
                    self.menu_command(ui, cmd, rect, project, lang);
                }
            }
            MenuTarget::Wire(w) => {
                ui.label(egui::RichText::new(tr(lang, T::Wire)).strong());
                ui.separator();
                if ui.button(tr(lang, T::CtxSelectNet)).clicked() {
                    project.selection.clear();
                    project.selection.wires = project.net_wires(w).into_iter().collect();
                    ui.close();
                }
                if edit_button(ui, project, lang, tr(lang, T::CtxDeleteSegment)) {
                    project.selection.clear();
                    project.delete_wires(&[w]);
                    ui.close();
                }
                if edit_button(ui, project, lang, tr(lang, T::CtxDeleteNet)) {
                    let net = project.net_wires(w);
                    project.selection.clear();
                    project.delete_wires(&net);
                    ui.close();
                }
            }
            MenuTarget::Selection => {
                ui.label(
                    egui::RichText::new(format!(
                        "{}: {} · {}: {}",
                        tr(lang, T::Components),
                        project.selection.comps.len(),
                        tr(lang, T::Wires),
                        project.selection.wires.len()
                    ))
                    .strong(),
                );
                ui.separator();
                for cmd in [
                    Command::Rotate,
                    Command::Duplicate,
                    Command::Grab,
                    Command::Copy,
                    Command::Cut,
                    Command::Delete,
                    Command::SelectNone,
                ] {
                    self.menu_command(ui, cmd, rect, project, lang);
                }
            }
            MenuTarget::Empty => {
                // Přidat součástku přímo na místo kliknutí (stejné kategorie jako Shift+A).
                // [en] Add a component right at the clicked place (same categories as Shift+A).
                let mut chosen = None;
                ui.add_enabled_ui(project.can_edit(), |ui| {
                    ui.menu_button(tr(lang, T::CtxAddHere), |ui| {
                        for cat in catalog::catalog(lang) {
                            ui.menu_button(tr(lang, cat.title), |ui| {
                                for (name, kind) in cat.items {
                                    if ui.button(name).clicked() {
                                        chosen = Some(kind);
                                    }
                                }
                            });
                        }
                        let subs = catalog::subcircuit_items(project);
                        if !subs.is_empty() {
                            ui.menu_button(tr(lang, T::CatSubcircuits), |ui| {
                                for (name, kind) in subs {
                                    if ui.button(name).clicked() {
                                        chosen = Some(kind);
                                    }
                                }
                            });
                        }
                    });
                })
                .response
                .on_disabled_hover_text(tr(lang, project.locked_hint()));
                if let Some(kind) = chosen {
                    project.add_component_at(kind, self.menu_point, Rotation::R0);
                    ui.close();
                }
                self.menu_command(ui, Command::Paste, rect, project, lang);
                ui.separator();
                for cmd in [Command::SelectAll, Command::FitView] {
                    self.menu_command(ui, cmd, rect, project, lang);
                }
                if project.sheet.is_some() {
                    let text = commands::with_shortcut(
                        tr(lang, T::CmdBackToParent),
                        Command::Subcircuit,
                        lang,
                    );
                    if ui.button(text).clicked() {
                        project.leave_subcircuit();
                        ui.close();
                    }
                }
                ui.separator();
                // Ovládání simulace – užitečné hlavně pro dotyk a projektor.
                // [en] Simulation controls – useful mainly for touch and a projector.
                let first = if project.is_simulating() {
                    Command::RunPause
                } else {
                    Command::Start
                };
                for cmd in [
                    first,
                    Command::Step,
                    Command::StepCycle,
                    Command::Stop,
                    Command::ResetSim,
                ] {
                    self.menu_command(ui, cmd, rect, project, lang);
                }
            }
        }
    }

    /// Položky podle druhu součástky (přepnout, frekvence, počet vstupů, bity…).
    ///
    /// [en] Items according to the component kind (toggle, frequency, inputs, bits…).
    fn kind_items(
        &mut self,
        ui: &mut Ui,
        project: &mut Project,
        lang: Lang,
        id: CompId,
        kind: ComponentKind,
    ) {
        let can_edit = project.can_edit();
        let hint = project.locked_hint();
        let mut new_kind = kind;
        match kind {
            ComponentKind::Switch { .. } => {
                if ui.button(tr(lang, T::CtxToggle)).clicked() {
                    project.toggle_switch(id);
                    ui.close();
                }
            }
            ComponentKind::Subcircuit { .. } => {
                if commands::menu_item(ui, Command::Subcircuit, lang, project) {
                    project.toggle_subcircuit(lang);
                    ui.close();
                }
            }
            // Frekvence hodin jde měnit i za běhu simulace.
            // [en] The clock frequency can be changed while simulating too.
            ComponentKind::Clock { period_ms } => {
                ui.menu_button(tr(lang, T::Frequency), |ui| {
                    for hz in crate::project::CLOCK_PRESETS_HZ {
                        let p = crate::project::period_for_hz(hz);
                        if ui
                            .add(egui::Button::selectable(
                                period_ms == p,
                                symbols::format_hz(hz),
                            ))
                            .clicked()
                        {
                            // Bez skoku fáze – hodiny navážou plynule.
                            // [en] Without a phase jump – the clock continues smoothly.
                            project.set_clock_period(id, p, true);
                            ui.close();
                        }
                    }
                });
            }
            ComponentKind::Gate { gate, inputs } if !gate.is_unary() => {
                edit_submenu(ui, (can_edit, hint), lang, tr(lang, T::InputCount), |ui| {
                    for n in ComponentKind::MIN_INPUTS..=ComponentKind::MAX_INPUTS {
                        if ui
                            .add(egui::Button::selectable(inputs == n, n.to_string()))
                            .clicked()
                        {
                            new_kind = ComponentKind::Gate { gate, inputs: n };
                        }
                    }
                });
            }
            ComponentKind::SevenSegment { pins } | ComponentKind::HexDisplay { pins } => {
                edit_submenu(
                    ui,
                    (can_edit, hint),
                    lang,
                    tr(lang, T::PinSideLabel),
                    |ui| {
                        for p in logicflow_core::PinSide::ALL {
                            let name = tr(
                                lang,
                                match p {
                                    logicflow_core::PinSide::Left => T::SideLeft,
                                    logicflow_core::PinSide::Right => T::SideRight,
                                    logicflow_core::PinSide::Top => T::SideTop,
                                    logicflow_core::PinSide::Bottom => T::SideBottom,
                                },
                            );
                            if ui.add(egui::Button::selectable(pins == p, name)).clicked() {
                                new_kind = match kind {
                                    ComponentKind::SevenSegment { .. } => {
                                        ComponentKind::SevenSegment { pins: p }
                                    }
                                    _ => ComponentKind::HexDisplay { pins: p },
                                };
                            }
                        }
                    },
                );
            }
            ComponentKind::Led { color } | ComponentKind::LedAk { color } => {
                edit_submenu(ui, can_edit, lang, tr(lang, T::LedColorLabel), |ui| {
                    for c in logicflow_core::LedColor::ALL {
                        let name = catalog::led_color_name(c, lang);
                        if ui.add(egui::Button::selectable(color == c, name)).clicked() {
                            new_kind = kind.with_led_color(c);
                        }
                    }
                });
                let two = matches!(kind, ComponentKind::LedAk { .. });
                edit_submenu(ui, can_edit, lang, tr(lang, T::PinSideLabel), |ui| {
                    for (t, label) in [(false, T::LedPinsOne), (true, T::LedPinsTwo)] {
                        if ui
                            .add(egui::Button::selectable(two == t, tr(lang, label)))
                            .clicked()
                        {
                            new_kind = kind.with_led_two_pins(t);
                        }
                    }
                });
            }
            ComponentKind::Block { block } => {
                // Tab: vnitřní zapojení bloku z hradel (jen pro čtení).
                // [en] Tab: the block's gate-level internal circuit (read-only).
                if commands::menu_item(ui, Command::Subcircuit, lang, project) {
                    project.toggle_subcircuit(lang);
                    ui.close();
                }
                if let Some(n) = block.param() {
                    let (lo, hi) = block.param_range();
                    let title = if matches!(
                        block,
                        logicflow_core::BlockKind::Mux { .. }
                            | logicflow_core::BlockKind::Decoder { .. }
                    ) {
                        T::SelectBits
                    } else {
                        T::Bits
                    };
                    edit_submenu(ui, (can_edit, hint), lang, tr(lang, title), |ui| {
                        for m in lo..=hi {
                            if ui
                                .add(egui::Button::selectable(n == m, m.to_string()))
                                .clicked()
                            {
                                new_kind = ComponentKind::Block {
                                    block: block.with_param(m),
                                };
                            }
                        }
                    });
                }
                if let logicflow_core::BlockKind::Counter { bits, down } = block {
                    edit_submenu(
                        ui,
                        (can_edit, hint),
                        lang,
                        tr(lang, T::CountDirection),
                        |ui| {
                            for (d, t) in [(false, T::CountUp), (true, T::CountDown)] {
                                if ui
                                    .add(egui::Button::selectable(down == d, tr(lang, t)))
                                    .clicked()
                                {
                                    new_kind = ComponentKind::Block {
                                        block: logicflow_core::BlockKind::Counter { bits, down: d },
                                    };
                                }
                            }
                        },
                    );
                }
            }
            _ => {}
        }
        if new_kind != kind {
            project.set_kind(id, new_kind);
            ui.close();
        }
    }

    /// Položka nabídky pro příkaz; po kliknutí ho provede na místě kliknutí pravým tlačítkem.
    ///
    /// [en] Menu item for a command; when clicked, it runs it at the right-clicked place.
    fn menu_command(
        &mut self,
        ui: &mut Ui,
        cmd: Command,
        rect: Rect,
        project: &mut Project,
        lang: Lang,
    ) {
        if commands::menu_item(ui, cmd, lang, project) {
            let at = Some(self.menu_point);
            self.run_command(cmd, ui, rect, project, at, lang);
            ui.close();
        }
    }

    /// Nabídka Přidat (Shift+A) u ukazatele myši.
    ///
    /// [en] The Add menu (Shift+A) at the mouse pointer.
    fn add_menu_ui(&mut self, ui: &Ui, lang: Lang, subs: Vec<(String, ComponentKind)>) {
        let Some(pos) = self.add_menu else { return };
        let mut chosen = None;
        let area = egui::Area::new(ui.scope_id().with("add_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::menu(ui.style()).show(ui, |ui| {
                    ui.set_min_width(170.0);
                    ui.label(egui::RichText::new(tr(lang, T::Add)).strong());
                    ui.separator();
                    // Kategorie jako podnabídky (jako Shift+A v Blenderu).
                    // [en] Categories as submenus (like Shift+A in Blender).
                    for cat in catalog::catalog(lang) {
                        ui.menu_button(tr(lang, cat.title), |ui| {
                            for (name, kind) in cat.items {
                                if ui.button(name).clicked() {
                                    chosen = Some(kind);
                                    ui.close();
                                }
                            }
                        });
                    }
                    if !subs.is_empty() {
                        ui.menu_button(tr(lang, T::CatSubcircuits), |ui| {
                            for (name, kind) in subs {
                                if ui.button(name).clicked() {
                                    chosen = Some(kind);
                                    ui.close();
                                }
                            }
                        });
                    }
                });
            });
        if let Some(kind) = chosen {
            self.tool = Tool::Place {
                kind,
                rotation: Rotation::R0,
            };
            self.add_menu = None;
        } else if area.response.clicked_elsewhere() {
            self.add_menu = None;
        }
    }
}

/// Tlačítko úpravy v nabídce: v režimu simulace zašedlé s vysvětlením. Vrací `true`
/// po kliknutí.
///
/// [en] Edit button in a menu: greyed out with an explanation in simulation mode.
/// Returns `true` when clicked.
fn edit_button(ui: &mut Ui, project: &Project, lang: Lang, text: &str) -> bool {
    ui.add_enabled(project.can_edit(), egui::Button::new(text))
        .on_disabled_hover_text(tr(lang, project.locked_hint()))
        .clicked()
}

/// Podnabídka úprav (počet vstupů, bity…): v režimu simulace zašedlá s vysvětlením.
///
/// [en] Edit submenu (inputs, bits…): greyed out with an explanation in simulation mode.
/// `edit` je (lze upravovat, vysvětlení při zamčení).
///
/// [en] `edit` is (can be edited, explanation when locked).
fn edit_submenu(ui: &mut Ui, edit: (bool, T), lang: Lang, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.add_enabled_ui(edit.0, |ui| {
        ui.menu_button(title, add);
    })
    .response
    .on_disabled_hover_text(tr(lang, edit.1));
}

/// Klíč pro předání stisku Tab mezi snímky.
///
/// [en] Key for passing a Tab press between frames.
fn tab_id() -> egui::Id {
    egui::Id::unique("logicflow-tab")
}

/// Zaznamená stisk klávesy Tab (zachycený dřív, než ho zpracuje egui).
///
/// [en] Records a Tab key press (captured before egui handles it).
pub fn request_tab(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(tab_id(), true));
}

/// Vrátí a smaže zaznamenaný stisk Tab.
///
/// [en] Returns and clears the recorded Tab press.
pub fn take_tab(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(tab_id()))
        .unwrap_or(false)
}

/// Kliknutí: vybere součástku nebo vodič (se Shift přidá/odebere), přepne přepínač.
///
/// [en] Click: selects a component or wire (with Shift adds/removes), toggles a switch.
fn click(project: &mut Project, m: Pos2, shift: bool) {
    if let Some(id) = component_at(&project.circuit, m) {
        if shift {
            if !project.selection.comps.remove(&id) {
                project.selection.comps.insert(id);
            }
        } else {
            project.selection.clear();
            project.selection.comps.insert(id);
        }
        // V režimu simulace přepíná přepínač jedno kliknutí, v režimu úprav dvojklik
        // (jedno kliknutí jen vybírá).
        // [en] In simulation mode a single click toggles a switch, in edit mode a
        // double-click (a single click only selects).
        if project.is_simulating()
            && matches!(
                project.circuit.component(id).map(|c| c.kind),
                Some(ComponentKind::Switch { .. })
            )
        {
            project.toggle_switch(id);
        }
    } else if let Some(w) = wire_at(&project.circuit, m) {
        if !shift {
            project.selection.clear();
        }
        project.selection.wires.insert(w);
    } else if !shift {
        project.selection.clear();
    }
}

/// Barva varování: vodič by se napojil i mimo své konce.
///
/// [en] Warning colour: the wire would also connect outside its ends.
const WARN: Color32 = Color32::from_rgb(0xff, 0x8c, 0x1a);

/// Cesta nového vodiče a místa, kde by se nechtěně napojil (mimo začátek a konec).
/// Má-li zvolené lomení taková místa a opačné ne (nebo méně), použije se opačné.
///
/// [en] Route of a new wire and the places where it would connect unintentionally
/// (outside its start and end). If the chosen bend has such places and the
/// opposite one does not (or fewer), the opposite one is used.
fn wire_route(
    c: &Circuit,
    start: Point,
    end: Point,
    horizontal_first: bool,
) -> (Vec<(Point, Point)>, Vec<Point>) {
    let route = |hf: bool| {
        let segs = wire_path(start, end, hf);
        let warn: Vec<Point> = c
            .wire_contacts(&segs)
            .into_iter()
            .filter(|p| *p != start && *p != end)
            .collect();
        (segs, warn)
    };
    let first = route(horizontal_first);
    if first.1.is_empty() {
        return first;
    }
    let other = route(!horizontal_first);
    if other.1.len() < first.1.len() {
        other
    } else {
        first
    }
}

/// Lomená cesta vodiče ze `start` do `end` (nejvýše dvě úsečky).
///
/// [en] Bent wire route from `start` to `end` (at most two segments).
fn wire_path(start: Point, end: Point, horizontal_first: bool) -> Vec<(Point, Point)> {
    let corner = if horizontal_first {
        Point::new(end.x, start.y)
    } else {
        Point::new(start.x, end.y)
    };
    [(start, corner), (corner, end)]
        .into_iter()
        .filter(|(a, b)| a != b)
        .collect()
}

/// Součástka pod bodem (nejvyšší = poslední v seznamu).
///
/// [en] Component under the point (the topmost = last in the list).
fn component_at(c: &Circuit, p: Pos2) -> Option<CompId> {
    c.components
        .iter()
        .rev()
        .find(|k| symbols::hit_rect(k).contains(p))
        .map(|k| k.id)
}

/// Vývod pod bodem (nejbližší, do vzdálenosti půl bodu mřížky).
///
/// [en] Pin under the point (the nearest one, within half a grid point).
fn pin_at(c: &Circuit, p: Pos2) -> Option<Point> {
    c.components
        .iter()
        .flat_map(|k| k.pins())
        .filter(|q| point(*q).distance(p) < 0.5)
        .min_by(|a, b| point(*a).distance(p).total_cmp(&point(*b).distance(p)))
}

/// Vodič pod bodem.
///
/// [en] Wire under the point.
fn wire_at(c: &Circuit, p: Pos2) -> Option<WireId> {
    c.wires
        .iter()
        .find(|w| dist_to_segment(p, point(w.a), point(w.b)) < 0.35)
        .map(|w| w.id)
}

/// Mřížka plátna (jemná a hrubá, při malém zvětšení jen hrubá).
///
/// [en] Canvas grid (fine and coarse, only coarse when zoomed out).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_route_avoids_accidental_contacts() {
        let mut c = Circuit::default();
        // Cizí vodič končí v (5,0) – přímo na cestě „nejdřív vodorovně“.
        // [en] A foreign wire ends at (5,0) – right on the “horizontal first” route.
        c.add_wire(Point::new(5, 0), Point::new(5, -3));
        let (segs, warn) = wire_route(&c, Point::new(0, 0), Point::new(10, 4), true);
        assert!(warn.is_empty());
        assert_eq!(
            segs[0],
            (Point::new(0, 0), Point::new(0, 4)),
            "lomí se opačně"
        );
        // Když se dotknou obě varianty, varování zůstane vidět.
        // [en] When both variants touch something, the warning stays visible.
        c.add_wire(Point::new(0, 2), Point::new(-3, 2));
        let (_, warn) = wire_route(&c, Point::new(0, 0), Point::new(10, 4), true);
        assert_eq!(warn, [Point::new(5, 0)]);
    }

    #[test]
    fn wire_path_shapes() {
        let a = Point::new(0, 0);
        assert_eq!(
            wire_path(a, Point::new(5, 0), true),
            vec![(a, Point::new(5, 0))]
        );
        assert_eq!(
            wire_path(a, Point::new(5, 3), true),
            vec![(a, Point::new(5, 0)), (Point::new(5, 0), Point::new(5, 3))]
        );
        assert_eq!(
            wire_path(a, Point::new(5, 3), false),
            vec![(a, Point::new(0, 3)), (Point::new(0, 3), Point::new(5, 3))]
        );
    }

    #[test]
    fn picking() {
        let c = crate::examples::and_or();
        // Výstup přepínače A je na (2, 0).
        // [en] The output of switch A is at (2, 0).
        assert_eq!(pin_at(&c, pos2(2.2, 0.1)), Some(Point::new(2, 0)));
        assert!(component_at(&c, pos2(0.0, 0.0)).is_some());
        assert!(wire_at(&c, pos2(4.0, 0.1)).is_some());
        assert!(component_at(&c, pos2(30.0, 30.0)).is_none());
    }
}
