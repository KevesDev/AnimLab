pub mod spline;
pub mod tessellator;
pub mod boolean;

use crate::math::{Vertex, AABB};
use geo::MultiPolygon;

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
pub struct CenterlineStroke {
    pub points: Vec<Point>, 
    pub thickness: f32,     
    pub color: [f32; 4],
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub aabb: AABB,
}

#[derive(Debug, Clone)]
pub struct ContourStroke {
    pub shape: MultiPolygon<f32>, 
    pub color: [f32; 4],
    pub vertices: Vec<Vertex>, 
    pub indices: Vec<u16>,
    pub aabb: AABB,
}

#[derive(Debug, Clone)]
pub enum VectorElement {
    Centerline(CenterlineStroke),
    Contour(ContourStroke),
}

impl VectorElement {
    pub fn aabb(&self) -> &AABB {
        match self {
            VectorElement::Centerline(s) => &s.aabb,
            VectorElement::Contour(s) => &s.aabb,
        }
    }

    pub fn vertices(&self) -> &[Vertex] {
        match self {
            VectorElement::Centerline(s) => &s.vertices,
            VectorElement::Contour(s) => &s.vertices,
        }
    }

    pub fn indices(&self) -> &[u16] {
        match self {
            VectorElement::Centerline(s) => &s.indices,
            VectorElement::Contour(s) => &s.indices,
        }
    }
}