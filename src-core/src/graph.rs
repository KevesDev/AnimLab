use std::collections::{HashMap, HashSet};
use crate::geometry::{VectorElement, Point};
use crate::math::AABB;
use geo::Contains;

pub type ElementId = u64;
pub type DrawingId = u64;
pub type StrokeId = u64;
pub type FrameNumber = u32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode { Normal, Multiply, Screen, Add }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtLayerType { Overlay, LineArt, ColorArt, Underlay }

#[derive(Debug, Clone)]
pub struct ArtLayer { pub vector_elements: HashMap<StrokeId, VectorElement> }
impl ArtLayer { pub fn new() -> Self { Self { vector_elements: HashMap::new() } } }

#[derive(Debug, Clone)]
pub struct DrawingData {
    pub overlay: ArtLayer, pub line_art: ArtLayer, pub color_art: ArtLayer, pub underlay: ArtLayer,
}
impl DrawingData {
    pub fn new() -> Self { Self { overlay: ArtLayer::new(), line_art: ArtLayer::new(), color_art: ArtLayer::new(), underlay: ArtLayer::new() } }
    pub fn get_art_layer_mut(&mut self, layer_type: ArtLayerType) -> &mut ArtLayer {
        match layer_type { ArtLayerType::Overlay => &mut self.overlay, ArtLayerType::LineArt => &mut self.line_art, ArtLayerType::ColorArt => &mut self.color_art, ArtLayerType::Underlay => &mut self.underlay }
    }
    pub fn get_art_layer(&self, layer_type: ArtLayerType) -> &ArtLayer {
        match layer_type { ArtLayerType::Overlay => &self.overlay, ArtLayerType::LineArt => &self.line_art, ArtLayerType::ColorArt => &self.color_art, ArtLayerType::Underlay => &self.underlay }
    }
}

#[derive(Debug, Clone)]
pub struct DrawingElement {
    pub id: ElementId, pub name: String, pub is_visible: bool, pub is_locked: bool, pub opacity: f32, pub blend_mode: BlendMode, pub z_nudge: f32,
    pub library: HashMap<DrawingId, DrawingData>, pub exposures: HashMap<FrameNumber, DrawingId>,
}
impl DrawingElement {
    pub fn new(id: ElementId, name: String) -> Self { Self { id, name, is_visible: true, is_locked: false, opacity: 1.0, blend_mode: BlendMode::Normal, z_nudge: 0.0, library: HashMap::new(), exposures: HashMap::new() } }
    pub fn get_drawing_mut(&mut self, frame: FrameNumber) -> Option<&mut DrawingData> {
        if let Some(&drawing_id) = self.exposures.get(&frame) { self.library.get_mut(&drawing_id) } else { None }
    }
    pub fn get_drawing(&self, frame: FrameNumber) -> Option<&DrawingData> {
        if let Some(&drawing_id) = self.exposures.get(&frame) { self.library.get(&drawing_id) } else { None }
    }
}

pub struct IdAllocator { next_id: u64 }
impl IdAllocator {
    pub fn new() -> Self { Self { next_id: 1000 } }
    pub fn generate(&mut self) -> u64 { let id = self.next_id; self.next_id += 1; id }
}

pub struct SceneManager {
    pub elements: HashMap<ElementId, DrawingElement>, pub z_stack: Vec<ElementId>, pub current_frame: FrameNumber,
    pub active_element_id: Option<ElementId>, pub active_art_layer: ArtLayerType, pub selected_strokes: HashSet<StrokeId>,
}

impl SceneManager {
    pub fn new() -> Self { Self { elements: HashMap::new(), z_stack: Vec::new(), current_frame: 1, active_element_id: None, active_art_layer: ArtLayerType::LineArt, selected_strokes: HashSet::new() } }

    pub fn ensure_drawing_exists(&mut self, allocator: &mut IdAllocator) {
        if let Some(el_id) = self.active_element_id {
            if let Some(el) = self.elements.get_mut(&el_id) {
                if !el.exposures.contains_key(&self.current_frame) {
                    let new_draw_id = allocator.generate();
                    el.library.insert(new_draw_id, DrawingData::new()); el.exposures.insert(self.current_frame, new_draw_id);
                }
            }
        }
    }

    pub fn get_active_art_layer_mut(&mut self) -> Option<(DrawingId, &mut ArtLayer)> {
        let el_id = self.active_element_id?; let frame = self.current_frame; let art_type = self.active_art_layer;
        let el = self.elements.get_mut(&el_id)?; let draw_id = *el.exposures.get(&frame)?; let drawing = el.library.get_mut(&draw_id)?;
        Some((draw_id, drawing.get_art_layer_mut(art_type)))
    }

