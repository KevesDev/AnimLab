use log::{info, warn};
use crate::graph::{AnimGraph, NodeId, StrokeId};
use crate::stroke::Stroke;

/// The strict AAA contract for any action that mutates engine state.
/// Every tool or system that alters the canvas MUST output a Command.
pub trait Command {
    fn execute(&self, graph: &mut AnimGraph);
    fn undo(&self, graph: &mut AnimGraph);
}

// --- SPECIFIC COMMAND PAYLOADS ---

/// Safely injects or removes a mathematical stroke sequence from a specific node.
pub struct AddStrokeCommand {
    pub target_node_id: NodeId,
    pub stroke_id: StrokeId,
    pub stroke: Stroke, 
}

impl Command for AddStrokeCommand {
    fn execute(&self, graph: &mut AnimGraph) {
        // AAA: We pass a clone of the stroke back into the graph when re-doing. 
        // This is memory-safe because the engine only clones the active payload, 
        // not the entire canvas history.
        graph.insert_stroke_by_id(self.target_node_id, self.stroke_id, self.stroke.clone());
        info!("Command Executed: Added Stroke [{}] to Node [{}]", self.stroke_id, self.target_node_id);
    }

    fn undo(&self, graph: &mut AnimGraph) {
        graph.remove_stroke_by_id(self.target_node_id, self.stroke_id);
        info!("Command Reversed: Removed Stroke [{}] from Node [{}]", self.stroke_id, self.target_node_id);
    }
}

// --- THE COMMAND CONTROLLER ---

/// The single source of truth for the engine's timeline.
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    next_stroke_id: StrokeId,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::with_capacity(512), // Pre-allocate memory to prevent runtime reallocation stutters
            redo_stack: Vec::with_capacity(512),
            next_stroke_id: 1, // 0 is reserved as a null-pointer safeguard
        }
    }

    /// Generates a globally unique identifier for new geometry.
    pub fn generate_stroke_id(&mut self) -> StrokeId {
        let id = self.next_stroke_id;
        self.next_stroke_id += 1;
        id
    }

    /// Pushes a new action onto the timeline and immediately executes it.
    /// This intentionally obliterates the redo stack, as the timeline has branched.
    pub fn push_and_execute(&mut self, command: Box<dyn Command>, graph: &mut AnimGraph) {
        command.execute(graph);
        self.undo_stack.push(command);
        self.redo_stack.clear(); 
    }

    pub fn undo(&mut self, graph: &mut AnimGraph) {
        if let Some(command) = self.undo_stack.pop() {
            command.undo(graph);
            self.redo_stack.push(command);
        } else {
            warn!("Undo blocked: Timeline is at the genesis state.");
        }
    }

    pub fn redo(&mut self, graph: &mut AnimGraph) {
        if let Some(command) = self.redo_stack.pop() {
            command.execute(graph);
            self.undo_stack.push(command);
        } else {
            warn!("Redo blocked: Timeline is at the current future state.");
        }
    }
}