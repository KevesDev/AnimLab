use super::{Point, VectorElement, CenterlineStroke, ContourStroke};
use crate::geometry::tessellator::Extruder;
use geo::{Polygon, MultiPolygon, LineString, Coord, BooleanOps, Contains};

pub struct BooleanSlicer;

impl BooleanSlicer {
    /// Unions every frame of the eraser path into one solid shape to prevent trailing artifacts.
    pub fn build_eraser_sweep(points: &[Point], base_thickness: f32) -> MultiPolygon<f32> {
        let mut total_sweep = MultiPolygon::new(vec![]);
        if points.is_empty() { return total_sweep; }
        
        let create_circle = |x: f32, y: f32, r: f32| -> Polygon<f32> {
            let steps = 12; let mut coords = Vec::with_capacity(steps + 1);
            for i in 0..=steps {
                let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
                coords.push(Coord { x: x + angle.cos() * r, y: y + angle.sin() * r });
            }
            Polygon::new(LineString::new(coords), vec![])
        };

        let r_start = (base_thickness * points[0].pressure) / 2.0;
        total_sweep = total_sweep.union(&create_circle(points[0].x, points[0].y, r_start));

        for i in 1..points.len() {
            let p1 = &points[i-1]; let p2 = &points[i];
            let r1 = (base_thickness * p1.pressure) / 2.0;
            let r2 = (base_thickness * p2.pressure) / 2.0;
            let dx = p2.x - p1.x; let dy = p2.y - p1.y;
            let length = (dx * dx + dy * dy).sqrt();
            
            if length > 0.01 {
                let nx = -dy / length; let ny = dx / length;
                let poly = Polygon::new(LineString::new(vec![
                    Coord { x: p1.x + nx * r1, y: p1.y + ny * r1 },
                    Coord { x: p2.x + nx * r2, y: p2.y + ny * r2 },
                    Coord { x: p2.x - nx * r2, y: p2.y - ny * r2 },
                    Coord { x: p1.x - nx * r1, y: p1.y - ny * r1 },
                    Coord { x: p1.x + nx * r1, y: p1.y + ny * r1 },
                ]), vec![]);
                total_sweep = total_sweep.union(&poly);
            }
            total_sweep = total_sweep.union(&create_circle(p2.x, p2.y, r2));
        }
        total_sweep
    }

    pub fn slice_element(
        element: &VectorElement, eraser_sweep: &MultiPolygon<f32>, canvas_width: f32, canvas_height: f32
    ) -> Vec<VectorElement> {
        match element {
            VectorElement::Contour(contour) => {
                // Execute a single, unified Boolean Subtraction (Martinez-Rueda)
                let clipped_multipoly = contour.shape.difference(eraser_sweep);
                if clipped_multipoly.0.is_empty() { return Vec::new(); }

                let (vertices, indices, aabb) = Extruder::tessellate_multipolygon(
                    &clipped_multipoly, contour.color, canvas_width, canvas_height
                );

                vec![VectorElement::Contour(ContourStroke { shape: clipped_multipoly, color: contour.color, vertices, indices, aabb })]
            },
            VectorElement::Centerline(centerline) => {
                // Densely resample the centerline (1 pt every 2px) to prevent "Large Chunk" erasing
                let mut dense_points = Vec::new();
                if !centerline.points.is_empty() {
                    dense_points.push(centerline.points[0]);
                    for i in 1..centerline.points.len() {
                        let p1 = centerline.points[i-1]; let p2 = centerline.points[i];
                        let dist = ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt();
                        let steps = (dist / 2.0).ceil().max(1.0) as usize; 
                        for s in 1..=steps {
                            let t = s as f32 / steps as f32;
                            dense_points.push(super::Point { x: p1.x + (p2.x - p1.x) * t, y: p1.y + (p2.y - p1.y) * t, pressure: p1.pressure + (p2.pressure - p1.pressure) * t });
                        }
                    }
                }

                let mut fragments = Vec::new();
                let mut current_fragment = Vec::new();

                for pt in &dense_points {
                    // Test if the resampled point falls inside the eraser's sweep shape
                    if eraser_sweep.contains(&geo::Point::new(pt.x, pt.y)) {
                        if current_fragment.len() >= 2 { fragments.push(current_fragment.clone()); }
                        current_fragment.clear();
                    } else { current_fragment.push(*pt); }
                }
                if current_fragment.len() >= 2 { fragments.push(current_fragment); }

                let mut new_elements = Vec::new();
                for frag in fragments {
                    // Simplify the remaining geometry to remove unnecessary dense points and save GPU RAM
                    let mut simplified = vec![frag[0]];
                    for i in 1..frag.len()-1 {
                        let p1 = simplified.last().unwrap(); let p2 = &frag[i];
                        if ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt() > 5.0 { simplified.push(*p2); }
                    }
                    simplified.push(*frag.last().unwrap());

                    let (vertices, indices, aabb) = Extruder::extrude_centerline(&simplified, centerline.thickness, centerline.color, canvas_width, canvas_height);
                    new_elements.push(VectorElement::Centerline(CenterlineStroke {
                        points: simplified, thickness: centerline.thickness, color: centerline.color, vertices, indices, aabb
                    }));
                }
                new_elements
            }
        }
    }
}