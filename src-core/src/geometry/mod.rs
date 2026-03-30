pub mod spline;
pub mod tessellator;
pub mod boolean;

use crate::math::{Vertex, AABB};
use geo::{MultiPolygon, Translate};

#[derive(Debug, Clone, Copy)]
pub struct Point { pub x: f32, pub y: f32, pub pressure: f32 }

impl Point {
    pub fn is_valid(&self) -> bool { self.x.is_finite() && self.y.is_finite() && self.pressure.is_finite() }
}

#[derive(Debug, Clone)]
pub struct CenterlineStroke {
    pub points: Vec<Point>, pub thickness: f32, pub color: [f32; 4],
    pub vertices: Vec<Vertex>, pub indices: Vec<u16>, pub aabb: AABB,
}

#[derive(Debug, Clone)]
pub struct EraserMask { pub vertices: Vec<Vertex>, pub indices: Vec<u16> }

#[derive(Debug, Clone)]
pub struct ContourStroke {
    pub shape: MultiPolygon<f32>, pub color: [f32; 4],
    pub vertices: Vec<Vertex>, pub indices: Vec<u16>, pub aabb: AABB,
    pub eraser_masks: Vec<EraserMask>, 
    pub clip_masks: Vec<EraserMask>,   
}

#[derive(Debug, Clone)]
pub enum VectorElement {
    Centerline(CenterlineStroke),
    Contour(ContourStroke),
}

impl VectorElement {
    pub fn aabb(&self) -> &AABB { match self { VectorElement::Centerline(s) => &s.aabb, VectorElement::Contour(s) => &s.aabb } }
    pub fn vertices(&self) -> &[Vertex] { match self { VectorElement::Centerline(s) => &s.vertices, VectorElement::Contour(s) => &s.vertices } }
    pub fn indices(&self) -> &[u16] { match self { VectorElement::Centerline(s) => &s.indices, VectorElement::Contour(s) => &s.indices } }

    /// AAA Architecture: O(N) Affine Translation. 
    /// Instantly moves the vector data and GPU buffers without triggering a heavy CPU re-tessellation.
    pub fn translate(&mut self, dx: f32, dy: f32, canvas_width: f32, canvas_height: f32) {
        let clip_dx = (dx / canvas_width) * 2.0;
        let clip_dy = -(dy / canvas_height) * 2.0; // WebGPU Y-axis is inverted

        match self {
            VectorElement::Centerline(s) => {
                for pt in &mut s.points { pt.x += dx; pt.y += dy; }
                for v in &mut s.vertices { v.position[0] += clip_dx; v.position[1] += clip_dy; }
                s.aabb.min_x += dx; s.aabb.max_x += dx;
                s.aabb.min_y += dy; s.aabb.max_y += dy;
            },
            VectorElement::Contour(s) => {
                s.shape.translate_mut(dx, dy);
                for v in &mut s.vertices { v.position[0] += clip_dx; v.position[1] += clip_dy; }
                for mask in &mut s.eraser_masks { for v in &mut mask.vertices { v.position[0] += clip_dx; v.position[1] += clip_dy; } }
                for mask in &mut s.clip_masks { for v in &mut mask.vertices { v.position[0] += clip_dx; v.position[1] += clip_dy; } }
                s.aabb.min_x += dx; s.aabb.max_x += dx;
                s.aabb.min_y += dy; s.aabb.max_y += dy;
            }
        }
    }
}