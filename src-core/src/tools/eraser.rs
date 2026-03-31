use crate::tools::CanvasTool;
use crate::geometry::Point;
use crate::geometry::boolean::BooleanSlicer;
use crate::geometry::spline::smooth_spline;
use crate::geometry::tessellator::Extruder;
use crate::settings::EngineSettings;
use crate::math::AABB;
use crate::graph::{SceneManager, IdAllocator, StrokeId};
use crate::command::{Command, CutCommand, BatchCommand};
use crate::math::Vertex;
use crate::geometry::{VectorElement, EraserMask};
use std::collections::HashMap;

pub struct EraserTool { 
    raw_points: Vec<Point>, 
    settings_snapshot: Option<EngineSettings>,
    original_elements: HashMap<StrokeId, VectorElement>,
    temp_fragment_ids: Vec<StrokeId>,
}

impl EraserTool { 
    pub fn new() -> Self { 
        Self { 
            raw_points: Vec::with_capacity(256), 
            settings_snapshot: None,
            original_elements: HashMap::new(),
            temp_fragment_ids: Vec::new(),
        } 
    } 
}

impl CanvasTool for EraserTool {
    fn get_cursor(&self) -> &'static str { "cell" }

    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, settings: EngineSettings, scene: &mut SceneManager, _id_allocator: &mut IdAllocator) {
        self.settings_snapshot = Some(settings); 
        self.raw_points.clear();
        self.original_elements.clear();
        self.temp_fragment_ids.clear();
        
        let pt = Point { x, y, pressure }; 
        if pt.is_valid() { self.raw_points.push(pt); }
        
        // Snapshot the active layer so we can non-destructively mask it during the stroke
        if let Some((_, layer)) = scene.get_active_art_layer() {
            for (id, el) in &layer.vector_elements {
                self.original_elements.insert(*id, el.clone());
            }
        }
    }
    
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, scene: &mut SceneManager, canvas_width: f32, canvas_height: f32) {
        let pt = Point { x, y, pressure }; 
        if pt.is_valid() { self.raw_points.push(pt); }
        
        if self.raw_points.len() < 2 { return; }
        
        let settings = self.settings_snapshot.as_ref().unwrap();
        let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
        
        // Generate the high-performance WebGPU Stencil Mask
        let (vertices, indices, sweep_aabb) = Extruder::extrude_centerline(&smoothed, settings.brush_thickness, [1.0; 4], canvas_width, canvas_height);
        let mask = EraserMask { shape: geo::MultiPolygon::new(vec![]), vertices, indices };
        
        if let Some((_, layer)) = scene.get_active_art_layer_mut() {
            // Clean up temporary rendering fragments from the previous frame
            for tid in &self.temp_fragment_ids { layer.vector_elements.remove(tid); }
            self.temp_fragment_ids.clear();
            
            let mut new_temps = Vec::new();
            
            for (id, orig_el) in &self.original_elements {
                if !orig_el.aabb().intersects(&sweep_aabb) {
                    layer.vector_elements.insert(*id, orig_el.clone());
                    continue;
                }
                
                match orig_el {
                    VectorElement::Contour(_) => {
                        // AAA FEATURE: Stencil Masking. The GPU effortlessly hides the pixels without altering CPU math.
                        let mut temp_el = orig_el.clone();
                        if let VectorElement::Contour(c) = &mut temp_el {
                            c.eraser_masks.push(mask.clone());
                        }
                        layer.vector_elements.insert(*id, temp_el);
                    },
                    VectorElement::Centerline(_) => {
                        // AAA FEATURE: Fast CPU Splitting. Pencils have no volume, so we split them on-the-fly for the preview.
                        let frags = BooleanSlicer::slice_element(orig_el, &self.raw_points, settings.brush_thickness, canvas_width, canvas_height, settings.smoothing_level);
                        layer.vector_elements.remove(id); // Hide the original
                        
                        for (i, frag) in frags.into_iter().enumerate() {
                            let tid = *id + 1_000_000 + i as u64; // Assign an ultra-high temporary ID
                            layer.vector_elements.insert(tid, frag);
                            new_temps.push(tid);
                        }
                    }
                }
            }
            self.temp_fragment_ids = new_temps;
        }
    }

    fn on_pointer_up(
        &mut self, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, scene: &mut SceneManager
    ) -> Option<Box<dyn Command>> {
        if let Some(settings) = self.settings_snapshot.take() {
            if self.raw_points.len() < 2 { return None; }
            
            // 1. Restore the layer to its pure, original state
            if let Some((_, layer)) = scene.get_active_art_layer_mut() {
                for tid in &self.temp_fragment_ids { layer.vector_elements.remove(tid); }
                for (id, el) in &self.original_elements { layer.vector_elements.insert(*id, el.clone()); }
            }
            self.temp_fragment_ids.clear();
            self.original_elements.clear();

            // 2. Determine which elements actually require a permanent mathematical cut
            let smoothed = smooth_spline(&self.raw_points, settings.smoothing_level);
            let mut sweep_aabb = AABB::empty();
            let max_r = settings.brush_thickness / 2.0;
            for pt in &smoothed { sweep_aabb.expand_to_include(pt.x, pt.y, max_r); }

            let element_id = scene.active_element_id.unwrap_or(1);
            let drawing_id = scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).copied().unwrap_or(1);
            let art_layer = scene.active_art_layer;

            let mut to_cut = Vec::new();
            if let Some((_, layer)) = scene.get_active_art_layer() {
                for (id, element) in &layer.vector_elements {
                    if element.aabb().intersects(&sweep_aabb) {
                        to_cut.push((*id, element.clone()));
                    }
                }
            }

            // 3. Execute the permanent boolean slice and push to History
            let mut batch_commands = Vec::new();

            for (stroke_id, original_element) in to_cut {
                let new_fragments = BooleanSlicer::slice_element(
                    &original_element, &self.raw_points, settings.brush_thickness, canvas_width, canvas_height, settings.smoothing_level
                );
                
                let mut new_fragments_with_ids = Vec::new();
                for frag in new_fragments { 
                    new_fragments_with_ids.push((id_allocator.generate(), frag)); 
                }

                batch_commands.push(Box::new(CutCommand {
                    element_id, drawing_id, art_layer, 
                    severed_stroke_id: stroke_id, 
                    original_element, 
                    new_fragments: new_fragments_with_ids,
                }) as Box<dyn Command>);
            }

            self.raw_points.clear();
            if !batch_commands.is_empty() { return Some(Box::new(BatchCommand { commands: batch_commands })); }
        }
        None
    }

    fn get_preview_mesh(&self, _canvas_width: f32, _canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        // AAA FIX: Returns completely empty. The GPU Stencil Buffer visually handles the erasure perfectly.
        (Vec::new(), Vec::new())
    }
}