use crate::tools::{CanvasTool, PreviewBlendMode};
use crate::geometry::Point;
use crate::geometry::boolean::BooleanSlicer;
use crate::geometry::tessellator::Extruder;
use crate::settings::EngineSettings;
use crate::math::AABB;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::{Command, CutCommand, BatchCommand};
use crate::math::Vertex;

pub struct CutterTool {
    raw_points: Vec<Point>,
}

impl CutterTool {
    pub fn new() -> Self { Self { raw_points: Vec::with_capacity(256) } }
}

impl CanvasTool for CutterTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _settings: EngineSettings) {
        self.raw_points.clear();
        let pt = Point { x, y, pressure };
        if pt.is_valid() { self.raw_points.push(pt); }
    }

    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _graph: &AnimGraph) {
        let pt = Point { x, y, pressure };
        if pt.is_valid() { self.raw_points.push(pt); }
    }

    fn on_pointer_up(
        &mut self, active_node_id: NodeId, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, graph: &AnimGraph
    ) -> Option<Box<dyn Command>> {
        if self.raw_points.len() < 3 { return None; }

        let mut sweep_aabb = AABB::empty();
        for pt in &self.raw_points { sweep_aabb.expand_to_include(pt.x, pt.y, 0.0); }

        let overlapping_ids = graph.query_spatial_grid_ids(active_node_id, &sweep_aabb);
        let mut batch_commands = Vec::new();

        if let crate::graph::NodeType::VectorLayer { elements, .. } = &graph.nodes.get(&active_node_id).unwrap().payload {
            for stroke_id in overlapping_ids {
                if let Some(original_element) = elements.get(&stroke_id) {
                    
                    let new_fragments = BooleanSlicer::lasso_slice(original_element, &self.raw_points, canvas_width, canvas_height);
                    
                    if new_fragments.len() > 1 {
                        let mut new_fragments_with_ids = Vec::new();
                        for frag in new_fragments { new_fragments_with_ids.push((id_allocator.generate(), frag)); }

                        batch_commands.push(Box::new(CutCommand {
                            target_node_id: active_node_id, severed_stroke_id: stroke_id, 
                            original_element: original_element.clone(), new_fragments: new_fragments_with_ids,
                        }) as Box<dyn Command>);
                    }
                }
            }
        }

        self.raw_points.clear();
        if !batch_commands.is_empty() { return Some(Box::new(BatchCommand { commands: batch_commands })); }
        None
    }

    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if self.raw_points.len() < 2 { return (Vec::new(), Vec::new()); }
        
        let mut closed_points = self.raw_points.clone();
        if closed_points.len() > 2 {
            closed_points.push(closed_points[0]); // Visually snap the lasso closed while drawing
        }
        
        // Draw the lasso as a thin, bright blue line
        let (verts, inds, _) = Extruder::extrude_centerline(&closed_points, 2.0, [0.2, 0.6, 1.0, 1.0], canvas_width, canvas_height);
        (verts, inds)
    }

    fn get_preview_blend_mode(&self) -> PreviewBlendMode { PreviewBlendMode::Normal }
}