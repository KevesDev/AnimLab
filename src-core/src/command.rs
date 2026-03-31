use crate::graph::{AnimGraph, NodeId, StrokeId};
use crate::geometry::VectorElement;

pub trait Command {
    fn execute(&self, graph: &mut AnimGraph, canvas_width: f32, canvas_height: f32);
    fn undo(&self, graph: &mut AnimGraph, canvas_width: f32, canvas_height: f32);
}

pub struct AddStrokeCommand {
    pub target_node_id: NodeId, pub stroke_id: StrokeId, pub element: VectorElement, 
}
impl Command for AddStrokeCommand {
    fn execute(&self, graph: &mut AnimGraph, _: f32, _: f32) { graph.insert_stroke_by_id(self.target_node_id, self.stroke_id, self.element.clone()); }
    fn undo(&self, graph: &mut AnimGraph, _: f32, _: f32) { graph.remove_stroke_by_id(self.target_node_id, self.stroke_id); }
}

pub struct CutCommand {
    pub target_node_id: NodeId, pub severed_stroke_id: StrokeId,
    pub original_element: VectorElement, pub new_fragments: Vec<(StrokeId, VectorElement)>,
}
impl Command for CutCommand {
    fn execute(&self, graph: &mut AnimGraph, _: f32, _: f32) {
        graph.remove_stroke_by_id(self.target_node_id, self.severed_stroke_id);
        for (frag_id, frag_element) in &self.new_fragments { graph.insert_stroke_by_id(self.target_node_id, *frag_id, frag_element.clone()); }
    }
    fn undo(&self, graph: &mut AnimGraph, _: f32, _: f32) {
        for (frag_id, _) in &self.new_fragments { graph.remove_stroke_by_id(self.target_node_id, *frag_id); }
        graph.insert_stroke_by_id(self.target_node_id, self.severed_stroke_id, self.original_element.clone());
    }
}

pub struct TransformCommand {
    pub target_node_id: NodeId, pub stroke_ids: Vec<StrokeId>, pub dx: f32, pub dy: f32,
}
impl Command for TransformCommand {
    fn execute(&self, graph: &mut AnimGraph, canvas_width: f32, canvas_height: f32) {
        if let Some(node) = graph.nodes.get_mut(&self.target_node_id) {
            if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                for id in &self.stroke_ids {
                    if let Some(element) = elements.get_mut(id) { element.translate(self.dx, self.dy, canvas_width, canvas_height); }
                }
            }
        }
    }
    fn undo(&self, graph: &mut AnimGraph, canvas_width: f32, canvas_height: f32) {
        if let Some(node) = graph.nodes.get_mut(&self.target_node_id) {
            if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                for id in &self.stroke_ids {
                    if let Some(element) = elements.get_mut(id) { element.translate(-self.dx, -self.dy, canvas_width, canvas_height); }
                }
            }
        }
    }
}

// AAA ARCHITECTURE: 100% Float-Drift-Proof Affine History state
pub struct AffineCommand {
    pub target_node_id: NodeId,
    pub old_elements: Vec<(StrokeId, VectorElement)>,
    pub new_elements: Vec<(StrokeId, VectorElement)>,
}
impl Command for AffineCommand {
    fn execute(&self, graph: &mut AnimGraph, _: f32, _: f32) {
        if let Some(node) = graph.nodes.get_mut(&self.target_node_id) {
            if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                for (id, el) in &self.new_elements { elements.insert(*id, el.clone()); }
            }
        }
    }
    fn undo(&self, graph: &mut AnimGraph, _: f32, _: f32) {
        if let Some(node) = graph.nodes.get_mut(&self.target_node_id) {
            if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                for (id, el) in &self.old_elements { elements.insert(*id, el.clone()); }
            }
        }
    }
}

pub struct BatchCommand { pub commands: Vec<Box<dyn Command>> }
impl Command for BatchCommand {
    fn execute(&self, graph: &mut AnimGraph, cw: f32, ch: f32) { for cmd in &self.commands { cmd.execute(graph, cw, ch); } }
    fn undo(&self, graph: &mut AnimGraph, cw: f32, ch: f32) { for cmd in self.commands.iter().rev() { cmd.undo(graph, cw, ch); } }
}

pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>, redo_stack: Vec<Box<dyn Command>>,
}
impl CommandHistory {
    pub fn new() -> Self { Self { undo_stack: Vec::with_capacity(512), redo_stack: Vec::with_capacity(512) } }
    pub fn push_and_execute(&mut self, command: Box<dyn Command>, graph: &mut AnimGraph, cw: f32, ch: f32) {
        command.execute(graph, cw, ch); self.undo_stack.push(command); self.redo_stack.clear(); 
    }
    pub fn undo(&mut self, graph: &mut AnimGraph, cw: f32, ch: f32) {
        if let Some(command) = self.undo_stack.pop() { command.undo(graph, cw, ch); self.redo_stack.push(command); } 
    }
    pub fn redo(&mut self, graph: &mut AnimGraph, cw: f32, ch: f32) {
        if let Some(command) = self.redo_stack.pop() { command.execute(graph, cw, ch); self.undo_stack.push(command); } 
    }
}