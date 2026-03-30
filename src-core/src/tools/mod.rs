pub mod brush;

use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, StrokeId};
use crate::command::Command;
use crate::math::Vertex;

/// The AAA Master Interface for all Compositor Tools.
/// The core engine knows nothing about specific tool logic; it only speaks to this trait.
pub trait CanvasTool: Send + Sync {
    /// Fires the moment the pen touches the glass. 
    /// The tool MUST capture a snapshot of the EngineSettings here to prevent 
    /// locking global memory threads during the high-frequency move loop.
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, settings: EngineSettings);
    
    /// The hot loop. Fires 144+ times a second.
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, graph: &AnimGraph);
    
    /// Fires when the pen leaves the glass.
    /// The tool resolves its math and outputs a Delta Command for the Undo Stack.
    fn on_pointer_up(
        &mut self, 
        active_node_id: NodeId, 
        next_stroke_id: StrokeId, 
        canvas_width: f32, 
        canvas_height: f32, 
        graph: &AnimGraph
    ) -> Option<Box<dyn Command>>;
    
    /// Safely queried by the WebGPU render loop to draw the temporary 
    /// mesh while the user is actively dragging the pen.
    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>);
}