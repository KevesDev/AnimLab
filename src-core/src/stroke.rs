use log::warn;
use crate::math::{Vertex, Tessellator};
use crate::settings; 

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

impl Point {
    pub fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.pressure.is_finite()
    }
}

#[derive(Debug)]
pub struct Stroke {
    pub points: Vec<Point>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Stroke {
    pub fn new() -> Self {
        Self {
            points: Vec::with_capacity(256),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_point(&mut self, x: f32, y: f32, pressure: f32) {
        let pt = Point { x, y, pressure };
        if pt.is_valid() {
            self.points.push(pt);
        } else {
            warn!("Hardware injected invalid point data (NaN/Inf). Point explicitly discarded.");
        }
    }

    /// Compiles the raw mathematical points into a renderable GPU mesh.
    pub fn build_mesh(&mut self, canvas_width: f32, canvas_height: f32) {
        let current_settings = settings::get_settings();
        
        // Execute the Catmull-Rom algorithm to inject curvature data between raw hardware frames
        let smoothed_points = crate::math::smooth_points(&self.points, current_settings.smoothing_level);
        
        // Pass the dynamically smoothed point array to the GPU memory architect
        let (verts, inds) = Tessellator::extrude_stroke(
            &smoothed_points, 
            current_settings.brush_thickness, 
            current_settings.brush_color, 
            canvas_width, 
            canvas_height
        );
        
        self.vertices = verts;
        self.indices = inds;
    }
}