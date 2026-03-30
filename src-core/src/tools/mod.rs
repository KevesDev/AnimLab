pub mod brush;
pub mod pencil;
pub mod eraser;

use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::Command;
use crate::math::Vertex;

pub enum PreviewBlendMode {
    Normal,
    Subtract, // AAA: Hardware deletion via the fragment shader
}

pub trait CanvasTool: Send + Sync {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, settings: EngineSettings);
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, graph: &AnimGraph);
    fn on_pointer_up(
        &mut self, active_node_id: NodeId, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, graph: &AnimGraph
    ) -> Option<Box<dyn Command>>;
    
    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>);
    
    // Tools declare how their live previews should interact with the framebuffer
    fn get_preview_blend_mode(&self) -> PreviewBlendMode {
        PreviewBlendMode::Normal 
    }
}