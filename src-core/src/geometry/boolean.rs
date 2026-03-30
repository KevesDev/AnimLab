use super::{Point, VectorElement, CenterlineStroke, ContourStroke, EraserMask};
use crate::geometry::tessellator::Extruder;
use crate::math::AABB;
use geo::{Polygon, LineString, Coord, Contains};

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
                let smoothed_eraser = crate::geometry::spline::smooth_spline(raw_sweep, smoothing);
                let (_, vertices, indices, _) = Extruder::extrude_contour(
                    &smoothed_eraser, base_thickness, [1.0, 1.0, 1.0, 1.0], canvas_width, canvas_height
                );
                let mut new_masks = contour.eraser_masks.clone();
                new_masks.push(EraserMask { vertices, indices });

                vec![VectorElement::Contour(ContourStroke {
                    shape: contour.shape.clone(), color: contour.color, vertices: contour.vertices.clone(),
                    indices: contour.indices.clone(), aabb: contour.aabb, eraser_masks: new_masks, clip_masks: contour.clip_masks.clone()
                })]
            },
            VectorElement::Centerline(centerline) => {
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
                    let (vertices, indices, aabb) = Extruder::extrude_centerline(&frag, centerline.thickness, centerline.color, canvas_width, canvas_height);
                    new_elements.push(VectorElement::Centerline(CenterlineStroke { points: frag, thickness: centerline.thickness, color: centerline.color, vertices, indices, aabb }));
                }
                new_elements
            }
        }
    }

    pub fn lasso_slice(
        element: &VectorElement, lasso_points: &[Point], canvas_width: f32, canvas_height: f32
    ) -> Vec<VectorElement> {
        if lasso_points.len() < 3 { return vec![element.clone()]; }

        let mut coords = Vec::new();
        let mut lasso_aabb = AABB::empty();
        
        for pt in lasso_points { 
            coords.push(Coord { x: pt.x as f64, y: pt.y as f64 }); 
            lasso_aabb.expand_to_include(pt.x, pt.y, 0.0);
        }
        coords.push(Coord { x: lasso_points[0].x as f64, y: lasso_points[0].y as f64 });
        
        let lasso_poly = Polygon::new(LineString::new(coords), vec![]);

        match element {
            VectorElement::Contour(contour) => {
                let (lasso_verts, lasso_inds) = Extruder::tessellate_lasso(lasso_points, canvas_width, canvas_height);
                let lasso_mask = EraserMask { vertices: lasso_verts, indices: lasso_inds };

                let mut outside_contour = contour.clone();
                outside_contour.eraser_masks.push(lasso_mask.clone());

                let mut inside_contour = contour.clone();
                inside_contour.clip_masks.push(lasso_mask);

                // AAA FIX: Clamp the Bounding Box of the severed fragment to exactly match the Lasso bounds.
                // This permanently stops the spatial grid from infinitely catching and duplicating this fragment.
                inside_contour.aabb.min_x = contour.aabb.min_x.max(lasso_aabb.min_x);
                inside_contour.aabb.min_y = contour.aabb.min_y.max(lasso_aabb.min_y);
                inside_contour.aabb.max_x = contour.aabb.max_x.min(lasso_aabb.max_x);
                inside_contour.aabb.max_y = contour.aabb.max_y.min(lasso_aabb.max_y);

                if inside_contour.aabb.min_x > inside_contour.aabb.max_x || inside_contour.aabb.min_y > inside_contour.aabb.max_y {
                    return vec![VectorElement::Contour(outside_contour)];
                }

                vec![ VectorElement::Contour(inside_contour), VectorElement::Contour(outside_contour) ]
            },
            VectorElement::Centerline(centerline) => {
                let mut inside_fragments = Vec::new();
                let mut outside_fragments = Vec::new();
                let mut current_fragment = Vec::new();
                
                if centerline.points.is_empty() { return Vec::new(); }
                let mut current_state_is_inside = lasso_poly.contains(&geo::Point::new(centerline.points[0].x as f64, centerline.points[0].y as f64));

                for pt in &centerline.points {
                    let is_inside = lasso_poly.contains(&geo::Point::new(pt.x as f64, pt.y as f64));
                    
                    if is_inside != current_state_is_inside {
                        current_fragment.push(*pt); 
                        
                        if current_fragment.len() >= 2 {
                            if current_state_is_inside { inside_fragments.push(current_fragment.clone()); } 
                            else { outside_fragments.push(current_fragment.clone()); }
                        }
                        current_fragment.clear();
                        current_fragment.push(*pt); 
                        current_state_is_inside = is_inside;
                    } else {
                        current_fragment.push(*pt);
                    }
                }
                
                if current_fragment.len() >= 2 {
                    if current_state_is_inside { inside_fragments.push(current_fragment); } 
                    else { outside_fragments.push(current_fragment); }
                }

                let mut results = Vec::new();
                let mut build_fragments = |frags: Vec<Vec<Point>>| {
                    for frag in frags {
                        let mut simplified = vec![frag[0]];
                        for i in 1..frag.len()-1 {
                            let p1 = simplified.last().unwrap(); let p2 = &frag[i];
                            if ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt() > 5.0 { simplified.push(*p2); }
                        }
                        simplified.push(*frag.last().unwrap());

                        let (vertices, indices, aabb) = Extruder::extrude_centerline(&simplified, centerline.thickness, centerline.color, canvas_width, canvas_height);
                        results.push(VectorElement::Centerline(CenterlineStroke { points: simplified, thickness: centerline.thickness, color: centerline.color, vertices, indices, aabb }));
                    }
                };

                build_fragments(inside_fragments);
                build_fragments(outside_fragments);
                results
            }
        }
    }
}