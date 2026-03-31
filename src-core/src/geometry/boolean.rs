use super::{Point, VectorElement, CenterlineStroke, ContourStroke, EraserMask};
use crate::geometry::tessellator::Extruder;
use crate::math::AABB;
use geo::{Polygon, MultiPolygon, LineString, Coord, Contains, MapCoords};

pub struct BooleanSlicer;

impl BooleanSlicer {
    fn is_point_in_capsule(test_point: &Point, p1: &Point, p2: &Point, radius: f32) -> bool {
        let dx = p2.x - p1.x; let dy = p2.y - p1.y;
        let length_sq = dx * dx + dy * dy;
        if length_sq < 0.0001 { return (test_point.x - p1.x).powi(2) + (test_point.y - p1.y).powi(2) <= radius * radius; }
        let t = (((test_point.x - p1.x) * dx + (test_point.y - p1.y) * dy) / length_sq).clamp(0.0, 1.0);
        (test_point.x - (p1.x + t * dx)).powi(2) + (test_point.y - (p1.y + t * dy)).powi(2) <= radius * radius
    }

    fn is_point_in_sweep(pt: &Point, sweep_points: &[Point], base_thickness: f32) -> bool {
        for i in 0..sweep_points.len() {
            let r = (base_thickness * sweep_points[i].pressure) / 2.0;
            if (pt.x - sweep_points[i].x).powi(2) + (pt.y - sweep_points[i].y).powi(2) <= r * r { return true; }
            if i > 0 { if Self::is_point_in_capsule(pt, &sweep_points[i-1], &sweep_points[i], r) { return true; } }
        }
        false
    }

