use super::{Point, VectorElement, CenterlineStroke, ContourStroke, EraserMask};
use crate::geometry::tessellator::Extruder;
use crate::geometry::spline::smooth_spline;

pub struct BooleanSlicer;

impl BooleanSlicer {
    fn is_point_in_capsule(test_point: &Point, p1: &Point, p2: &Point, radius: f32) -> bool {
        let dx = p2.x - p1.x; let dy = p2.y - p1.y;
        let length_sq = dx * dx + dy * dy;
        if length_sq < 0.0001 {
            let dist_sq = (test_point.x - p1.x).powi(2) + (test_point.y - p1.y).powi(2);
            return dist_sq <= radius * radius;
        }
        let t = ((test_point.x - p1.x) * dx + (test_point.y - p1.y) * dy) / length_sq;
        let t = t.clamp(0.0, 1.0);
        let closest_x = p1.x + t * dx; let closest_y = p1.y + t * dy;
        let dist_sq = (test_point.x - closest_x).powi(2) + (test_point.y - closest_y).powi(2);
        dist_sq <= radius * radius
    }

    fn is_point_in_sweep(pt: &Point, sweep_points: &[Point], base_thickness: f32) -> bool {
        for i in 0..sweep_points.len() {
            let r = (base_thickness * sweep_points[i].pressure) / 2.0;
            if (pt.x - sweep_points[i].x).powi(2) + (pt.y - sweep_points[i].y).powi(2) <= r * r { return true; }
            if i > 0 {
                let p1 = &sweep_points[i-1];
                if Self::is_point_in_capsule(pt, p1, &sweep_points[i], r) { return true; }
            }
        }
        false
    }

    pub fn slice_element(
        element: &VectorElement, raw_sweep: &[Point], base_thickness: f32, canvas_width: f32, canvas_height: f32, smoothing: f32
    ) -> Vec<VectorElement> {
        match element {
            VectorElement::Contour(contour) => {
                // AAA ARCHITECTURE: Stencil Masking
                // We do NOT perform heavy CPU math. We generate a fast triangle mesh of the eraser stroke 
                // and attach it to the contour. The WebGPU pipeline handles the visual subtraction.
                let smoothed_eraser = smooth_spline(raw_sweep, smoothing);
                let (_, vertices, indices, _) = Extruder::extrude_contour(
                    &smoothed_eraser, base_thickness, [1.0, 1.0, 1.0, 1.0], canvas_width, canvas_height
                );
                
                let mut new_masks = contour.eraser_masks.clone();
                new_masks.push(EraserMask { vertices, indices });

                vec![VectorElement::Contour(ContourStroke {
                    shape: contour.shape.clone(),
                    color: contour.color,
                    vertices: contour.vertices.clone(),
                    indices: contour.indices.clone(),
                    aabb: contour.aabb,
                    eraser_masks: new_masks,
                })]
            },
            VectorElement::Centerline(centerline) => {
                // Centerlines are 1D arrays, so O(N) distance checks are safe and blistering fast on the CPU.
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
                    if Self::is_point_in_sweep(pt, raw_sweep, base_thickness) {
                        if current_fragment.len() >= 2 { fragments.push(current_fragment.clone()); }
                        current_fragment.clear();
                    } else { current_fragment.push(*pt); }
                }
                if current_fragment.len() >= 2 { fragments.push(current_fragment); }

                let mut new_elements = Vec::new();
                for frag in fragments {
                    let mut simplified = vec![frag[0]];
                    for i in 1..frag.len()-1 {
                        let p1 = simplified.last().unwrap(); let p2 = &frag[i];
                        if ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt() > 5.0 { simplified.push(*p2); }
                    }
                    simplified.push(*frag.last().unwrap());

                    let (vertices, indices, aabb) = Extruder::extrude_centerline(&simplified, centerline.thickness, centerline.color, canvas_width, canvas_height);
                    new_elements.push(VectorElement::Centerline(CenterlineStroke { points: simplified, thickness: centerline.thickness, color: centerline.color, vertices, indices, aabb }));
                }
                new_elements
            }
        }
    }
}