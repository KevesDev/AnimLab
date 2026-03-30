use log::warn;
use crate::math::{Vertex, Tessellator, AABB};

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

#[derive(Debug, Clone)]
pub struct Stroke {
    pub points: Vec<Point>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub aabb: AABB,
}

impl Stroke {
    pub fn new() -> Self {
        Self {
            points: Vec::with_capacity(256),
            vertices: Vec::new(),
            indices: Vec::new(),
            aabb: AABB::empty(),
        }
    }

    pub fn add_point(&mut self, x: f32, y: f32, pressure: f32) {
        let pt = Point { x, y, pressure };
        if pt.is_valid() {
            self.points.push(pt);
        } else {
            warn!("Hardware injected invalid point data. Point explicitly discarded.");
        }
    }

    // AAA FIX: Settings are explicitly injected from the Tool's snapshot.
    pub fn build_mesh(&mut self, base_thickness: f32, color: [f32; 4], smoothing: f32, canvas_width: f32, canvas_height: f32) {
        let smoothed_points = crate::math::smooth_points(&self.points, smoothing);
        
        let mut bounds = AABB::empty();
        let max_radius = base_thickness / 2.0; 
        for pt in &smoothed_points {
            bounds.expand_to_include(pt.x, pt.y, max_radius);
        }
        self.aabb = bounds;

        let (verts, inds) = Tessellator::extrude_stroke(
            &smoothed_points, 
            base_thickness, 
            color, 
            canvas_width, 
            canvas_height
        );
        self.vertices = verts;
        self.indices = inds;
    }
}