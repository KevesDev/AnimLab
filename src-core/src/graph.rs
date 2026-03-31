use std::collections::{HashMap, HashSet};
use log::info;
use crate::math::AABB;
use crate::geometry::VectorElement;
use geo::Contains;

pub type NodeId = u64;
pub type StrokeId = u64;
const CHUNK_SIZE: f32 = 128.0;

pub struct IdAllocator { next_id: u64 }
impl IdAllocator {
    pub fn new() -> Self { Self { next_id: 1 } }
    pub fn generate(&mut self) -> u64 { let id = self.next_id; self.next_id += 1; id }
}

#[derive(Debug)]
pub enum NodeType {
    VectorLayer { elements: HashMap<StrokeId, VectorElement>, spatial_grid: HashMap<(i32, i32), HashSet<StrokeId>> },
    RasterLayer { width: u32, height: u32, pixels: Vec<u8>, is_dirty: bool },
    Composite, Output,
}

#[derive(Debug)]
pub struct AnimNode { pub id: NodeId, pub name: String, pub payload: NodeType }

#[derive(Debug)]
pub struct AnimGraph {
    pub nodes: HashMap<NodeId, AnimNode>,
    pub edges: Vec<(NodeId, NodeId)>,
    next_id: NodeId,
    pub active_layer_node: Option<NodeId>,
    pub selected_strokes: HashSet<StrokeId>, 
}

impl AnimGraph {
    pub fn new() -> Self {
        let mut graph = Self { nodes: HashMap::new(), edges: Vec::new(), next_id: 1, active_layer_node: None, selected_strokes: HashSet::new() };
        let output_id = graph.add_node("Master Output".to_string(), NodeType::Output);
        let initial_vector_id = graph.add_node("Vector Layer 1".to_string(), NodeType::VectorLayer { elements: HashMap::new(), spatial_grid: HashMap::new() });
        graph.active_layer_node = Some(initial_vector_id);
        graph.connect_nodes(initial_vector_id, output_id);
        info!("AnimGraph Initialized.");
        graph
    }

    pub fn add_node(&mut self, name: String, payload: NodeType) -> NodeId {
        let id = self.next_id; self.next_id += 1;
        self.nodes.insert(id, AnimNode { id, name, payload });
        id
    }

    pub fn connect_nodes(&mut self, source_id: NodeId, target_id: NodeId) {
        if self.nodes.contains_key(&source_id) && self.nodes.contains_key(&target_id) { self.edges.push((source_id, target_id)); } 
    }

    fn get_chunks_for_aabb(aabb: &AABB) -> Vec<(i32, i32)> {
        let start_x = (aabb.min_x / CHUNK_SIZE).floor() as i32; let end_x = (aabb.max_x / CHUNK_SIZE).floor() as i32;
        let start_y = (aabb.min_y / CHUNK_SIZE).floor() as i32; let end_y = (aabb.max_y / CHUNK_SIZE).floor() as i32;
        let mut chunks = Vec::new();
        for x in start_x..=end_x { for y in start_y..=end_y { chunks.push((x, y)); } }
        chunks
    }

    pub fn insert_stroke_by_id(&mut self, node_id: NodeId, stroke_id: StrokeId, element: VectorElement) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let NodeType::VectorLayer { elements, spatial_grid } = &mut node.payload {
                let chunks = Self::get_chunks_for_aabb(element.aabb());
                for chunk in chunks { spatial_grid.entry(chunk).or_insert_with(HashSet::new).insert(stroke_id); }
                elements.insert(stroke_id, element);
            }
        }
    }

    pub fn remove_stroke_by_id(&mut self, node_id: NodeId, stroke_id: StrokeId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let NodeType::VectorLayer { elements, spatial_grid } = &mut node.payload {
                if let Some(element) = elements.get(&stroke_id) {
                    let chunks = Self::get_chunks_for_aabb(element.aabb());
                    for chunk in chunks {
                        if let Some(cell) = spatial_grid.get_mut(&chunk) { cell.remove(&stroke_id); if cell.is_empty() { spatial_grid.remove(&chunk); } }
                    }
                }
                elements.remove(&stroke_id);
            }
        }
    }

    pub fn collect_renderable_elements(&self) -> Vec<&VectorElement> {
        let mut render_list = Vec::new();
        for node in self.nodes.values() {
            if let NodeType::VectorLayer { elements, .. } = &node.payload { for element in elements.values() { render_list.push(element); } }
        }
        render_list
    }

    pub fn query_spatial_grid_ids(&self, node_id: NodeId, target_aabb: &AABB) -> Vec<StrokeId> {
        let mut found_ids = HashSet::new();
        if let Some(node) = self.nodes.get(&node_id) {
            if let NodeType::VectorLayer { elements, spatial_grid } = &node.payload {
                let chunks = Self::get_chunks_for_aabb(target_aabb);
                for chunk in chunks {
                    if let Some(ids) = spatial_grid.get(&chunk) {
                        for id in ids {
                            if let Some(element) = elements.get(id) {
                                if element.aabb().intersects(target_aabb) { found_ids.insert(*id); }
                            }
                        }
                    }
                }
            }
        }
        found_ids.into_iter().collect()
    }

    pub fn hit_test(&self, node_id: NodeId, x: f32, y: f32) -> Option<StrokeId> {
        let mut best_hit: Option<StrokeId> = None;
        let mut highest_id = 0;

        if let Some(node) = self.nodes.get(&node_id) {
            if let NodeType::VectorLayer { elements, .. } = &node.payload {
                for (id, element) in elements {
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

                                // AAA FIX: Mask-Aware Hit-Testing
                                if is_hit {
                                    // 1. If it's a child piece (inside a clip mask), you MUST have clicked inside the mask
                                    if !c.clip_masks.is_empty() {
                                        let mut inside_clip = false;
                                        for mask in &c.clip_masks { if mask.shape.contains(&pt) { inside_clip = true; break; } }
                                        if !inside_clip { is_hit = false; }
                                    }
                                    // 2. If it's a parent piece with a hole (eraser mask), you CANNOT click the hole
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
        }
        best_hit
    }

    pub fn hit_test_lasso(&self, node_id: NodeId, lasso_points: &[crate::geometry::Point]) -> Vec<StrokeId> {
        let mut selected = Vec::new();
        if lasso_points.len() < 3 { return selected; }

        let mut lasso_aabb = AABB::empty();
        for pt in lasso_points { lasso_aabb.expand_to_include(pt.x, pt.y, 0.0); }

        if let Some(node) = self.nodes.get(&node_id) {
            if let NodeType::VectorLayer { elements, .. } = &node.payload {
                for (id, element) in elements {
                    // Because we fixed the AABB shrink-wrapping, this simple intersection check is incredibly precise
                    if element.aabb().intersects(&lasso_aabb) { selected.push(*id); }
                }
            }
        }
        selected
    }

    pub fn get_selection_aabb(&self, node_id: NodeId) -> Option<AABB> {
        let mut bounds = AABB::empty();
        let mut has_bounds = false;
        if let Some(node) = self.nodes.get(&node_id) {
            if let NodeType::VectorLayer { elements, .. } = &node.payload {
                for id in &self.selected_strokes {
                    if let Some(element) = elements.get(id) {
                        let e_aabb = element.aabb();
                        bounds.expand_to_include(e_aabb.min_x, e_aabb.min_y, 0.0);
                        bounds.expand_to_include(e_aabb.max_x, e_aabb.max_y, 0.0);
                        has_bounds = true;
                    }
                }
            }
        }
        if has_bounds { Some(bounds) } else { None }
    }
}