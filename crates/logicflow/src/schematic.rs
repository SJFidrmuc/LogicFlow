//! Editor schématu: nekonečné plátno s mřížkou, přidávání a úpravy součástek,
//! kreslení vodičů. Ovládání vychází z Blenderu (Shift+A, G, R, X, Shift+D, A).

use crate::catalog;
use crate::i18n::{Lang, T, tr};
use crate::project::Project;
use crate::symbols::{self, Camera, Look, UNIT, Values, dist_to_segment, point, snap};
use crate::theme::{colors, signal_color};
use egui::{
    Align2, Color32, FontId, Key, Modifiers, Painter, PointerButton, Pos2, Rect, Sense, Stroke, Ui,
    Vec2, pos2, vec2,
};
use logicflow_core::{Circuit, CompId, Component, ComponentKind, Point, Rotation, WireId};

const MIN_ZOOM: f32 = 0.15;
const MAX_ZOOM: f32 = 6.0;

#[derive(Clone, Debug)]
enum Tool {
    Idle,
    /// Umisťování nové součástky (náhled sleduje myš).
    Place {
        kind: ComponentKind,
        rotation: Rotation,
    },
    /// Přesun výběru. `origin` je stav před přesunem.
    Grab {
        origin: Box<Circuit>,
        start: Point,
        delta: (i32, i32),
        comps: Vec<CompId>,
        wires: Vec<WireId>,
        by_key: bool,
        undo_pushed: bool,
        was_modified: bool,
    },
    /// Kreslení vodiče z bodu `start`.
    Wire {
        start: Point,
        horizontal_first: Option<bool>,
    },
    /// Výběr obdélníkem (ve světových souřadnicích).
    BoxSelect {
        start: Pos2,
        additive: bool,
    },
    /// Držené tlačítko.
    Press(CompId),
}

/// Kamera a stav nástroje jednoho editoru schématu (každá oblast má vlastní).
#[derive(Clone, Debug)]
pub struct SchematicView {
    pub offset: Vec2,
    pub zoom: f32,
    pub needs_fit: bool,
    tool: Tool,
    add_menu: Option<Pos2>,
    /// List, který byl zobrazený naposledy (při změně se pohled přizpůsobí).
    shown_sheet: Option<Option<u32>>,
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
        }
    }
}

impl SchematicView {
    fn camera(&self, rect: Rect) -> Camera {
        Camera {
            origin: rect.center() + self.offset,
            scale: UNIT * self.zoom,
        }
    }

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