    pub fn get_active_art_layer(&self) -> Option<(DrawingId, &ArtLayer)> {
        let el_id = self.active_element_id?; let frame = self.current_frame; let art_type = self.active_art_layer;
        let el = self.elements.get(&el_id)?; let draw_id = *el.exposures.get(&frame)?; let drawing = el.library.get(&draw_id)?;
        Some((draw_id, drawing.get_art_layer(art_type)))
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<StrokeId> {
        let mut best_hit: Option<StrokeId> = None;
        let mut highest_id = 0;

        if let Some((_, layer)) = self.get_active_art_layer() {
            for (id, element) in &layer.vector_elements {
                if *id > highest_id && x >= element.aabb().min_x && x <= element.aabb().max_x && y >= element.aabb().min_y && y <= element.aabb().max_y {
                    let mut is_hit = false;
                    match element {
                        VectorElement::Centerline(c) => {
                            let r = (c.thickness / 2.0).max(5.0); 
                            if c.points.len() == 1 {
                                if (c.points[0].x - x).powi(2) + (c.points[0].y - y).powi(2) <= r * r { is_hit = true; }
                            } else {
                                for i in 0..c.points.len().saturating_sub(1) {
                                    let p1 = &c.points[i]; let p2 = &c.points[i+1];
                                    let dx = p2.x - p1.x; let dy = p2.y - p1.y;
                                    let length_sq = dx * dx + dy * dy;
                                    let dist_sq = if length_sq < 0.0001 { (x - p1.x).powi(2) + (y - p1.y).powi(2) } 
                                    else { let t = (((x - p1.x) * dx + (y - p1.y) * dy) / length_sq).clamp(0.0, 1.0); (x - (p1.x + t * dx)).powi(2) + (y - (p1.y + t * dy)).powi(2) };
                                    if dist_sq <= r * r { is_hit = true; break; }
                                }
                            }
                        },
                        VectorElement::Contour(c) => {
                            let pt = geo::Point::new(x, y);
                            is_hit = c.shape.contains(&pt);
                            if is_hit {
                                if !c.clip_masks.is_empty() {
                                    let mut inside_clip = false;
                                    for mask in &c.clip_masks { if mask.shape.contains(&pt) { inside_clip = true; break; } }
                                    if !inside_clip { is_hit = false; }
                                }
                                if is_hit {
                                    for mask in &c.eraser_masks { if mask.shape.contains(&pt) { is_hit = false; break; } }
                                }
                            }
                        }
                    }
                    if is_hit { highest_id = *id; best_hit = Some(*id); }
                }
            }
        }
        best_hit
    }

    pub fn hit_test_lasso(&self, lasso_points: &[Point]) -> Vec<StrokeId> {
        let mut hits = Vec::new();
        if let Some((_, layer)) = self.get_active_art_layer() {
            for (id, element) in &layer.vector_elements {
                let aabb = element.aabb();
                let cx = aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0;
                let cy = aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0;
                if crate::geometry::boolean::BooleanSlicer::is_point_in_polygon(cx, cy, lasso_points) { hits.push(*id); }
            }
        }
        hits
    }

    pub fn get_selection_aabb(&self) -> Option<AABB> {
        if self.selected_strokes.is_empty() { return None; }
        let mut combined = AABB::empty();
        if let Some((_, layer)) = self.get_active_art_layer() {
            for id in &self.selected_strokes {
                if let Some(element) = layer.vector_elements.get(id) {
                    let aabb = element.aabb();
                    combined.expand_to_include(aabb.min_x, aabb.min_y, 0.0);
                    combined.expand_to_include(aabb.max_x, aabb.max_y, 0.0);
                }
            }
        }
        if combined.min_x > combined.max_x { None } else { Some(combined) }
    }

    pub fn collect_renderable_elements(&self) -> Vec<&VectorElement> {
        let mut elements = Vec::new();
        for el_id in &self.z_stack {
            if let Some(el) = self.elements.get(el_id) {
                if !el.is_visible { continue; }
                if let Some(drawing) = el.get_drawing(self.current_frame) {
                    let types = [ArtLayerType::Underlay, ArtLayerType::ColorArt, ArtLayerType::LineArt, ArtLayerType::Overlay];
                    for art_type in types {
                        let layer = drawing.get_art_layer(art_type);
                        for vector_el in layer.vector_elements.values() { elements.push(vector_el); }
                    }
                }
            }
        }
        elements
    }
}