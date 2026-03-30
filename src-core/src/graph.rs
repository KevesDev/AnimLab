use std::collections::HashMap;
use log::{info, warn};
use crate::stroke::Stroke;

/// Unique identifier for every node in the graph.
pub type NodeId = u64;

/// Defines the specific behavior and memory payload of a Node.
#[derive(Debug)]
pub enum NodeType {
    /// Holds raw mathematical vector artwork. Equivalent to a "Layer".
    Drawing { strokes: Vec<Stroke> },
    
    /// Mathematically combines multiple input nodes into a single image.
    Composite,
    
    /// The final destination of the render pipeline. What the user actually sees.
    Output,
}

/// A singular block of the Directed Acyclic Graph.
#[derive(Debug)]
pub struct AnimNode {
    pub id: NodeId,
    pub name: String,
    pub payload: NodeType,
}

/// The master Directed Acyclic Graph (DAG) that runs the entire animation engine.
#[derive(Debug)]
pub struct AnimGraph {
    pub nodes: HashMap<NodeId, AnimNode>,
    /// Defines the cables connecting nodes. Format: (Output_Pin_Of_Node_A, Input_Pin_Of_Node_B)
    pub edges: Vec<(NodeId, NodeId)>,
    next_id: NodeId,
    
    /// Tracks which node the user is currently drawing on.
    pub active_drawing_node: Option<NodeId>,
}

impl AnimGraph {
    pub fn new() -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            next_id: 1, // Start IDs at 1, reserving 0 as a potential null-pointer
            active_drawing_node: None,
        };

        // --- AAA ENGINE INITIALIZATION ---
        // A node-based engine must always have a Master Output, and at least 
        // one Drawing Node connected to it so the user can immediately start working.
        
        let output_id = graph.add_node("Master Output".to_string(), NodeType::Output);
        
        let initial_drawing_id = graph.add_node("Drawing 1".to_string(), NodeType::Drawing { strokes: Vec::new() });
        graph.active_drawing_node = Some(initial_drawing_id);

        // Connect the Drawing Node to the Master Output
        graph.connect_nodes(initial_drawing_id, output_id);

        info!("AnimGraph Initialized: Master Output [{}] connected to [{}]", output_id, initial_drawing_id);

        graph
    }

    /// Safely generates a new node and registers it in the memory map.
    pub fn add_node(&mut self, name: String, payload: NodeType) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;

        let node = AnimNode { id, name, payload };
        self.nodes.insert(id, node);
        id
    }

    /// Creates a directional data flow from one node to another.
    pub fn connect_nodes(&mut self, source_id: NodeId, target_id: NodeId) {
        if self.nodes.contains_key(&source_id) && self.nodes.contains_key(&target_id) {
            self.edges.push((source_id, target_id));
        } else {
            warn!("Engine attempted to connect invalid node IDs: {} -> {}", source_id, target_id);
        }
    }

    /// Injects a completed brush stroke into the currently active drawing node.
    pub fn inject_stroke(&mut self, stroke: Stroke) {
        if let Some(active_id) = self.active_drawing_node {
            if let Some(node) = self.nodes.get_mut(&active_id) {
                if let NodeType::Drawing { ref mut strokes } = node.payload {
                    strokes.push(stroke);
                    return;
                }
            }
        }
        warn!("Engine Input Dropped: No active drawing node found in the graph.");
    }

    /// Extracts all strokes from the graph for the render loop.
    /// In the future, this will traverse the graph hierarchically.
    pub fn collect_renderable_strokes(&self) -> Vec<&Stroke> {
        let mut render_list = Vec::new();
        
        for node in self.nodes.values() {
            if let NodeType::Drawing { strokes } = &node.payload {
                for stroke in strokes {
                    render_list.push(stroke);
                }
            }
        }
        
        render_list
    }
}