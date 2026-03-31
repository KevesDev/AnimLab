use wasm_bindgen::prelude::*;
use log::{info, warn, error, Level};
use web_sys::HtmlCanvasElement;
use std::panic;

pub mod math;
pub mod geometry; 
pub mod settings; 
pub mod graph; 
pub mod command; 
pub mod tools;
pub mod renderer; // AAA ARCHITECTURE: The new modular graphics pipeline

use graph::{AnimGraph, IdAllocator};
use command::{CommandHistory, CutCommand, BatchCommand, AddStrokeCommand, AffineCommand};
use tools::{CanvasTool, brush::BrushTool, pencil::PencilTool, eraser::EraserTool, cutter::CutterTool, select::SelectTool};
use geometry::VectorElement;
use renderer::WebGpuRenderer;

#[derive(Debug)]
pub enum EngineError { LoggerInitFailed(String) }

impl From<EngineError> for JsValue {
    fn from(err: EngineError) -> JsValue {
        match err { EngineError::LoggerInitFailed(msg) => JsValue::from_str(&format!("AnimLab Fatal: {}", msg)), }
    }
}

pub struct CursorManager { current: &'static str }
impl CursorManager {
    pub fn new() -> Self { Self { current: "default" } }
    pub fn apply(&mut self, cursor: &'static str) {
        if self.current != cursor {
            self.current = cursor;
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(body) = doc.body() { let _ = body.style().set_property("cursor", cursor); }
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn init_core() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).map_err(|e| EngineError::LoggerInitFailed(e.to_string()))?;
    info!("AnimLab Core System: WASM Binary loaded.");
    Ok(())
}

#[wasm_bindgen]
pub struct AnimLabEngine {
    is_ready: bool,
    canvas_width: f32,
    canvas_height: f32,
    
    #[wasm_bindgen(skip)] pub renderer: Option<WebGpuRenderer>,
    #[wasm_bindgen(skip)] pub active_tool: Box<dyn CanvasTool>,
    #[wasm_bindgen(skip)] pub graph: AnimGraph,
    #[wasm_bindgen(skip)] pub history: CommandHistory,
    #[wasm_bindgen(skip)] pub id_allocator: IdAllocator,
    #[wasm_bindgen(skip)] pub cursor_manager: CursorManager,
    
    // AAA ARCHITECTURE: Lightning Fast Internal Memory Clipboard
    #[wasm_bindgen(skip)] pub clipboard: Vec<VectorElement>, 
}

#[wasm_bindgen]
impl AnimLabEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<AnimLabEngine, JsValue> {
        Ok(AnimLabEngine {
            is_ready: true, canvas_width: 0.0, canvas_height: 0.0,
            renderer: None,
            active_tool: Box::new(BrushTool::new()), 
            graph: AnimGraph::new(), history: CommandHistory::new(), id_allocator: IdAllocator::new(),
            cursor_manager: CursorManager::new(),
            clipboard: Vec::new(),
        })
    }

    #[wasm_bindgen] pub fn get_system_status(&self) -> String { if self.is_ready { String::from("AnimLab Rust Core: Online.") } else { String::from("AnimLab Rust Core: FATAL OFFLINE.") } }
    #[wasm_bindgen] pub fn set_brush_settings(&mut self, thickness: f32, r: f32, g: f32, b: f32, a: f32) { settings::update_settings(settings::EngineSettings { brush_thickness: thickness, brush_color: [r, g, b, a], smoothing_level: settings::get_settings().smoothing_level }); }
    #[wasm_bindgen] pub fn trigger_undo(&mut self) { self.history.undo(&mut self.graph, self.canvas_width, self.canvas_height); }
    #[wasm_bindgen] pub fn trigger_redo(&mut self) { self.history.redo(&mut self.graph, self.canvas_width, self.canvas_height); }

