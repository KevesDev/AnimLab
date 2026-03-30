pub mod brush;
pub mod pencil;
pub mod eraser;
pub mod cutter;
pub mod select; // AAA FIX: Added Select to the tool registry

use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, IdAllocator};
use crate::command::Command;
use crate::math::Vertex;

pub enum PreviewBlendMode { Normal, Subtract }

pub trait CanvasTool: Send + Sync {
    // Note: Tools now pass the mutable graph during interaction to allow real-time transformations
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, settings: EngineSettings, active_node_id: NodeId, graph: &mut AnimGraph);
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, active_node_id: NodeId, graph: &mut AnimGraph, canvas_width: f32, canvas_height: f32);
    fn on_pointer_up(
        &mut self, active_node_id: NodeId, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, graph: &mut AnimGraph
    ) -> Option<Box<dyn Command>>;
    
    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>);
    fn get_preview_blend_mode(&self) -> PreviewBlendMode { PreviewBlendMode::Normal }
}