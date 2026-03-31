use crate::tools::CanvasTool;
use crate::settings::EngineSettings;
use crate::graph::{SceneManager, IdAllocator, StrokeId};
use crate::command::{Command, AffineCommand};
use crate::geometry::tessellator::Extruder;
use crate::geometry::{Point, VectorElement};
use crate::math::Vertex; 

#[derive(PartialEq)]
enum SelectState { Idle, Translating, DraggingPivot, Rotating, Scaling(usize) }

const ROTATE_CURSOR: &str = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23ffffff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8'/%3E%3Cpath d='M3 3v5h5'/%3E%3C/svg%3E\") 12 12, auto";

pub struct SelectTool {
    state: SelectState, lasso_points: Vec<Point>,
    start_x: f32, start_y: f32, total_dx: f32, total_dy: f32, total_sx: f32, total_sy: f32, total_rot: f32,
    active_pivot: (f32, f32), custom_pivot: Option<(f32, f32)>, original_elements: Vec<(StrokeId, VectorElement)>, current_cursor: &'static str,
}

impl SelectTool { pub fn new() -> Self { Self { state: SelectState::Idle, lasso_points: Vec::with_capacity(256), start_x: 0.0, start_y: 0.0, total_dx: 0.0, total_dy: 0.0, total_sx: 1.0, total_sy: 1.0, total_rot: 0.0, active_pivot: (0.0, 0.0), custom_pivot: None, original_elements: Vec::new(), current_cursor: "default" } } }

impl CanvasTool for SelectTool {
    fn get_cursor(&self) -> &'static str { self.current_cursor }
    fn get_custom_pivot(&self) -> Option<(f32, f32)> { self.custom_pivot }