    #[wasm_bindgen]
    pub fn set_active_tool(&mut self, tool_name: &str) {
        info!("Rust Engine processing Semantic Tool Swap: {}", tool_name);
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Some(body) = doc.body() { let _ = body.style().set_property("cursor", "crosshair"); }
            }
        }
        match tool_name {
            "ToolBrush" => self.active_tool = Box::new(BrushTool::new()),
            "ToolPencil" => self.active_tool = Box::new(PencilTool::new()), 
            "ToolEraser" => self.active_tool = Box::new(EraserTool::new()), 
            "ToolCutter" => self.active_tool = Box::new(CutterTool::new()), 
            "ToolSelect" => self.active_tool = Box::new(SelectTool::new()), 
            _ => { warn!("Tool [{}] safely defaulting to Brush.", tool_name); self.active_tool = Box::new(BrushTool::new()); }
        }
        let target_cursor = self.active_tool.get_cursor();
        self.cursor_manager.apply(target_cursor);
    }

    #[wasm_bindgen]
    pub async fn attach_canvas(&mut self, canvas: HtmlCanvasElement, physical_width: u32, physical_height: u32) -> Result<(), JsValue> {
        self.canvas_width = physical_width as f32; self.canvas_height = physical_height as f32;
        let renderer = WebGpuRenderer::new(canvas, physical_width, physical_height).await.map_err(|e| JsValue::from_str(&e))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    #[wasm_bindgen] 
    pub fn resize_surface(&mut self, physical_width: u32, physical_height: u32) {
        let safe_width = physical_width.max(1); let safe_height = physical_height.max(1);
        self.canvas_width = safe_width as f32; self.canvas_height = safe_height as f32;
        if let Some(renderer) = &mut self.renderer { renderer.resize(safe_width, safe_height); }
    }

    #[wasm_bindgen] pub fn hover(&mut self, x: f32, y: f32, constrain: bool, center: bool) -> Result<(), JsValue> {
        let active_node_id = self.graph.active_layer_node.expect("Fatal Engine Error: Missing active layer.");
        let tool = &mut self.active_tool; let graph = &mut self.graph;
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_hover(x, y, constrain, center, active_node_id, graph); })).unwrap_or_else(|_| {});
        Ok(())
    }

    #[wasm_bindgen] pub fn begin_stroke(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool) -> Result<(), JsValue> { 
        let settings = settings::get_settings(); 
        let active_node_id = self.graph.active_layer_node.expect("Fatal Engine Error: Missing active layer.");
        self.active_tool.on_pointer_down(x, y, pressure, constrain, center, settings, active_node_id, &mut self.graph); 
        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor);
        Ok(()) 
    }
    
    #[wasm_bindgen] pub fn push_point(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool) -> Result<(), JsValue> {
        let active_node_id = self.graph.active_layer_node.expect("Fatal Engine Error: Missing active layer.");
        let canvas_w = self.canvas_width; let canvas_h = self.canvas_height;
        let tool = &mut self.active_tool; let graph = &mut self.graph;

        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            tool.on_pointer_move(x, y, pressure, constrain, center, active_node_id, graph, canvas_w, canvas_h);
        })).unwrap_or_else(|_| { error!("AAA Safety Net: Handled math panic in push_point."); });
        
        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor);
        Ok(())
    }
    
    #[wasm_bindgen] pub fn end_stroke(&mut self) -> Result<(), JsValue> {
        let target_node_id = self.graph.active_layer_node.expect("Fatal Engine Error: Missing active layer.");
        let canvas_w = self.canvas_width; let canvas_h = self.canvas_height;
        let tool = &mut self.active_tool; let allocator = &mut self.id_allocator; let graph = &mut self.graph;

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_up(target_node_id, allocator, canvas_w, canvas_h, graph) }));
        match result {
            Ok(Some(command)) => self.history.push_and_execute(command, &mut self.graph, canvas_w, canvas_h),
            Ok(None) => {}, Err(_) => { error!("AAA Safety Net: Handled geometric panic during final slice."); }
        }

        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&mut self) {
        let target_cursor = self.active_tool.get_cursor();
        self.cursor_manager.apply(target_cursor);

        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.graph, self.active_tool.as_ref(), self.canvas_width, self.canvas_height);
        }
    }

    #[wasm_bindgen]
    pub fn has_selection(&self) -> bool { !self.graph.selected_strokes.is_empty() }

    #[wasm_bindgen]
    pub fn select_all(&mut self) {
        let target_node_id = self.graph.active_layer_node.unwrap_or(1);
        self.graph.selected_strokes.clear();
        if let Some(node) = self.graph.nodes.get(&target_node_id) {
            if let crate::graph::NodeType::VectorLayer { elements, .. } = &node.payload {
                for (id, _) in elements.iter() { self.graph.selected_strokes.insert(*id); }
            }
        }
    }

    #[wasm_bindgen]
    pub fn flip_selection(&mut self, flip_h: bool, flip_v: bool) -> Result<(), JsValue> {
        let target_node_id = self.graph.active_layer_node.unwrap_or(1);
        if self.graph.selected_strokes.is_empty() { return Ok(()); }

        if let Some(aabb) = self.graph.get_selection_aabb(target_node_id) {
            let cx = aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0;
            let cy = aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0;
            let sx = if flip_h { -1.0 } else { 1.0 };
            let sy = if flip_v { -1.0 } else { 1.0 };

            let mut old_elements = Vec::new();
            let mut new_elements = Vec::new();

            if let Some(node) = self.graph.nodes.get_mut(&target_node_id) {
                if let crate::graph::NodeType::VectorLayer { elements, .. } = &mut node.payload {
                    for id in &self.graph.selected_strokes {
                        if let Some(el) = elements.get(id) {
                            old_elements.push((*id, el.clone()));
                            let mut new_el = el.clone();
                            new_el.transform(0.0, 0.0, sx, sy, 0.0, cx, cy, self.canvas_width, self.canvas_height);
                            new_elements.push((*id, new_el.clone()));
                            elements.insert(*id, new_el);
                        }
                    }
                }
            }

            if !new_elements.is_empty() {
                let cmd = Box::new(AffineCommand { target_node_id, old_elements, new_elements });
                self.history.push_and_execute(cmd, &mut self.graph, self.canvas_width, self.canvas_height);
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn delete_selection(&mut self) -> Result<(), JsValue> {
        let target_node_id = self.graph.active_layer_node.unwrap_or(1);
        if self.graph.selected_strokes.is_empty() { return Ok(()); }
        
        let mut severed_fragments = Vec::new();
        if let Some(node) = self.graph.nodes.get(&target_node_id) {
            if let crate::graph::NodeType::VectorLayer { elements, .. } = &node.payload {
                for id in &self.graph.selected_strokes {
                    if let Some(el) = elements.get(id) { severed_fragments.push((*id, el.clone())); }
                }
            }
        }
        
        for (stroke_id, original_element) in severed_fragments {
            let cmd = Box::new(CutCommand { target_node_id, severed_stroke_id: stroke_id, original_element, new_fragments: Vec::new() });
            self.history.push_and_execute(cmd, &mut self.graph, self.canvas_width, self.canvas_height);
        }
        
        self.graph.selected_strokes.clear();
        Ok(())
    }

    #[wasm_bindgen]
    pub fn copy_selection(&mut self) {
        info!("Rust Engine processing Copy action.");
        self.clipboard.clear();
        let target_node_id = self.graph.active_layer_node.unwrap_or(1);
        if let Some(node) = self.graph.nodes.get(&target_node_id) {
            if let crate::graph::NodeType::VectorLayer { elements, .. } = &node.payload {
                for id in &self.graph.selected_strokes {
                    if let Some(el) = elements.get(id) { self.clipboard.push(el.clone()); }
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn cut_selection(&mut self) {
        self.copy_selection();
        let _ = self.delete_selection();
    }

    #[wasm_bindgen]
    pub fn paste_clipboard(&mut self) -> Result<(), JsValue> {
        info!("Rust Engine processing Paste action.");
        if self.clipboard.is_empty() { return Ok(()); }

        let target_node_id = self.graph.active_layer_node.unwrap_or(1);
        let mut commands: Vec<Box<dyn crate::command::Command>> = Vec::new();
        
        self.graph.selected_strokes.clear();
        
        for el in &self.clipboard {
            let new_id = self.id_allocator.generate();
            self.graph.selected_strokes.insert(new_id);
            
            let mut pasted_el = el.clone();
            // AAA FEATURE: Offset pasted element by 20 pixels down/right so it is visually distinguishable
            pasted_el.translate(20.0, 20.0, self.canvas_width, self.canvas_height); 
            
            commands.push(Box::new(AddStrokeCommand { target_node_id, stroke_id: new_id, element: pasted_el }));
        }

        let batch = Box::new(BatchCommand { commands });
        self.history.push_and_execute(batch, &mut self.graph, self.canvas_width, self.canvas_height);
        
        Ok(())
    }

    #[wasm_bindgen] pub fn group_selection(&mut self) { info!("Rust Engine processing Group action."); }
    #[wasm_bindgen] pub fn ungroup_selection(&mut self) { info!("Rust Engine processing Ungroup action."); }
}