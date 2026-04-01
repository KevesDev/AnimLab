use crate::graph::{SceneManager, IdAllocator};
use crate::command::{CommandHistory, CutCommand, BatchCommand, AddStrokeCommand, AffineCommand};
use crate::geometry::VectorElement;

pub fn select_all(scene: &mut SceneManager) {
    scene.selected_strokes.clear();
    let ids: Vec<_> = if let Some((_, layer)) = scene.get_active_art_layer() { 
        layer.vector_elements.keys().copied().collect() 
    } else { 
        Vec::new() 
    };
    for id in ids { scene.selected_strokes.insert(id); }
}

pub fn flip_selection(scene: &mut SceneManager, history: &mut CommandHistory, canvas_width: f32, canvas_height: f32, flip_h: bool, flip_v: bool) {
    if scene.selected_strokes.is_empty() { return; }
    let element_id = scene.active_element_id.unwrap_or(1);
    let drawing_id = scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).copied().unwrap_or(1);

    if let Some(aabb) = scene.get_selection_aabb() {
        let cx = aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0; 
        let cy = aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0;
        let sx = if flip_h { -1.0 } else { 1.0 }; 
        let sy = if flip_v { -1.0 } else { 1.0 };
        let mut old_elements = Vec::new(); 
        let mut new_elements = Vec::new();

        let selected: Vec<_> = scene.selected_strokes.iter().copied().collect();

        if let Some((_, layer)) = scene.get_active_art_layer_mut() {
            for id in &selected {
                if let Some(el) = layer.vector_elements.get(id) {
                    old_elements.push((*id, el.clone()));
                    let mut new_el = el.clone(); 
                    new_el.transform(0.0, 0.0, sx, sy, 0.0, cx, cy, canvas_width, canvas_height);
                    new_elements.push((*id, new_el.clone())); 
                    layer.vector_elements.insert(*id, new_el);
                }
            }
        }
        if !new_elements.is_empty() {
            let cmd = Box::new(AffineCommand { element_id, drawing_id, art_layer: scene.active_art_layer, old_elements, new_elements });
            history.push_and_execute(cmd, scene, canvas_width, canvas_height);
        }
    }
}

pub fn delete_selection(scene: &mut SceneManager, history: &mut CommandHistory, canvas_width: f32, canvas_height: f32) {
    if scene.selected_strokes.is_empty() { return; }
    let element_id = scene.active_element_id.unwrap_or(1);
    let drawing_id = scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).copied().unwrap_or(1);
    let mut severed_fragments = Vec::new();
    
    let selected: Vec<_> = scene.selected_strokes.iter().copied().collect();
    if let Some((_, layer)) = scene.get_active_art_layer() {
        for id in &selected { 
            if let Some(el) = layer.vector_elements.get(id) { severed_fragments.push((*id, el.clone())); } 
        }
    }
    
    for (stroke_id, original_element) in severed_fragments {
        let cmd = Box::new(CutCommand { element_id, drawing_id, art_layer: scene.active_art_layer, severed_stroke_id: stroke_id, original_element, new_fragments: Vec::new() });
        history.push_and_execute(cmd, scene, canvas_width, canvas_height);
    }
    scene.selected_strokes.clear(); 
}

pub fn copy_selection(scene: &SceneManager, clipboard: &mut Vec<VectorElement>) {
    clipboard.clear();
    let selected: Vec<_> = scene.selected_strokes.iter().copied().collect();
    if let Some((_, layer)) = scene.get_active_art_layer() {
        for id in &selected { 
            if let Some(el) = layer.vector_elements.get(id) { clipboard.push(el.clone()); } 
        }
    }
}

pub fn paste_clipboard(scene: &mut SceneManager, history: &mut CommandHistory, allocator: &mut IdAllocator, clipboard: &[VectorElement], canvas_width: f32, canvas_height: f32) {
    if clipboard.is_empty() { return; }
    scene.ensure_drawing_exists(allocator);
    
    let element_id = scene.active_element_id.unwrap_or(1);
    let drawing_id = scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).copied().unwrap_or(1);
    let mut commands: Vec<Box<dyn crate::command::Command>> = Vec::new();
    
    scene.selected_strokes.clear();
    for el in clipboard {
        let new_id = allocator.generate();
        scene.selected_strokes.insert(new_id);
        let mut pasted_el = el.clone(); 
        pasted_el.translate(20.0, 20.0, canvas_width, canvas_height); 
        commands.push(Box::new(AddStrokeCommand { element_id, drawing_id, art_layer: scene.active_art_layer, stroke_id: new_id, element: pasted_el }));
    }
    let batch = Box::new(BatchCommand { commands });
    history.push_and_execute(batch, scene, canvas_width, canvas_height); 
}