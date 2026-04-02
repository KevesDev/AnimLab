use crate::graph::{SceneManager, ElementId, DrawingId, ArtLayerType, StrokeId};
use crate::geometry::VectorElement;

pub trait Command {
    fn execute(&self, scene: &mut SceneManager, canvas_width: f32, canvas_height: f32);
    fn undo(&self, scene: &mut SceneManager, canvas_width: f32, canvas_height: f32);
}

pub struct AddStrokeCommand {
    pub element_id: ElementId, pub drawing_id: DrawingId, pub art_layer: ArtLayerType,
    pub stroke_id: StrokeId, pub element: VectorElement, 
}
impl Command for AddStrokeCommand {
    fn execute(&self, scene: &mut SceneManager, _: f32, _: f32) { 
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if let Some(drawing) = el.library.get_mut(&self.drawing_id) {
                drawing.get_art_layer_mut(self.art_layer).vector_elements.insert(self.stroke_id, self.element.clone());
            }
        }
    }
    fn undo(&self, scene: &mut SceneManager, _: f32, _: f32) { 
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if let Some(drawing) = el.library.get_mut(&self.drawing_id) {
                drawing.get_art_layer_mut(self.art_layer).vector_elements.remove(&self.stroke_id);
            }
        }
    }
}

pub struct CutCommand {
    pub element_id: ElementId, pub drawing_id: DrawingId, pub art_layer: ArtLayerType,
    pub severed_stroke_id: StrokeId, pub original_element: VectorElement, 
    pub new_fragments: Vec<(StrokeId, VectorElement)>,
}
impl Command for CutCommand {
    fn execute(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if let Some(drawing) = el.library.get_mut(&self.drawing_id) {
                let layer = drawing.get_art_layer_mut(self.art_layer);
                layer.vector_elements.remove(&self.severed_stroke_id);
                for (frag_id, frag_element) in &self.new_fragments { layer.vector_elements.insert(*frag_id, frag_element.clone()); }
            }
        }
    }
    fn undo(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if let Some(drawing) = el.library.get_mut(&self.drawing_id) {
                let layer = drawing.get_art_layer_mut(self.art_layer);
                for (frag_id, _) in &self.new_fragments { layer.vector_elements.remove(frag_id); }
                layer.vector_elements.insert(self.severed_stroke_id, self.original_element.clone());
            }
        }
    }
}

pub struct AffineCommand {
    pub element_id: ElementId, pub drawing_id: DrawingId, pub art_layer: ArtLayerType,
    pub old_elements: Vec<(StrokeId, VectorElement)>,
    pub new_elements: Vec<(StrokeId, VectorElement)>,
}
impl Command for AffineCommand {
    fn execute(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if let Some(drawing) = el.library.get_mut(&self.drawing_id) {
                let layer = drawing.get_art_layer_mut(self.art_layer);
                for (id, el) in &self.new_elements { layer.vector_elements.insert(*id, el.clone()); }
            }
        }
    }
    fn undo(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if let Some(drawing) = el.library.get_mut(&self.drawing_id) {
                let layer = drawing.get_art_layer_mut(self.art_layer);
                for (id, el) in &self.old_elements { layer.vector_elements.insert(*id, el.clone()); }
            }
        }
    }
}

pub struct BatchCommand { pub commands: Vec<Box<dyn Command>> }
impl Command for BatchCommand {
    fn execute(&self, scene: &mut SceneManager, cw: f32, ch: f32) { for cmd in &self.commands { cmd.execute(scene, cw, ch); } }
    fn undo(&self, scene: &mut SceneManager, cw: f32, ch: f32) { for cmd in self.commands.iter().rev() { cmd.undo(scene, cw, ch); } }
}

pub struct LayerCommand {
    pub element_id: ElementId, pub element: crate::graph::DrawingElement, pub is_add: bool,
}
impl Command for LayerCommand {
    fn execute(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if self.is_add {
            scene.elements.insert(self.element_id, self.element.clone());
            if !scene.z_stack.contains(&self.element_id) { scene.z_stack.push(self.element_id); }
        } else {
            scene.elements.remove(&self.element_id);
            scene.z_stack.retain(|&id| id != self.element_id);
        }
    }
    fn undo(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if self.is_add {
            scene.elements.remove(&self.element_id);
            scene.z_stack.retain(|&id| id != self.element_id);
        } else {
            scene.elements.insert(self.element_id, self.element.clone());
            if !scene.z_stack.contains(&self.element_id) { scene.z_stack.push(self.element_id); }
        }
    }
}

pub struct ToggleLayerStateCommand { pub element_id: ElementId, pub toggle_visibility: bool }
impl Command for ToggleLayerStateCommand {
    fn execute(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if self.toggle_visibility { el.is_visible = !el.is_visible; } else { el.is_locked = !el.is_locked; }
        }
    }
    fn undo(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if let Some(el) = scene.elements.get_mut(&self.element_id) {
            if self.toggle_visibility { el.is_visible = !el.is_visible; } else { el.is_locked = !el.is_locked; }
        }
    }
}

pub struct ReorderLayerCommand { pub element_id: ElementId, pub old_index: usize, pub new_index: usize }
impl Command for ReorderLayerCommand {
    fn execute(&self, scene: &mut SceneManager, _: f32, _: f32) {
        if self.old_index < scene.z_stack.len() {
            let id = scene.z_stack.remove(self.old_index);
            let safe_new = self.new_index.min(scene.z_stack.len());
            scene.z_stack.insert(safe_new, id);
        }
    }
    fn undo(&self, scene: &mut SceneManager, _: f32, _: f32) {
        let safe_new = self.new_index.min(scene.z_stack.len() - 1);
        if safe_new < scene.z_stack.len() {
            let id = scene.z_stack.remove(safe_new);
            let safe_old = self.old_index.min(scene.z_stack.len());
            scene.z_stack.insert(safe_old, id);
        }
    }
}

pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>, redo_stack: Vec<Box<dyn Command>>,
}
impl CommandHistory {
    pub fn new() -> Self { Self { undo_stack: Vec::with_capacity(512), redo_stack: Vec::with_capacity(512) } }
    pub fn push_and_execute(&mut self, command: Box<dyn Command>, scene: &mut SceneManager, cw: f32, ch: f32) {
        command.execute(scene, cw, ch); self.undo_stack.push(command); self.redo_stack.clear(); 
    }
    pub fn undo(&mut self, scene: &mut SceneManager, cw: f32, ch: f32) {
        if let Some(command) = self.undo_stack.pop() { command.undo(scene, cw, ch); self.redo_stack.push(command); } 
    }
    pub fn redo(&mut self, scene: &mut SceneManager, cw: f32, ch: f32) {
        if let Some(command) = self.redo_stack.pop() { command.execute(scene, cw, ch); self.undo_stack.push(command); } 
    }
}