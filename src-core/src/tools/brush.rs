use crate::tools::CanvasTool;
use crate::settings::EngineSettings;
use crate::graph::{SceneManager, IdAllocator};
use crate::command::{Command, AddStrokeCommand};
use crate::geometry::tessellator::Extruder;
use crate::geometry::{Point, ContourStroke, VectorElement};
use crate::math::Vertex;
use geo::LineString;
use wasm_bindgen::JsValue; // Needed to pass strings to the browser console

pub struct BrushTool { 
    points: Vec<Point>, 
    active_thickness: f32, 
    active_color: [f32; 4] 
}

impl BrushTool { 
    pub fn new() -> Self { 
        Self { 
            points: Vec::with_capacity(1024), 
            active_thickness: 10.0, 
            active_color: [1.0, 1.0, 1.0, 1.0] 
        } 
    } 
}

impl CanvasTool for BrushTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, settings: EngineSettings, _scene: &mut SceneManager, _id_allocator: &mut IdAllocator) {
        self.points.clear(); 
        self.active_thickness = settings.brush_thickness; 
        self.active_color = settings.brush_color;
        
        let pt = Point { x, y, pressure }; 
        if pt.is_valid() { 
            self.points.push(pt); 
        }

        // AAA DEBUG: Verify stroke initialization and base settings
        web_sys::console::log_1(&JsValue::from_str(
            &format!("[Brush Tool] DOWN -> Initialized stroke at X: {:.2}, Y: {:.2}. Thickness: {:.2}, Color: {:?}", x, y, self.active_thickness, self.active_color)
        ));
    }
    
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, _scene: &mut SceneManager, _cw: f32, _ch: f32) {
        let pt = Point { x, y, pressure }; 
        if pt.is_valid() { 
            self.points.push(pt); 
        }

        // AAA DEBUG: Sample move events to ensure tracking works, logging every 20th point to avoid console lag
        if self.points.len() % 20 == 0 {
            web_sys::console::log_1(&JsValue::from_str(
                &format!("[Brush Tool] MOVE -> Collected {} points so far. Current X: {:.2}, Y: {:.2}", self.points.len(), x, y)
            ));
        }
    }
    
    fn on_pointer_up(&mut self, id_allocator: &mut IdAllocator, cw: f32, ch: f32, scene: &mut SceneManager) -> Option<Box<dyn Command>> {
        // AAA DEBUG: Verify the final point count
        web_sys::console::log_1(&JsValue::from_str(
            &format!("[Brush Tool] UP -> Finalizing stroke with {} raw points.", self.points.len())
        ));

        if self.points.len() < 2 { 
            self.points.clear(); 
            web_sys::console::log_1(&JsValue::from_str("[Brush Tool] ABORT -> Not enough points to create a stroke."));
            return None; 
        }
        
        let settings = crate::settings::get_settings();
        let stroke_id = id_allocator.generate();
        
        // AAA INTEGRITY RESTORED: Utilizing your original smoothing pipeline
        let smoothed_points = crate::geometry::spline::smooth_spline(&self.points, settings.smoothing_level);
        let (vertices, indices, aabb) = Extruder::extrude_centerline(&smoothed_points, self.active_thickness, self.active_color, cw, ch);
        
        // AAA DEBUG: Crucial geometry verification. If Vertices/Indices are 0, the tessellator math is failing.
        web_sys::console::log_1(&JsValue::from_str(
            &format!("[Brush Tool] GENERATED -> Smoothed points: {}, Vertices: {}, Indices: {}, AABB: {:?}", 
            smoothed_points.len(), vertices.len(), indices.len(), aabb)
        ));

        let coords: Vec<geo::Coord<f32>> = smoothed_points.iter().map(|p| geo::coord!{ x: p.x, y: p.y }).collect();
        let shape = geo::MultiPolygon::new(vec![geo::Polygon::new(LineString(coords), vec![])]);
        
        let element = VectorElement::Contour(ContourStroke { shape, color: self.active_color, vertices, indices, aabb, eraser_masks: Vec::new(), clip_masks: Vec::new() });
        self.points.clear();

        let element_id = scene.active_element_id.unwrap_or(1);
        let drawing_id = *scene.elements.get(&element_id).unwrap().exposures.get(&scene.current_frame).unwrap_or(&1);
        
        // AAA DEBUG: Ensure the command is actually being built for the correct drawing
        web_sys::console::log_1(&JsValue::from_str(
            &format!("[Brush Tool] SUCCESS -> Pushing AddStrokeCommand for element_id: {}, drawing_id: {}", element_id, drawing_id)
        ));

        Some(Box::new(AddStrokeCommand { element_id, drawing_id, art_layer: scene.active_art_layer, stroke_id, element }))
    }
    
    fn get_preview_mesh(&self, cw: f32, ch: f32) -> (Vec<Vertex>, Vec<u16>) {
        if self.points.len() < 2 { return (Vec::new(), Vec::new()); }
        let settings = crate::settings::get_settings();
        let smoothed_points = crate::geometry::spline::smooth_spline(&self.points, settings.smoothing_level);
        let (verts, inds, _) = Extruder::extrude_centerline(&smoothed_points, self.active_thickness, self.active_color, cw, ch);
        
        // AAA DEBUG: Verify that the active preview mesh is generating vertices for the real-time render loop
        if self.points.len() % 30 == 0 {
            web_sys::console::log_1(&JsValue::from_str(
                &format!("[Brush Tool] PREVIEW MESH -> Outputting {} vertices and {} indices to WebGpuRenderer.", verts.len(), inds.len())
            ));
        }

        (verts, inds)
    }
}