use crate::tools::{CanvasTool, PreviewBlendMode};
use crate::geometry::Point;
use crate::geometry::boolean::BooleanSlicer;
use crate::geometry::spline::smooth_spline;
use crate::geometry::tessellator::Extruder;
use crate::settings::EngineSettings;
use crate::math::AABB;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::{Command, CutCommand, BatchCommand};
use crate::math::Vertex;

pub struct EraserTool { raw_points: Vec<Point>, settings_snapshot: Option<EngineSettings> }
impl EraserTool { pub fn new() -> Self { Self { raw_points: Vec::with_capacity(256), settings_snapshot: None } } }

impl CanvasTool for EraserTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, settings: EngineSettings, _active_node_id: NodeId, _graph: &mut AnimGraph) {
        self.settings_snapshot = Some(settings); self.raw_points.clear();
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _active_node_id: NodeId, _graph: &mut AnimGraph, _canvas_width: f32, _canvas_height: f32) {
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }

    fn on_pointer_up(
        &mut self, active_node_id: NodeId, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, graph: &mut AnimGraph
    ) -> Option<Box<dyn Command>> {
        if let Some(settings) = self.settings_snapshot.take() {
            if self.raw_points.len() < 2 { return None; }

            let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
            let mut sweep_aabb = AABB::empty();
            let max_r = settings.brush_thickness / 2.0;
            for pt in &smoothed { sweep_aabb.expand_to_include(pt.x, pt.y, max_r); }

            let mut overlapping_ids = std::collections::HashSet::new();
            overlapping_ids.extend(graph.query_spatial_grid_ids(active_node_id, &sweep_aabb));
            
            let mut batch_commands = Vec::new();

            if let crate::graph::NodeType::VectorLayer { elements, .. } = &graph.nodes.get(&active_node_id).unwrap().payload {
                for stroke_id in overlapping_ids {
                    if let Some(original_element) = elements.get(&stroke_id) {
                        let new_fragments = BooleanSlicer::slice_element(
                            original_element, &self.raw_points, settings.brush_thickness, canvas_width, canvas_height, settings.smoothing_level
                        );
                        
                        let mut new_fragments_with_ids = Vec::new();
                        for frag in new_fragments { new_fragments_with_ids.push((id_allocator.generate(), frag)); }

                        batch_commands.push(Box::new(CutCommand {
                            target_node_id: active_node_id, severed_stroke_id: stroke_id, original_element: original_element.clone(), new_fragments: new_fragments_with_ids,
                        }) as Box<dyn Command>);
                    }
                }
            }

            self.raw_points.clear();
            if !batch_commands.is_empty() { return Some(Box::new(BatchCommand { commands: batch_commands })); }
        }
        None
    }

    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if let Some(settings) = &self.settings_snapshot {
            if self.raw_points.len() < 2 { return (Vec::new(), Vec::new()); }
            let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
            let (verts, inds, _) = Extruder::extrude_centerline(&smoothed, settings.brush_thickness, [0.08, 0.09, 0.10, 1.0], canvas_width, canvas_height);
            (verts, inds)
        } else { (Vec::new(), Vec::new()) }
    }

    fn get_preview_blend_mode(&self) -> PreviewBlendMode { PreviewBlendMode::Normal }
}