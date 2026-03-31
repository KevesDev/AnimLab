use crate::tools::{CanvasTool, PreviewBlendMode};
use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::{Command, TransformCommand};
use crate::geometry::tessellator::Extruder;
use crate::geometry::Point;
use crate::math::Vertex;

pub struct SelectTool {
    is_dragging: bool,
    lasso_points: Vec<Point>,
    start_x: f32, start_y: f32,
    last_x: f32, last_y: f32,
    total_dx: f32, total_dy: f32,
    current_cursor: &'static str,
}

impl SelectTool {
    pub fn new() -> Self { 
        Self { 
            is_dragging: false, lasso_points: Vec::with_capacity(256), 
            start_x: 0.0, start_y: 0.0, last_x: 0.0, last_y: 0.0, 
            total_dx: 0.0, total_dy: 0.0, current_cursor: "default" 
        } 
    }
}

impl CanvasTool for SelectTool {
    fn get_cursor(&self) -> &'static str { self.current_cursor }

    // AAA FIX: Calculate Cursor Context continuously during hover
    fn on_pointer_hover(&mut self, x: f32, y: f32, active_node_id: NodeId, graph: &AnimGraph) {
        if self.is_dragging { return; } // Keep "move" locked while dragging
        
        let mut cursor = "default";
        if let Some(aabb) = graph.get_selection_aabb(active_node_id) {
            let hs = 8.0; 
            let coords = [
                ("nwse-resize", aabb.min_x, aabb.min_y), ("nesw-resize", aabb.max_x, aabb.min_y), 
                ("nwse-resize", aabb.max_x, aabb.max_y), ("nesw-resize", aabb.min_x, aabb.max_y),
                ("ns-resize", (aabb.min_x + aabb.max_x) / 2.0, aabb.min_y), ("ns-resize", (aabb.min_x + aabb.max_x) / 2.0, aabb.max_y),
                ("ew-resize", aabb.min_x, (aabb.min_y + aabb.max_y) / 2.0), ("ew-resize", aabb.max_x, (aabb.min_y + aabb.max_y) / 2.0)
            ];
            let mut over_handle = false;
            for (c, hx, hy) in coords {
                if (x - hx).abs() <= hs && (y - hy).abs() <= hs { cursor = c; over_handle = true; break; }
            }
            if !over_handle {
                if x >= aabb.min_x && x <= aabb.max_x && y >= aabb.min_y && y <= aabb.max_y { cursor = "move"; } 
                else if graph.hit_test(active_node_id, x, y).is_some() { cursor = "pointer"; }
            }
        } else if graph.hit_test(active_node_id, x, y).is_some() {
            cursor = "pointer";
        }
        self.current_cursor = cursor;
    }

    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _settings: EngineSettings, active_node_id: NodeId, graph: &mut AnimGraph) {
        if let Some(aabb) = graph.get_selection_aabb(active_node_id) {
            let hit_padding = 10.0;
            if x >= aabb.min_x - hit_padding && x <= aabb.max_x + hit_padding && y >= aabb.min_y - hit_padding && y <= aabb.max_y + hit_padding {
                self.is_dragging = true;
                self.start_x = x; self.start_y = y; self.last_x = x; self.last_y = y; self.total_dx = 0.0; self.total_dy = 0.0;
                self.current_cursor = "move";
                return;
            }
        }

        let hit_id = graph.hit_test(active_node_id, x, y);
        match hit_id {
            Some(id) => {
                graph.selected_strokes.clear(); graph.selected_strokes.insert(id);
                self.is_dragging = true;
                self.start_x = x; self.start_y = y; self.last_x = x; self.last_y = y; self.total_dx = 0.0; self.total_dy = 0.0;
                self.current_cursor = "move";
            },
            None => { 
                graph.selected_strokes.clear(); self.is_dragging = false; self.lasso_points.clear();
                let pt = Point { x, y, pressure }; if pt.is_valid() { self.lasso_points.push(pt); }
                self.current_cursor = "default";
            }
        }
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
            let pt = Point { x, y, pressure }; if pt.is_valid() { self.lasso_points.push(pt); }
        }
    }

    fn on_pointer_up(&mut self, active_node_id: NodeId, _id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, graph: &mut AnimGraph) -> Option<Box<dyn Command>> {
        if self.is_dragging {
            self.is_dragging = false;
            self.current_cursor = "move";
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
        } else {
            if self.lasso_points.len() > 2 {
                let hit_ids = graph.hit_test_lasso(active_node_id, &self.lasso_points);
                for id in hit_ids { graph.selected_strokes.insert(id); }
            }
            self.lasso_points.clear();
            self.current_cursor = "default";
        }
        None
    }

    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if self.is_dragging || self.lasso_points.len() < 2 { return (Vec::new(), Vec::new()); }
        let mut closed_points = self.lasso_points.clone();
        if closed_points.len() > 2 { closed_points.push(closed_points[0]); }
        let (verts, inds, _) = Extruder::extrude_centerline(&closed_points, 1.5, [1.0, 0.45, 0.0, 1.0], canvas_width, canvas_height);
        (verts, inds)
    }

    fn get_preview_blend_mode(&self) -> PreviewBlendMode { PreviewBlendMode::Normal }
}