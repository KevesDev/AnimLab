use log::{info, warn};
use crate::graph::{AnimGraph, NodeId, StrokeId};
use crate::geometry::VectorElement;

pub trait Command {
    fn execute(&self, graph: &mut AnimGraph);
    fn undo(&self, graph: &mut AnimGraph);
}

pub struct AddStrokeCommand {
    pub target_node_id: NodeId, pub stroke_id: StrokeId, pub element: VectorElement, 
}
impl Command for AddStrokeCommand {
    fn execute(&self, graph: &mut AnimGraph) { graph.insert_stroke_by_id(self.target_node_id, self.stroke_id, self.element.clone()); }
    fn undo(&self, graph: &mut AnimGraph) { graph.remove_stroke_by_id(self.target_node_id, self.stroke_id); }
}

pub struct CutCommand {
    pub target_node_id: NodeId, pub severed_stroke_id: StrokeId,
    pub original_element: VectorElement, pub new_fragments: Vec<(StrokeId, VectorElement)>,
}
impl Command for CutCommand {
    fn execute(&self, graph: &mut AnimGraph) {
        graph.remove_stroke_by_id(self.target_node_id, self.severed_stroke_id);
        for (frag_id, frag_element) in &self.new_fragments { graph.insert_stroke_by_id(self.target_node_id, *frag_id, frag_element.clone()); }
        info!("CutCommand Executed: Severed Stroke [{}] into {} fragments.", self.severed_stroke_id, self.new_fragments.len());
    }
    fn undo(&self, graph: &mut AnimGraph) {
        for (frag_id, _) in &self.new_fragments { graph.remove_stroke_by_id(self.target_node_id, *frag_id); }
        graph.insert_stroke_by_id(self.target_node_id, self.severed_stroke_id, self.original_element.clone());
        info!("CutCommand Reversed: Restored Stroke [{}].", self.severed_stroke_id);
    }
}

pub struct BatchCommand {
    pub commands: Vec<Box<dyn Command>>,
}
impl Command for BatchCommand {
    fn execute(&self, graph: &mut AnimGraph) { for cmd in &self.commands { cmd.execute(graph); } }
    fn undo(&self, graph: &mut AnimGraph) { for cmd in self.commands.iter().rev() { cmd.undo(graph); } }
}

pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}
impl CommandHistory {
    pub fn new() -> Self { Self { undo_stack: Vec::with_capacity(512), redo_stack: Vec::with_capacity(512) } }
    pub fn push_and_execute(&mut self, command: Box<dyn Command>, graph: &mut AnimGraph) {
        command.execute(graph); self.undo_stack.push(command); self.redo_stack.clear(); 
    }
    pub fn undo(&mut self, graph: &mut AnimGraph) {
        if let Some(command) = self.undo_stack.pop() { command.undo(graph); self.redo_stack.push(command); } 
        else { warn!("Undo blocked: Timeline is at genesis."); }
    }
    pub fn redo(&mut self, graph: &mut AnimGraph) {
        if let Some(command) = self.redo_stack.pop() { command.execute(graph); self.undo_stack.push(command); } 
        else { warn!("Redo blocked: Timeline is at current future state."); }
    }
}