    pub fn slice_element(
        element: &VectorElement, raw_sweep: &[Point], base_thickness: f32, canvas_width: f32, canvas_height: f32, smoothing: f32
    ) -> Vec<VectorElement> {
        match element {
            VectorElement::Contour(contour) => {
                let smoothed_eraser = crate::geometry::spline::smooth_spline(raw_sweep, smoothing);
                let (shape, vertices, indices, _) = Extruder::extrude_contour(&smoothed_eraser, base_thickness, [1.0; 4], canvas_width, canvas_height);
                let mut new_masks = contour.eraser_masks.clone();
                new_masks.push(EraserMask { shape: shape.clone(), vertices, indices });

                let mut new_aabb = AABB::empty();
                for poly in contour.shape.iter() {
                    for pt in poly.exterior().0.iter() {
                        if !shape.contains(&geo::Point::new(pt.x, pt.y)) { new_aabb.expand_to_include(pt.x, pt.y, 0.0); }
                    }
                    for int in poly.interiors() {
                        for pt in int.0.iter() {
                            if !shape.contains(&geo::Point::new(pt.x, pt.y)) { new_aabb.expand_to_include(pt.x, pt.y, 0.0); }
                        }
                    }
                }
                let final_aabb = if new_aabb.min_x <= new_aabb.max_x { new_aabb } else { contour.aabb };

                vec![VectorElement::Contour(ContourStroke {
                    shape: contour.shape.clone(), color: contour.color, vertices: contour.vertices.clone(),
                    indices: contour.indices.clone(), aabb: final_aabb, eraser_masks: new_masks, clip_masks: contour.clip_masks.clone()
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

                let mut fragments = Vec::new(); let mut current_fragment = Vec::new();
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
    ) -> (Vec<VectorElement>, Vec<VectorElement>) {
        if lasso_points.len() < 3 { return (Vec::new(), vec![element.clone()]); }

        let mut coords = Vec::new();
        for pt in lasso_points { coords.push(Coord { x: pt.x as f64, y: pt.y as f64 }); }
        coords.push(Coord { x: lasso_points[0].x as f64, y: lasso_points[0].y as f64 });
        
        let lasso_poly = Polygon::new(LineString::new(coords), vec![]);
        let lasso_multi_f32: MultiPolygon<f32> = MultiPolygon::new(vec![lasso_poly.clone()]).map_coords(|c| Coord { x: c.x as f32, y: c.y as f32 });

        match element {
            VectorElement::Contour(contour) => {
                let (lasso_verts, lasso_inds) = Extruder::tessellate_lasso(lasso_points, canvas_width, canvas_height);
                let lasso_mask = EraserMask { shape: lasso_multi_f32.clone(), vertices: lasso_verts, indices: lasso_inds };

                let mut outside_contour = contour.clone();
                outside_contour.eraser_masks.push(lasso_mask.clone());

                let mut outside_aabb = AABB::empty();
                for v in &outside_contour.vertices {
                    let px = (v.position[0] + 1.0) / 2.0 * canvas_width;
                    let py = (1.0 - v.position[1]) / 2.0 * canvas_height;
                    if !lasso_multi_f32.contains(&geo::Point::new(px, py)) {
                        outside_aabb.expand_to_include(px, py, 0.0);
                    }
                }
                outside_contour.aabb = outside_aabb;

                let mut inside_contour = contour.clone();
                inside_contour.clip_masks.push(lasso_mask);

                let mut inside_aabb = AABB::empty();
                for v in &inside_contour.vertices {
                    let px = (v.position[0] + 1.0) / 2.0 * canvas_width;
                    let py = (1.0 - v.position[1]) / 2.0 * canvas_height;
                    if lasso_multi_f32.contains(&geo::Point::new(px, py)) {
                        inside_aabb.expand_to_include(px, py, 0.0);
                    }
                }
                inside_contour.aabb = inside_aabb;

                let mut in_res = Vec::new();
                let mut out_res = Vec::new();

                if inside_contour.aabb.min_x <= inside_contour.aabb.max_x { in_res.push(VectorElement::Contour(inside_contour)); }
                if outside_contour.aabb.min_x <= outside_contour.aabb.max_x { out_res.push(VectorElement::Contour(outside_contour)); }

                (in_res, out_res)
            },
            VectorElement::Centerline(centerline) => {
                let mut inside_fragments = Vec::new();
                let mut outside_fragments = Vec::new();
                let mut current_fragment = Vec::new();
                
                if centerline.points.is_empty() { return (Vec::new(), Vec::new()); }
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
                    } else { current_fragment.push(*pt); }
                }
                if current_fragment.len() >= 2 {
                    if current_state_is_inside { inside_fragments.push(current_fragment); } 
                    else { outside_fragments.push(current_fragment); }
                }

                let build_fragments = |frags: Vec<Vec<Point>>| -> Vec<VectorElement> {
                    let mut res = Vec::new();
                    for frag in frags {
                        let (vertices, indices, aabb) = Extruder::extrude_centerline(&frag, centerline.thickness, centerline.color, canvas_width, canvas_height);
                        res.push(VectorElement::Centerline(CenterlineStroke { points: frag, thickness: centerline.thickness, color: centerline.color, vertices, indices, aabb }));
                    }
                    res
                };

                (build_fragments(inside_fragments), build_fragments(outside_fragments))
            }
        }
    }

    // AAA RESTORED: Appended inside the BooleanSlicer impl block as required by graph.rs
    pub fn is_point_in_polygon(x: f32, y: f32, polygon: &[Point]) -> bool {
        let mut inside = false;
        let mut j = polygon.len() - 1;
        for i in 0..polygon.len() {
            let pi = &polygon[i];
            let pj = &polygon[j];
            if (pi.y > y) != (pj.y > y) && x < (pj.x - pi.x) * (y - pi.y) / (pj.y - pi.y) + pi.x {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
}

// AAA RESTORED: Appended outside the impl block as required by cutter.rs
pub fn create_boolean_mask(lasso_points: &[Point], canvas_width: f32, canvas_height: f32) -> EraserMask {
    let mut coords = Vec::new();
    for pt in lasso_points { coords.push(Coord { x: pt.x as f64, y: pt.y as f64 }); }
    if !coords.is_empty() { coords.push(coords[0]); } 
    
    let shape = MultiPolygon::new(vec![Polygon::new(LineString::new(coords), vec![])])
        .map_coords(|c| Coord { x: c.x as f32, y: c.y as f32 });
        
    let (vertices, indices) = Extruder::tessellate_lasso(lasso_points, canvas_width, canvas_height);
    EraserMask { shape, vertices, indices }
}

pub fn recalculate_aabb(contour: &mut ContourStroke) {
    let mut new_aabb = AABB::empty();
    for poly in &contour.shape {
        for pt in poly.exterior().0.iter() {
            new_aabb.expand_to_include(pt.x, pt.y, 0.0);
        }
    }
    contour.aabb = if new_aabb.min_x <= new_aabb.max_x { new_aabb } else { AABB::empty() };
}