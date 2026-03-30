use crate::tools::CanvasTool;
use crate::geometry::{Point, ContourStroke, VectorElement};
use crate::geometry::spline::smooth_spline;
use crate::geometry::tessellator::Extruder;
use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::{Command, AddStrokeCommand};
use crate::math::Vertex;

pub struct BrushTool { raw_points: Vec<Point>, settings_snapshot: Option<EngineSettings> }
impl BrushTool { pub fn new() -> Self { Self { raw_points: Vec::with_capacity(256), settings_snapshot: None } } }

impl CanvasTool for BrushTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, settings: EngineSettings) {
        self.settings_snapshot = Some(settings); self.raw_points.clear();
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _graph: &AnimGraph) {
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }
    fn on_pointer_up(&mut self, active_node_id: NodeId, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, _graph: &AnimGraph) -> Option<Box<dyn Command>> {
        if let Some(settings) = self.settings_snapshot.take() {
            let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
            let (shape, vertices, indices, aabb) = Extruder::extrude_contour(&smoothed, settings.brush_thickness, settings.brush_color, canvas_width, canvas_height);
            // AAA FIX: Initialize with empty mask array
            let contour = ContourStroke { shape, color: settings.brush_color, vertices, indices, aabb, eraser_masks: Vec::new() };
            self.raw_points.clear();
            Some(Box::new(AddStrokeCommand { target_node_id: active_node_id, stroke_id: id_allocator.generate(), element: VectorElement::Contour(contour) }))
        } else { None }
    }
    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if let Some(settings) = &self.settings_snapshot {
            let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
            let (_, verts, inds, _) = Extruder::extrude_contour(&smoothed, settings.brush_thickness, settings.brush_color, canvas_width, canvas_height);
            (verts, inds)
        } else { (Vec::new(), Vec::new()) }
    }
}