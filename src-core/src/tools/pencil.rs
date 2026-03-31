use crate::tools::CanvasTool;
use crate::geometry::{Point, CenterlineStroke, VectorElement};
use crate::geometry::spline::smooth_spline;
use crate::geometry::tessellator::Extruder;
use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::{Command, AddStrokeCommand};
use crate::math::Vertex;

pub struct PencilTool { raw_points: Vec<Point>, settings_snapshot: Option<EngineSettings> }
impl PencilTool { pub fn new() -> Self { Self { raw_points: Vec::with_capacity(256), settings_snapshot: None } } }

impl CanvasTool for PencilTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, settings: EngineSettings, _active_node_id: NodeId, _graph: &mut AnimGraph) {
        self.settings_snapshot = Some(settings); self.raw_points.clear();
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, _active_node_id: NodeId, _graph: &mut AnimGraph, _canvas_width: f32, _canvas_height: f32) {
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }
    fn on_pointer_up(&mut self, active_node_id: NodeId, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, _graph: &mut AnimGraph) -> Option<Box<dyn Command>> {
        if let Some(settings) = self.settings_snapshot.take() {
            if self.raw_points.len() < 2 { return None; }
            let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
            let (vertices, indices, aabb) = Extruder::extrude_centerline(&smoothed, settings.brush_thickness, settings.brush_color, canvas_width, canvas_height);
            let centerline = CenterlineStroke { points: self.raw_points.clone(), thickness: settings.brush_thickness, color: settings.brush_color, vertices, indices, aabb };
            self.raw_points.clear();
            Some(Box::new(AddStrokeCommand { target_node_id: active_node_id, stroke_id: id_allocator.generate(), element: VectorElement::Centerline(centerline) }))
        } else { None }
    }
    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if let Some(settings) = &self.settings_snapshot {
            if self.raw_points.len() < 2 { return (Vec::new(), Vec::new()); }
            let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
            let (verts, inds, _) = Extruder::extrude_centerline(&smoothed, settings.brush_thickness, settings.brush_color, canvas_width, canvas_height);
            (verts, inds)
        } else { (Vec::new(), Vec::new()) }
    }
}