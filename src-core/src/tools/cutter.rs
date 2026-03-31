use crate::tools::CanvasTool;
use crate::geometry::Point;
use crate::geometry::boolean::BooleanSlicer;
use crate::geometry::tessellator::Extruder;
use crate::settings::EngineSettings;
use crate::math::{AABB, Vertex};
use crate::graph::{SceneManager, IdAllocator, StrokeId};
use crate::command::{Command, CutCommand, BatchCommand, AffineCommand};
use crate::geometry::VectorElement;

pub struct CutterTool { 
    raw_points: Vec<Point>,
    is_dragging: bool,
    last_x: f32, last_y: f32,
    total_dx: f32, total_dy: f32,
    original_elements: Vec<(StrokeId, VectorElement)>,
}

impl CutterTool { 
    pub fn new() -> Self { Self { raw_points: Vec::with_capacity(256), is_dragging: false, last_x: 0.0, last_y: 0.0, total_dx: 0.0, total_dy: 0.0, original_elements: Vec::new() } } 
}

impl CanvasTool for CutterTool {
    fn get_cursor(&self) -> &'static str { "crosshair" }

    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, _settings: EngineSettings, scene: &mut SceneManager, _id_allocator: &mut IdAllocator) {
        // AAA INTEGRITY RESTORED: Dragging behavior check
        if let Some(id) = scene.hit_test(x, y) {
            if scene.selected_strokes.contains(&id) {
                self.is_dragging = true;
                self.last_x = x; self.last_y = y;
                self.total_dx = 0.0; self.total_dy = 0.0;
                
                self.original_elements.clear();
                if let Some((_, layer)) = scene.get_active_art_layer() {
                    for sel_id in &scene.selected_strokes {
                        if let Some(el) = layer.vector_elements.get(sel_id) {
                            self.original_elements.push((*sel_id, el.clone()));
                        }
                    }
                }
                return;
            }
        }
        scene.selected_strokes.clear();
        self.is_dragging = false;
        self.raw_points.clear();
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }
    
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, scene: &mut SceneManager, canvas_width: f32, canvas_height: f32) {
        if self.is_dragging {
            let dx = x - self.last_x; let dy = y - self.last_y;
            self.total_dx += dx; self.total_dy += dy;
            
            if let Some((_, layer)) = scene.get_active_art_layer_mut() {
                for (id, orig_el) in &self.original_elements {
                    let mut new_el = orig_el.clone();
                    new_el.translate(self.total_dx, self.total_dy, canvas_width, canvas_height);
                    layer.vector_elements.insert(*id, new_el);
                }
            }
            self.last_x = x; self.last_y = y;
        } else {
            let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
        }
    }
    
    fn on_pointer_up(&mut self, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, scene: &mut SceneManager) -> Option<Box<dyn Command>> {
        let element_id = scene.active_element_id.unwrap_or(1);
        let drawing_id = scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).copied().unwrap_or(1);
        let art_layer = scene.active_art_layer;

        // 1. Resolve Dragging
        if self.is_dragging {
            self.is_dragging = false;
            if self.total_dx.abs() > 0.1 || self.total_dy.abs() > 0.1 {
                let mut new_elements = Vec::new();
                if let Some((_, layer)) = scene.get_active_art_layer() {
                    for (id, _) in &self.original_elements {
                        if let Some(el) = layer.vector_elements.get(id) { new_elements.push((*id, el.clone())); }
                    }
                }
                return Some(Box::new(AffineCommand {
                    element_id, drawing_id, art_layer,
                    old_elements: self.original_elements.clone(),
                    new_elements
                }));
            }
            return None;
        }

        // 2. Resolve Lasso Cut
        if self.raw_points.len() < 3 { return None; }

        let mut sweep_aabb = AABB::empty();
        for pt in &self.raw_points { sweep_aabb.expand_to_include(pt.x, pt.y, 0.0); }

        let mut batch_commands = Vec::new();
        let mut to_cut = Vec::new();

        // AAA INTEGRITY RESTORED: Find all elements intersecting the lasso bounds exactly like your original logic
        if let Some((_, layer)) = scene.get_active_art_layer() {
            for (id, element) in &layer.vector_elements {
                if element.aabb().intersects(&sweep_aabb) {
                    to_cut.push((*id, element.clone()));
                }
            }
        }

        let mut newly_selected = Vec::new();

        for (stroke_id, original_element) in to_cut {
            let (inside_frags, outside_frags) = BooleanSlicer::lasso_slice(&original_element, &self.raw_points, canvas_width, canvas_height);
            
            if !inside_frags.is_empty() {
                let mut new_fragments_with_ids = Vec::new();
                
                for frag in inside_frags { 
                    let new_id = id_allocator.generate();
                    newly_selected.push(new_id); // Auto-select the sliced piece
                    new_fragments_with_ids.push((new_id, frag)); 
                }
                for frag in outside_frags { 
                    new_fragments_with_ids.push((id_allocator.generate(), frag)); 
                }
                batch_commands.push(Box::new(CutCommand {
                    element_id, drawing_id, art_layer, severed_stroke_id: stroke_id, original_element, new_fragments: new_fragments_with_ids,
                }) as Box<dyn Command>);
            }
        }
        
        self.raw_points.clear();
        
        if !batch_commands.is_empty() { 
            scene.selected_strokes.clear();
            for id in newly_selected { scene.selected_strokes.insert(id); }
            return Some(Box::new(BatchCommand { commands: batch_commands })); 
        }
        None
    }

    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if self.is_dragging || self.raw_points.len() < 2 { return (Vec::new(), Vec::new()); }
        let mut closed_points = self.raw_points.clone();
        if closed_points.len() > 2 { closed_points.push(closed_points[0]); }
        let (verts, inds, _) = Extruder::extrude_centerline(&closed_points, 1.5, [1.0, 0.45, 0.0, 1.0], canvas_width, canvas_height);
        (verts, inds)
    }
}