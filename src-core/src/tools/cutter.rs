use crate::tools::{CanvasTool, PreviewBlendMode};
use crate::geometry::Point;
use crate::geometry::boolean::BooleanSlicer;
use crate::geometry::tessellator::Extruder;
use crate::settings::EngineSettings;
use crate::math::AABB;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::{Command, CutCommand, BatchCommand, TransformCommand};
use crate::math::Vertex;

pub struct CutterTool { 
    raw_points: Vec<Point>,
    is_dragging: bool,
    last_x: f32, last_y: f32,
    total_dx: f32, total_dy: f32,
}

impl CutterTool { 
    pub fn new() -> Self { Self { raw_points: Vec::with_capacity(256), is_dragging: false, last_x: 0.0, last_y: 0.0, total_dx: 0.0, total_dy: 0.0 } } 
}

impl CanvasTool for CutterTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _settings: EngineSettings, active_node_id: NodeId, graph: &mut AnimGraph) {
        if let Some(id) = graph.hit_test(active_node_id, x, y) {
            if graph.selected_strokes.contains(&id) {
                // User clicked an existing selection to move it
                self.is_dragging = true;
                self.last_x = x; self.last_y = y;
                self.total_dx = 0.0; self.total_dy = 0.0;
                return;
            }
        }
        // User clicked dead space or a new area to initiate a fresh cut
        graph.selected_strokes.clear();
        self.is_dragging = false;
        self.raw_points.clear();
        let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
    }
    
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, active_node_id: NodeId, graph: &mut AnimGraph, canvas_width: f32, canvas_height: f32) {
        if self.is_dragging {
            let dx = x - self.last_x; let dy = y - self.last_y;
            self.total_dx += dx; self.total_dy += dy;
            if let Some(node) = graph.nodes.get_mut(&active_node_id) {
                if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                    for id in &graph.selected_strokes {
                        if let Some(element) = elements.get_mut(id) { element.translate(dx, dy, canvas_width, canvas_height); }
                    }
                }
            }
            self.last_x = x; self.last_y = y;
        } else {
            let pt = Point { x, y, pressure }; if pt.is_valid() { self.raw_points.push(pt); }
        }
    }
    
    fn on_pointer_up(
        &mut self, active_node_id: NodeId, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, graph: &mut AnimGraph
    ) -> Option<Box<dyn Command>> {
        if self.is_dragging {
            self.is_dragging = false;
            if self.total_dx.abs() > 0.1 || self.total_dy.abs() > 0.1 {
                if let Some(node) = graph.nodes.get_mut(&active_node_id) {
                    if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                        for id in &graph.selected_strokes {
                            if let Some(element) = elements.get_mut(id) { element.translate(-self.total_dx, -self.total_dy, canvas_width, canvas_height); }
                        }
                    }
                }
                let selected_vec: Vec<u64> = graph.selected_strokes.iter().cloned().collect();
                return Some(Box::new(TransformCommand { target_node_id: active_node_id, stroke_ids: selected_vec, dx: self.total_dx, dy: self.total_dy }));
            }
            return None;
        }

        if self.raw_points.len() < 3 { return None; }

        let mut sweep_aabb = AABB::empty();
        for pt in &self.raw_points { sweep_aabb.expand_to_include(pt.x, pt.y, 0.0); }

        let overlapping_ids = graph.query_spatial_grid_ids(active_node_id, &sweep_aabb);
        let mut batch_commands = Vec::new();

        if let crate::graph::NodeType::VectorLayer { elements, .. } = &graph.nodes.get(&active_node_id).unwrap().payload {
            for stroke_id in overlapping_ids {
                if let Some(original_element) = elements.get(&stroke_id) {
                    let (inside_frags, outside_frags) = BooleanSlicer::lasso_slice(original_element, &self.raw_points, canvas_width, canvas_height);
                    
                    if !inside_frags.is_empty() {
                        let mut new_fragments_with_ids = Vec::new();
                        
                        for frag in inside_frags { 
                            let new_id = id_allocator.generate();
                            graph.selected_strokes.insert(new_id); // AAA Auto-Select Behavior!
                            new_fragments_with_ids.push((new_id, frag)); 
                        }
                        for frag in outside_frags { 
                            new_fragments_with_ids.push((id_allocator.generate(), frag)); 
                        }
                        batch_commands.push(Box::new(CutCommand {
                            target_node_id: active_node_id, severed_stroke_id: stroke_id, original_element: original_element.clone(), new_fragments: new_fragments_with_ids,
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
        if self.is_dragging || self.raw_points.len() < 2 { return (Vec::new(), Vec::new()); }
        let mut closed_points = self.raw_points.clone();
        if closed_points.len() > 2 { closed_points.push(closed_points[0]); }
        let (verts, inds, _) = Extruder::extrude_centerline(&closed_points, 2.0, [0.2, 0.6, 1.0, 1.0], canvas_width, canvas_height);
        (verts, inds)
    }
    
    fn get_preview_blend_mode(&self) -> PreviewBlendMode { PreviewBlendMode::Normal }
}