    fn on_pointer_hover(&mut self, x: f32, y: f32, _constrain: bool, _center: bool, scene: &SceneManager) {
        if self.state != SelectState::Idle { return; } 
        let mut cursor = "default";
        if let Some(aabb) = scene.get_selection_aabb() {
            let cx = aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0; let cy = aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0;
            let (px, py) = self.custom_pivot.unwrap_or((cx, cy));
            if (x - px).abs() < 10.0 && (y - py).abs() < 10.0 { self.current_cursor = "crosshair"; return; }
            let hs = 8.0; 
            let coords = [ ("nwse-resize", aabb.min_x, aabb.min_y), ("nesw-resize", aabb.max_x, aabb.min_y), ("nwse-resize", aabb.max_x, aabb.max_y), ("nesw-resize", aabb.min_x, aabb.max_y), ("ns-resize", cx, aabb.min_y), ("ew-resize", aabb.max_x, cy), ("ns-resize", cx, aabb.max_y), ("ew-resize", aabb.min_x, cy) ];
            for (c, hx, hy) in coords { if (x - hx).abs() <= hs && (y - hy).abs() <= hs { cursor = c; break; } }
            if cursor == "default" {
                let rs = 20.0; let corners = [(aabb.min_x, aabb.min_y), (aabb.max_x, aabb.min_y), (aabb.max_x, aabb.max_y), (aabb.min_x, aabb.max_y)];
                for (cx, cy) in corners { if (x - cx).abs() <= rs && (y - cy).abs() <= rs { cursor = ROTATE_CURSOR; break; } }
            }
            if cursor == "default" {
                if x >= aabb.min_x && x <= aabb.max_x && y >= aabb.min_y && y <= aabb.max_y { cursor = "move"; } 
                else if scene.hit_test(x, y).is_some() { cursor = "pointer"; }
            }
        } else if scene.hit_test(x, y).is_some() { cursor = "pointer"; }
        self.current_cursor = cursor;
    }

    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, center: bool, _settings: EngineSettings, scene: &mut SceneManager, _id_allocator: &mut IdAllocator) {
        self.start_x = x; self.start_y = y; self.total_dx = 0.0; self.total_dy = 0.0; self.total_sx = 1.0; self.total_sy = 1.0; self.total_rot = 0.0; self.state = SelectState::Idle;
        self.original_elements.clear();
        if let Some((_, layer)) = scene.get_active_art_layer() {
            for id in &scene.selected_strokes {
                if let Some(el) = layer.vector_elements.get(id) { self.original_elements.push((*id, el.clone())); }
            }
        }

        if let Some(aabb) = scene.get_selection_aabb() {
            let cx = aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0; let cy = aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0;
            let (px, py) = self.custom_pivot.unwrap_or((cx, cy));
            if (x - px).abs() < 10.0 && (y - py).abs() < 10.0 { self.state = SelectState::DraggingPivot; self.current_cursor = "crosshair"; return; }
            let hs = 8.0;
            let coords = [ (aabb.min_x, aabb.min_y, aabb.max_x, aabb.max_y), (aabb.max_x, aabb.min_y, aabb.min_x, aabb.max_y), (aabb.max_x, aabb.max_y, aabb.min_x, aabb.min_y), (aabb.min_x, aabb.max_y, aabb.max_x, aabb.min_y), (cx, aabb.min_y, cx, aabb.max_y), (aabb.max_x, cy, aabb.min_x, cy), (cx, aabb.max_y, cx, aabb.min_y), (aabb.min_x, cy, aabb.max_x, cy) ];
            for (idx, (hx, hy, ox, oy)) in coords.iter().enumerate() {
                if (x - hx).abs() <= hs && (y - hy).abs() <= hs { self.state = SelectState::Scaling(idx); self.active_pivot = if center { (px, py) } else { (*ox, *oy) }; return; }
            }
            let rs = 20.0; let corners = [(aabb.min_x, aabb.min_y), (aabb.max_x, aabb.min_y), (aabb.max_x, aabb.max_y), (aabb.min_x, aabb.max_y)];
            for (cx, cy) in corners {
                if (x - cx).abs() <= rs && (y - cy).abs() <= rs { self.state = SelectState::Rotating; self.active_pivot = (px, py); self.current_cursor = ROTATE_CURSOR; return; }
            }
            if x >= aabb.min_x && x <= aabb.max_x && y >= aabb.min_y && y <= aabb.max_y { self.state = SelectState::Translating; self.current_cursor = "move"; return; }
        }

        if let Some(id) = scene.hit_test(x, y) {
            scene.selected_strokes.clear(); scene.selected_strokes.insert(id); self.state = SelectState::Translating; self.current_cursor = "move"; self.custom_pivot = None; 
        } else {
            scene.selected_strokes.clear(); self.lasso_points.clear();
            let pt = Point { x, y, pressure }; if pt.is_valid() { self.lasso_points.push(pt); }
            self.current_cursor = "default"; self.custom_pivot = None;
        }
    }

    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, _center: bool, scene: &mut SceneManager, cw: f32, ch: f32) {
        match self.state {
            SelectState::DraggingPivot => { self.custom_pivot = Some((x, y)); },
            SelectState::Translating => { self.total_dx = x - self.start_x; self.total_dy = y - self.start_y; },
            SelectState::Rotating => { let (px, py) = self.active_pivot; let start_angle = (self.start_y - py).atan2(self.start_x - px); let current_angle = (y - py).atan2(x - px); self.total_rot = current_angle - start_angle; },
            SelectState::Scaling(idx) => {
                let (px, py) = self.active_pivot; let start_vec_x = self.start_x - px; let start_vec_y = self.start_y - py; let cur_vec_x = x - px; let cur_vec_y = y - py;
                let mut sx = if start_vec_x.abs() > 1.0 { cur_vec_x / start_vec_x } else { 1.0 }; let mut sy = if start_vec_y.abs() > 1.0 { cur_vec_y / start_vec_y } else { 1.0 };
                if idx == 4 || idx == 6 { sx = 1.0; } if idx == 5 || idx == 7 { sy = 1.0; }
                if constrain && idx < 4 { let s = sx.abs().max(sy.abs()); sx = sx.signum() * s; sy = sy.signum() * s; }
                self.total_sx = sx; self.total_sy = sy;
            },
            SelectState::Idle => { let pt = Point { x, y, pressure }; if pt.is_valid() { self.lasso_points.push(pt); } return; }
        }

        if self.state != SelectState::Idle && self.state != SelectState::DraggingPivot {
            if let Some((_, layer)) = scene.get_active_art_layer_mut() {
                for (id, orig_el) in &self.original_elements {
                    let mut new_el = orig_el.clone();
                    new_el.transform(self.total_dx, self.total_dy, self.total_sx, self.total_sy, self.total_rot, self.active_pivot.0, self.active_pivot.1, cw, ch);
                    layer.vector_elements.insert(*id, new_el);
                }
            }
        }
    }

    fn on_pointer_up(&mut self, _id_allocator: &mut IdAllocator, _cw: f32, _ch: f32, scene: &mut SceneManager) -> Option<Box<dyn Command>> {
        match self.state {
            SelectState::Translating | SelectState::Scaling(_) | SelectState::Rotating => {
                if self.total_dx.abs() > 0.1 || self.total_dy.abs() > 0.1 || (self.total_sx - 1.0).abs() > 0.01 || (self.total_sy - 1.0).abs() > 0.01 || self.total_rot.abs() > 0.01 {
                    if let SelectState::Translating = self.state { if let Some((px, py)) = self.custom_pivot { self.custom_pivot = Some((px + self.total_dx, py + self.total_dy)); } }
                    let mut new_elements = Vec::new();
                    if let Some((_, layer)) = scene.get_active_art_layer() {
                        for (id, _) in &self.original_elements { if let Some(el) = layer.vector_elements.get(id) { new_elements.push((*id, el.clone())); } }
                    }
                    self.state = SelectState::Idle;
                    let element_id = scene.active_element_id.unwrap_or(1);
                    let drawing_id = scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).copied().unwrap_or(1);
                    return Some(Box::new(AffineCommand { element_id, drawing_id, art_layer: scene.active_art_layer, old_elements: self.original_elements.clone(), new_elements }));
                }
            },
            SelectState::DraggingPivot => { self.state = SelectState::Idle; },
            SelectState::Idle => {
                if self.lasso_points.len() > 2 {
                    let hit_ids = scene.hit_test_lasso(&self.lasso_points);
                    for id in hit_ids { scene.selected_strokes.insert(id); }
                }
                self.lasso_points.clear();
            }
        }
        self.state = SelectState::Idle; None
    }

    fn get_preview_mesh(&self, cw: f32, ch: f32) -> (Vec<Vertex>, Vec<u16>) {
        if self.state != SelectState::Idle || self.lasso_points.len() < 2 { return (Vec::new(), Vec::new()); }
        let mut closed_points = self.lasso_points.clone();
        if closed_points.len() > 2 { closed_points.push(closed_points[0]); }
        let (verts, inds, _) = Extruder::extrude_centerline(&closed_points, 1.5, [1.0, 0.45, 0.0, 1.0], cw, ch);
        (verts, inds)
    }
}