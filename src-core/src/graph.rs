use std::collections::{HashMap, HashSet};
use log::{info, warn, error};
use crate::math::AABB;
use crate::geometry::VectorElement;

pub type NodeId = u64;
pub type StrokeId = u64;

const CHUNK_SIZE: f32 = 128.0;

// AAA ARCHITECTURE: Decoupled ID Allocator
// Safely passed to Tools during 'on_pointer_up' to allow dynamic fragment generation
pub struct IdAllocator {
    next_id: u64,
}
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
}

impl AnimGraph {
    pub fn new() -> Self {
        let mut graph = Self { nodes: HashMap::new(), edges: Vec::new(), next_id: 1, active_layer_node: None };
        let output_id = graph.add_node("Master Output".to_string(), NodeType::Output);
        let initial_vector_id = graph.add_node("Vector Layer 1".to_string(), NodeType::VectorLayer { elements: HashMap::new(), spatial_grid: HashMap::new() });
        graph.active_layer_node = Some(initial_vector_id);
        graph.connect_nodes(initial_vector_id, output_id);
        info!("AnimGraph Initialized: DAG utilizing global geometry pipeline.");
        graph
    }

    pub fn add_node(&mut self, name: String, payload: NodeType) -> NodeId {
        let id = self.next_id; self.next_id += 1;
        let node = AnimNode { id, name, payload };
        self.nodes.insert(id, node);
        id
    }

    pub fn connect_nodes(&mut self, source_id: NodeId, target_id: NodeId) {
        if self.nodes.contains_key(&source_id) && self.nodes.contains_key(&target_id) { self.edges.push((source_id, target_id)); } 
        else { error!("DAG Integrity Error: Attempted to connect missing nodes {} -> {}", source_id, target_id); }
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
            } else { warn!("Engine Routing Error: Tool targeted a Raster Node [{}]. Discarding geometry.", node_id); }
        } else { error!("Engine Routing Error: Node [{}] does not exist.", node_id); }
    }

    pub fn remove_stroke_by_id(&mut self, node_id: NodeId, stroke_id: StrokeId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let NodeType::VectorLayer { elements, spatial_grid } = &mut node.payload {
                if let Some(element) = elements.get(&stroke_id) {
                    let chunks = Self::get_chunks_for_aabb(element.aabb());
                    for chunk in chunks {
                        if let Some(cell) = spatial_grid.get_mut(&chunk) {
                            cell.remove(&stroke_id);
                            if cell.is_empty() { spatial_grid.remove(&chunk); } 
                        }
                    }
                }
                elements.remove(&stroke_id);
            }
        }
    }

    pub fn collect_renderable_elements(&self) -> Vec<&VectorElement> {
        let mut render_list = Vec::new();
        for node in self.nodes.values() {
            if let NodeType::VectorLayer { elements, .. } = &node.payload {
                for element in elements.values() { render_list.push(element); }
            }
        }
        render_list
    }

    // AAA UPGRADE: Fast-path querying for the Boolean Slicer
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
}