    pub fn ui(&mut self, ui: &mut Ui, project: &mut Project, lang: Lang) {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        if self.shown_sheet != Some(project.sheet) {
            if self.shown_sheet.is_some() {
                self.tool = Tool::Idle;
                self.add_menu = None;
            }
            self.shown_sheet = Some(project.sheet);
            self.needs_fit = true;
        }
        if self.needs_fit && rect.width() > 10.0 {
            self.fit(rect, &project.circuit);
        }

        // Posun prostředním tlačítkem (nebo tažením pravým).
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

        // Součástka vybraná v knihovně se umístí do editoru, nad kterým je myš.
        if response.hovered()
            && let Some(kind) = project.pending_place.take()
        {
            self.tool = Tool::Place {
                kind,
                rotation: Rotation::R0,
            };
        }

        if response.hovered() && !ui.ctx().egui_wants_keyboard_input() {
            self.keyboard(ui, rect, project, mouse_pt);
        }
        self.pointer(ui, &response, cam, project, mouse, mouse_pt);

        // --- Kreslení ---
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
        if project.sim.is_oscillating() {
            painter.text(
                rect.left_top() + vec2(10.0, 10.0),
                Align2::LEFT_TOP,
                tr(lang, T::Oscillating),
                FontId::proportional(13.0),
                Color32::from_rgb(0xff, 0x70, 0x60),
            );
        }
        let hint = match self.tool {
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

    fn keyboard(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        project: &mut Project,
        mouse_pt: Option<Point>,
    ) {
        // Konkrétnější zkratky (se Shift/Alt) se musí zkontrolovat dřív.
        let (add, dup, deselect, all, grab, rotate, delete, esc, home) = ui.input_mut(|i| {
            (
                i.consume_key(Modifiers::SHIFT, Key::A),
                i.consume_key(Modifiers::SHIFT, Key::D),
                i.consume_key(Modifiers::ALT, Key::A),
                i.consume_key(Modifiers::NONE, Key::A),
                i.consume_key(Modifiers::NONE, Key::G),
                i.consume_key(Modifiers::NONE, Key::R),
                i.consume_key(Modifiers::NONE, Key::X)
                    || i.consume_key(Modifiers::NONE, Key::Delete),
                i.consume_key(Modifiers::NONE, Key::Escape),
                i.consume_key(Modifiers::NONE, Key::Home),
            )
        });
        let tab = take_tab(ui.ctx());
        // Tab: do vybraného podobvodu a zpět (jako skupiny uzlů v Blenderu).
        if tab && matches!(self.tool, Tool::Idle) {
            let target = project.selection.single_comp().and_then(|id| {
                match project.circuit.component(id).map(|c| c.kind) {
                    Some(ComponentKind::Subcircuit { id, .. }) => Some(id),
                    _ => None,
                }
            });
            match target {
                Some(id) => project.enter_subcircuit(id),
                None => {
                    project.leave_subcircuit();
                }
            }
            return;
        }
        let busy = !matches!(self.tool, Tool::Idle);
        if esc {
            if self.add_menu.take().is_some() {
                // jen zavřít nabídku
            } else if busy {
                self.cancel(project);
            } else {
                project.selection.clear();
            }
        }
        if home {
            self.fit(rect, &project.circuit);
        }
        if add && !busy {
            self.add_menu = ui.input(|i| i.pointer.hover_pos());
        }
        if rotate {
            match &mut self.tool {
                Tool::Place { rotation, .. } => *rotation = rotation.next(),
                Tool::Idle if !project.selection.comps.is_empty() => {
                    project.checkpoint();
                    for id in project.selection.comp_vec() {
                        if let Some(c) = project.circuit.component_mut(id) {
                            c.rotation = c.rotation.next();
                        }
                    }
                    project.after_structure_change();
                }
                _ => {}
            }
        }
        if busy {
            return;
        }
        if all {
            project.selection.comps = project.circuit.components.iter().map(|c| c.id).collect();
            project.selection.wires = project.circuit.wires.iter().map(|w| w.id).collect();
        }
        if deselect {
            project.selection.clear();
        }
        if delete && !project.selection.is_empty() {
            project.checkpoint();
            for id in project.selection.comp_vec() {
                project.circuit.remove_component(id);
            }
            for id in project.selection.wire_vec() {
                project.circuit.remove_wire(id);
            }
            project.selection.clear();
            project.after_structure_change();
        }
        if let Some(start) = mouse_pt {
            if dup && !project.selection.is_empty() {
                let before = project.circuit.clone();
                let (comps, wires) = project
                    .circuit
                    .duplicate(&project.selection.comp_vec(), &project.selection.wire_vec());
                project.push_undo(before);
                project.selection.comps = comps.iter().copied().collect();
                project.selection.wires = wires.iter().copied().collect();
                project.after_structure_change();
                self.start_grab(project, start, true, true);
            } else if grab && !project.selection.is_empty() {
                self.start_grab(project, start, true, false);
            }
        }
    }

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
                // Stisk tlačítka v obvodu.
                if response.is_pointer_button_down_on()
                    && primary_down
                    && let Some(m) = mouse
                    && let Some(id) = component_at(&project.circuit, m)
                    && matches!(
                        project.circuit.component(id).map(|c| c.kind),
                        Some(ComponentKind::Button)
                    )
                {
                    project.set_button(id, true);
                    project.selection.clear();
                    project.selection.comps.insert(id);
                    self.tool = Tool::Press(id);
                    return;
                }
                if response.drag_started_by(PointerButton::Primary) {
                    let origin = ui
                        .input(|i| i.pointer.press_origin())
                        .map(|p| cam.to_world(p));
                    if let Some(p0) = origin {
                        self.begin_drag(project, p0, shift, ctrl);
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
                    project.checkpoint();
                    let id = project.circuit.add_component(kind, p);
                    if let Some(c) = project.circuit.component_mut(id) {
                        c.rotation = rotation;
                    }
                    project.selection.clear();
                    project.selection.comps.insert(id);
                    project.after_structure_change();
                    // Se Shift zůstane nástroj aktivní pro další kus.
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

    fn begin_drag(&mut self, project: &mut Project, p0: Pos2, shift: bool, ctrl: bool) {
        let circuit = &project.circuit;
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

fn tab_id() -> egui::Id {
    egui::Id::unique("logicflow-tab")
}

/// Zaznamená stisk klávesy Tab (zachycený dřív, než ho zpracuje egui).
pub fn request_tab(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(tab_id(), true));
}

pub fn take_tab(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(tab_id()))
        .unwrap_or(false)
}

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
        if matches!(
            project.circuit.component(id).map(|c| c.kind),
            Some(ComponentKind::Switch { .. })
        ) {
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
const WARN: Color32 = Color32::from_rgb(0xff, 0x8c, 0x1a);

/// Cesta nového vodiče a místa, kde by se nechtěně napojil (mimo začátek a konec).
/// Má-li zvolené lomení taková místa a opačné ne (nebo méně), použije se opačné.
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
fn component_at(c: &Circuit, p: Pos2) -> Option<CompId> {
    c.components
        .iter()
        .rev()
        .find(|k| symbols::hit_rect(k).contains(p))
        .map(|k| k.id)
}

fn pin_at(c: &Circuit, p: Pos2) -> Option<Point> {
    c.components
        .iter()
        .flat_map(|k| k.pins())
        .filter(|q| point(*q).distance(p) < 0.5)
        .min_by(|a, b| point(*a).distance(p).total_cmp(&point(*b).distance(p)))
}

fn wire_at(c: &Circuit, p: Pos2) -> Option<WireId> {
    c.wires
        .iter()
        .find(|w| dist_to_segment(p, point(w.a), point(w.b)) < 0.35)
        .map(|w| w.id)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_route_avoids_accidental_contacts() {
        let mut c = Circuit::default();
        // Cizí vodič končí v (5,0) – přímo na cestě „nejdřív vodorovně“.
        c.add_wire(Point::new(5, 0), Point::new(5, -3));
        let (segs, warn) = wire_route(&c, Point::new(0, 0), Point::new(10, 4), true);
        assert!(warn.is_empty());
        assert_eq!(
            segs[0],
            (Point::new(0, 0), Point::new(0, 4)),
            "lomí se opačně"
        );
        // Když se dotknou obě varianty, varování zůstane vidět.
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
        assert_eq!(pin_at(&c, pos2(2.2, 0.1)), Some(Point::new(2, 0)));
        assert!(component_at(&c, pos2(0.0, 0.0)).is_some());
        assert!(wire_at(&c, pos2(4.0, 0.1)).is_some());
        assert!(component_at(&c, pos2(30.0, 30.0)).is_none());
    }
}
