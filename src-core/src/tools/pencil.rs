use crate::tools::CanvasTool;
use crate::settings::EngineSettings;
use crate::graph::{SceneManager, IdAllocator};
use crate::command::{Command, AddStrokeCommand};
use crate::geometry::tessellator::Extruder;
use crate::geometry::{Point, CenterlineStroke, VectorElement};
use crate::math::Vertex;

pub struct PencilTool { points: Vec<Point>, active_thickness: f32, active_color: [f32; 4] }

impl PencilTool { pub fn new() -> Self { Self { points: Vec::with_capacity(1024), active_thickness: 2.0, active_color: [0.0, 0.0, 0.0, 1.0] } } }

impl CanvasTool for PencilTool {
    fn get_cursor(&self) -> &'static str { "crosshair" }
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, settings: EngineSettings, _scene: &mut SceneManager, _id_allocator: &mut IdAllocator) {
        self.points.clear(); self.active_thickness = settings.brush_thickness * 0.5; self.active_color = settings.brush_color;
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.points.push(pt); }
    }
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, _scene: &mut SceneManager, _cw: f32, _ch: f32) {
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.points.push(pt); }
    }
    fn on_pointer_up(&mut self, id_allocator: &mut IdAllocator, cw: f32, ch: f32, scene: &mut SceneManager) -> Option<Box<dyn Command>> {
        if self.points.len() < 2 { self.points.clear(); return None; }
        
        let settings = crate::settings::get_settings();
        let stroke_id = id_allocator.generate();
        
        // AAA INTEGRITY RESTORED: Utilizing your original smoothing pipeline
        let smoothed_points = crate::geometry::spline::smooth_spline(&self.points, settings.smoothing_level);
        let (vertices, indices, aabb) = Extruder::extrude_centerline(&smoothed_points, self.active_thickness, self.active_color, cw, ch);
        let element = VectorElement::Centerline(CenterlineStroke { points: smoothed_points, thickness: self.active_thickness, color: self.active_color, vertices, indices, aabb });
        self.points.clear();

        let element_id = scene.active_element_id.unwrap_or(1);
        let drawing_id = *scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).unwrap_or(&1);

        Some(Box::new(AddStrokeCommand { element_id, drawing_id, art_layer: scene.active_art_layer, stroke_id, element }))
    }
    fn get_preview_mesh(&self, cw: f32, ch: f32) -> (Vec<Vertex>, Vec<u16>) {
        if self.points.len() < 2 { return (Vec::new(), Vec::new()); }
        let settings = crate::settings::get_settings();
        let smoothed_points = crate::geometry::spline::smooth_spline(&self.points, settings.smoothing_level);
        let (verts, inds, _) = Extruder::extrude_centerline(&smoothed_points, self.active_thickness, self.active_color, cw, ch);
        (verts, inds)
    }
}