use std::collections::{HashMap, HashSet};
use log::{info, warn};
use crate::stroke::Stroke;
use crate::math::AABB;

pub type NodeId = u64;
pub type StrokeId = u64;

const CHUNK_SIZE: f32 = 128.0;

#[derive(Debug)]
pub enum NodeType {
    VectorLayer { 
        strokes: HashMap<StrokeId, Stroke>,
        spatial_grid: HashMap<(i32, i32), HashSet<StrokeId>>,
    },
    
    // AAA BIFURCATION: The Hardware-Agnostic Pixel Buffer
    RasterLayer {
        width: u32,
        height: u32,
        pixels: Vec<u8>, // RGBA 8-bit array
        is_dirty: bool,  // The Resource Manager Sync Flag
    },
    
    Composite,
    Output,
}

#[derive(Debug)]
pub struct AnimNode {
    pub id: NodeId,
    pub name: String,
    pub payload: NodeType,
}

#[derive(Debug)]
pub struct AnimGraph {
    pub nodes: HashMap<NodeId, AnimNode>,
    pub edges: Vec<(NodeId, NodeId)>,
    next_id: NodeId,
    pub active_layer_node: Option<NodeId>,
}

impl AnimGraph {
    pub fn new() -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            next_id: 1, 
            active_layer_node: None,
        };

        let output_id = graph.add_node("Master Output".to_string(), NodeType::Output);
        
        let initial_vector_id = graph.add_node("Vector Layer 1".to_string(), NodeType::VectorLayer { 
            strokes: HashMap::new(),
            spatial_grid: HashMap::new(),
        });
        
        graph.active_layer_node = Some(initial_vector_id);
        graph.connect_nodes(initial_vector_id, output_id);

        info!("AnimGraph Initialized: DAG Bifurcation Online.");
        graph
    }

    pub fn add_node(&mut self, name: String, payload: NodeType) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        let node = AnimNode { id, name, payload };
        self.nodes.insert(id, node);
        id
    }

    pub fn connect_nodes(&mut self, source_id: NodeId, target_id: NodeId) {
        if self.nodes.contains_key(&source_id) && self.nodes.contains_key(&target_id) {
            self.edges.push((source_id, target_id));
        }
    }

    fn get_chunks_for_aabb(aabb: &AABB) -> Vec<(i32, i32)> {
        let start_x = (aabb.min_x / CHUNK_SIZE).floor() as i32;
        let end_x = (aabb.max_x / CHUNK_SIZE).floor() as i32;
        let start_y = (aabb.min_y / CHUNK_SIZE).floor() as i32;
        let end_y = (aabb.max_y / CHUNK_SIZE).floor() as i32;

        let mut chunks = Vec::new();
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                chunks.push((x, y));
            }
        }
        chunks
    }

    pub fn insert_stroke_by_id(&mut self, node_id: NodeId, stroke_id: StrokeId, stroke: Stroke) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let NodeType::VectorLayer { strokes, spatial_grid } = &mut node.payload {
                let chunks = Self::get_chunks_for_aabb(&stroke.aabb);
                for chunk in chunks {
                    spatial_grid.entry(chunk).or_insert_with(HashSet::new).insert(stroke_id);
                }
                strokes.insert(stroke_id, stroke);
                return;
            } else {
                warn!("Engine Collision: Attempted to inject Vector Math into a Raster/Composite Node [{}].", node_id);
                return;
            }
        }
    }

    pub fn remove_stroke_by_id(&mut self, node_id: NodeId, stroke_id: StrokeId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let NodeType::VectorLayer { strokes, spatial_grid } = &mut node.payload {
                if let Some(stroke) = strokes.get(&stroke_id) {
                    let chunks = Self::get_chunks_for_aabb(&stroke.aabb);
                    for chunk in chunks {
                        if let Some(cell) = spatial_grid.get_mut(&chunk) {
                            cell.remove(&stroke_id);
                            if cell.is_empty() { spatial_grid.remove(&chunk); } 
                        }
                    }
                }
                strokes.remove(&stroke_id);
                return;
            }
        }
    }

    pub fn query_spatial_grid(&self, node_id: NodeId, target_aabb: &AABB) -> Vec<&Stroke> {
        let mut found_strokes = Vec::new();
        let mut checked_ids = HashSet::new(); 

        if let Some(node) = self.nodes.get(&node_id) {
            if let NodeType::VectorLayer { strokes, spatial_grid } = &node.payload {
                let chunks_to_check = Self::get_chunks_for_aabb(target_aabb);
                
                for chunk in chunks_to_check {
                    if let Some(stroke_ids_in_chunk) = spatial_grid.get(&chunk) {
                        for id in stroke_ids_in_chunk {
                            if checked_ids.insert(*id) { 
                                if let Some(stroke) = strokes.get(id) {
                                    if stroke.aabb.intersects(target_aabb) {
                                        found_strokes.push(stroke);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        found_strokes
    }

    pub fn collect_renderable_strokes(&self) -> Vec<&Stroke> {
        let mut render_list = Vec::new();
        for node in self.nodes.values() {
            if let NodeType::VectorLayer { strokes, .. } = &node.payload {
                for stroke in strokes.values() {
                    render_list.push(stroke);
                }
            }
        }
        render_list
    }
}