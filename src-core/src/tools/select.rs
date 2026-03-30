use crate::tools::{CanvasTool, PreviewBlendMode};
use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::{Command, TransformCommand};
use crate::math::Vertex;

pub struct SelectTool {
    is_dragging: bool,
    start_x: f32, start_y: f32,
    last_x: f32, last_y: f32,
    total_dx: f32, total_dy: f32,
}

impl SelectTool {
    pub fn new() -> Self { Self { is_dragging: false, start_x: 0.0, start_y: 0.0, last_x: 0.0, last_y: 0.0, total_dx: 0.0, total_dy: 0.0 } }
}

impl CanvasTool for SelectTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, _pressure: f32, _settings: EngineSettings, active_node_id: NodeId, graph: &mut AnimGraph) {
        let hit_id = graph.hit_test(active_node_id, x, y);
        
        match hit_id {
            Some(id) => {
                if !graph.selected_strokes.contains(&id) {
                    graph.selected_strokes.clear();
                    graph.selected_strokes.insert(id);
                }
                self.is_dragging = true;
                self.start_x = x; self.start_y = y;
                self.last_x = x; self.last_y = y;
                self.total_dx = 0.0; self.total_dy = 0.0;
            },
            None => {
                graph.selected_strokes.clear();
                self.is_dragging = false;
            }
        }
    }

    fn on_pointer_move(&mut self, x: f32, y: f32, _pressure: f32, active_node_id: NodeId, graph: &mut AnimGraph, canvas_width: f32, canvas_height: f32) {
        if self.is_dragging {
            let dx = x - self.last_x;
            let dy = y - self.last_y;
            self.total_dx += dx;
            self.total_dy += dy;

            // Live-translate the actual vector data for a perfectly smooth 144hz preview
            if let Some(node) = graph.nodes.get_mut(&active_node_id) {
                if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                    for id in &graph.selected_strokes {
                        if let Some(element) = elements.get_mut(id) {
                            element.translate(dx, dy, canvas_width, canvas_height);
                        }
                    }
                }
            }
            self.last_x = x; self.last_y = y;
        }
    }

    fn on_pointer_up(
        &mut self, active_node_id: NodeId, _id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, graph: &mut AnimGraph
    ) -> Option<Box<dyn Command>> {
        if self.is_dragging && (self.total_dx.abs() > 0.1 || self.total_dy.abs() > 0.1) {
            // Revert the live-preview translation so the Command History can permanently execute it.
            if let Some(node) = graph.nodes.get_mut(&active_node_id) {
                if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                    for id in &graph.selected_strokes {
                        if let Some(element) = elements.get_mut(id) {
                            element.translate(-self.total_dx, -self.total_dy, canvas_width, canvas_height);
                        }
                    }
                }
            }

            self.is_dragging = false;
            
            let selected_vec: Vec<u64> = graph.selected_strokes.iter().cloned().collect();
            Some(Box::new(TransformCommand {
                target_node_id: active_node_id, stroke_ids: selected_vec, dx: self.total_dx, dy: self.total_dy
            }))
        } else {
            self.is_dragging = false;
            None
        }
    }

    fn get_preview_mesh(&self, _canvas_width: f32, _canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) { (Vec::new(), Vec::new()) }
}