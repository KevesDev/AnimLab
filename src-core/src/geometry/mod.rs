pub mod spline;
pub mod tessellator;
pub mod boolean;

use crate::math::{Vertex, AABB};
use geo::{MultiPolygon, MapCoords};

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
pub struct EraserMask { 
    pub shape: MultiPolygon<f32>, 
    pub vertices: Vec<Vertex>, 
    pub indices: Vec<u16> 
}

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

    pub fn translate(&mut self, dx: f32, dy: f32, canvas_width: f32, canvas_height: f32) {
        self.transform(dx, dy, 1.0, 1.0, 0.0, 0.0, 0.0, canvas_width, canvas_height);
    }

    // AAA ARCHITECTURE: 2D Affine Transformation Matrix.
    // Handles Scaling, Translating, Rotating, and Flipping symmetrically around any target pivot.
    pub fn transform(&mut self, dx: f32, dy: f32, scale_x: f32, scale_y: f32, rotation: f32, pivot_x: f32, pivot_y: f32, canvas_width: f32, canvas_height: f32) {
        let cos_a = rotation.cos();
        let sin_a = rotation.sin();

        let transform_pt = |x: f32, y: f32| -> (f32, f32) {
            let tx = x - pivot_x; let ty = y - pivot_y;
            let sx = tx * scale_x; let sy = ty * scale_y;
            let rx = sx * cos_a - sy * sin_a;
            let ry = sx * sin_a + sy * cos_a;
            (rx + pivot_x + dx, ry + pivot_y + dy)
        };

        let transform_ndc = |x: f32, y: f32| -> [f32; 2] {
            let px = (x + 1.0) / 2.0 * canvas_width;
            let py = (1.0 - y) / 2.0 * canvas_height;
            let (nx, ny) = transform_pt(px, py);
            let ndc_x = (nx / canvas_width) * 2.0 - 1.0;
            let ndc_y = 1.0 - (ny / canvas_height) * 2.0;
            [ndc_x, ndc_y]
        };

        match self {
            VectorElement::Centerline(s) => {
                for pt in &mut s.points {
                    let (nx, ny) = transform_pt(pt.x, pt.y);
                    pt.x = nx; pt.y = ny;
                }
                for v in &mut s.vertices { v.position = transform_ndc(v.position[0], v.position[1]); }
                s.aabb = AABB::empty();
                for pt in &s.points { s.aabb.expand_to_include(pt.x, pt.y, s.thickness / 2.0); }
            },
            VectorElement::Contour(s) => {
                s.shape = s.shape.map_coords(|c| {
                    let (nx, ny) = transform_pt(c.x, c.y);
                    geo::Coord { x: nx, y: ny }
                });
                for v in &mut s.vertices { v.position = transform_ndc(v.position[0], v.position[1]); }
                
                for mask in &mut s.eraser_masks { 
                    mask.shape = mask.shape.map_coords(|c| { 
                        let (nx, ny) = transform_pt(c.x, c.y); 
                        geo::Coord { x: nx, y: ny } 
                    });
                    for v in &mut mask.vertices { v.position = transform_ndc(v.position[0], v.position[1]); } 
                }
                for mask in &mut s.clip_masks { 
                    mask.shape = mask.shape.map_coords(|c| { 
                        let (nx, ny) = transform_pt(c.x, c.y); 
                        geo::Coord { x: nx, y: ny } 
                    });
                    for v in &mut mask.vertices { v.position = transform_ndc(v.position[0], v.position[1]); } 
                }
                
                // AAA FIX: Preserve the boolean slice shrink-wrap by transforming the existing AABB corners
                // instead of rebuilding it from the raw un-masked vertices.
                if s.aabb.min_x <= s.aabb.max_x {
                    let corners = [
                        (s.aabb.min_x, s.aabb.min_y),
                        (s.aabb.max_x, s.aabb.min_y),
                        (s.aabb.max_x, s.aabb.max_y),
                        (s.aabb.min_x, s.aabb.max_y),
                    ];
                    s.aabb = AABB::empty();
                    for (cx, cy) in corners {
                        let (nx, ny) = transform_pt(cx, cy);
                        s.aabb.expand_to_include(nx, ny, 0.0);
                    }
                }
            }
        }
    }
}