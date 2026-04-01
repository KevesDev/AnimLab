use crate::tools::CanvasTool;
use crate::settings::EngineSettings;
use crate::graph::{SceneManager, IdAllocator};
use crate::command::{Command, AddStrokeCommand};
use crate::geometry::tessellator::Extruder;
use crate::geometry::{Point, ContourStroke, VectorElement};
use crate::math::Vertex;
use geo::LineString;

pub struct BrushTool { 
    points: Vec<Point>, 
    active_thickness: f32, 
    active_color: [f32; 4] 
}

impl BrushTool { 
    pub fn new() -> Self { 
        // AAA FIX: Initialize with the actual engine settings, not hardcoded white
        let settings = crate::settings::get_settings();
        Self { 
            points: Vec::with_capacity(1024), 
            active_thickness: settings.brush_thickness, 
            active_color: settings.brush_color 
        } 
    } 
}

impl CanvasTool for BrushTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, settings: EngineSettings, _scene: &mut SceneManager, _id_allocator: &mut IdAllocator) {
        self.points.clear(); 
        self.active_thickness = settings.brush_thickness; 
        self.active_color = settings.brush_color;
        
        let pt = Point { x, y, pressure }; 
        if pt.is_valid() { self.points.push(pt); }
    }
    
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _constrain: bool, _center: bool, _scene: &mut SceneManager, _cw: f32, _ch: f32) {
        let pt = Point { x, y, pressure }; 
        if pt.is_valid() { self.points.push(pt); }
    }
    
    fn on_pointer_up(&mut self, id_allocator: &mut IdAllocator, cw: f32, ch: f32, scene: &mut SceneManager) -> Option<Box<dyn Command>> {
        if self.points.len() < 2 { 
            self.points.clear(); 
            return None; 
        }
        
        let settings = crate::settings::get_settings();
        let stroke_id = id_allocator.generate();
        
        let smoothed_points = crate::geometry::spline::smooth_spline(&self.points, settings.smoothing_level);
        let (vertices, indices, aabb) = Extruder::extrude_centerline(&smoothed_points, self.active_thickness, self.active_color, cw, ch);
        
        let coords: Vec<geo::Coord<f32>> = smoothed_points.iter().map(|p| geo::coord!{ x: p.x, y: p.y }).collect();
        let shape = geo::MultiPolygon::new(vec![geo::Polygon::new(LineString(coords), vec![])]);
        
        let element = VectorElement::Contour(ContourStroke { shape, color: self.active_color, vertices, indices, aabb, eraser_masks: Vec::new(), clip_masks: Vec::new() });
        self.points.clear();

        let element_id = scene.active_element_id.unwrap_or(1);
        let drawing_id = scene.elements.get(&element_id).unwrap().get_exposure_id(scene.current_frame).unwrap_or(1);
        
        Some(Box::new(AddStrokeCommand { element_id, drawing_id, art_layer: scene.active_art_layer, stroke_id, element }))
    }
    
    fn get_preview_mesh(&self, cw: f32, ch: f32) -> (Vec<Vertex>, Vec<u16>) {
        if self.points.len() < 2 { return (Vec::new(), Vec::new()); }
        let settings = crate::settings::get_settings();
        let smoothed_points = crate::geometry::spline::smooth_spline(&self.points, settings.smoothing_level);
        let (verts, inds, _) = Extruder::extrude_centerline(&smoothed_points, self.active_thickness, self.active_color, cw, ch);
        (verts, inds)
    }
}