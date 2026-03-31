use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use std::panic;

pub mod math;
pub mod geometry; 
pub mod settings; 
pub mod graph; 
pub mod command; 
pub mod tools;
pub mod renderer;

use graph::{SceneManager, IdAllocator, DrawingElement, DrawingData};
use command::{CommandHistory, CutCommand, BatchCommand, AddStrokeCommand, AffineCommand};
use tools::{CanvasTool, brush::BrushTool, pencil::PencilTool, eraser::EraserTool, cutter::CutterTool, select::SelectTool};
use geometry::VectorElement;
use renderer::WebGpuRenderer;

#[derive(Debug)]
pub enum EngineError { LoggerInitFailed(String) }
impl From<EngineError> for JsValue { fn from(err: EngineError) -> JsValue { match err { EngineError::LoggerInitFailed(msg) => JsValue::from_str(&format!("AnimLab Fatal: {}", msg)), } } }

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
    console_log::init_with_level(log::Level::Debug).map_err(|e| EngineError::LoggerInitFailed(e.to_string()))?;
    Ok(())
}

#[wasm_bindgen]
pub struct AnimLabEngine {
    is_ready: bool,
    canvas_width: f32,
    canvas_height: f32,
    
    #[wasm_bindgen(skip)] pub renderer: Option<WebGpuRenderer>,
    #[wasm_bindgen(skip)] pub active_tool: Box<dyn CanvasTool>,
    #[wasm_bindgen(skip)] pub scene: SceneManager,
    #[wasm_bindgen(skip)] pub history: CommandHistory,
    #[wasm_bindgen(skip)] pub id_allocator: IdAllocator,
    #[wasm_bindgen(skip)] pub cursor_manager: CursorManager,
    #[wasm_bindgen(skip)] pub clipboard: Vec<VectorElement>, 
}

#[wasm_bindgen]
impl AnimLabEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<AnimLabEngine, JsValue> {
        let mut scene = SceneManager::new();
        let mut id_allocator = IdAllocator::new();
        
        let el_id = id_allocator.generate();
        let mut el = DrawingElement::new(el_id, "Drawing_1".to_string());
        let draw_id = id_allocator.generate();
        el.library.insert(draw_id, DrawingData::new());
        el.exposures.insert(1, draw_id);
        scene.elements.insert(el_id, el);
        scene.z_stack.push(el_id);
        scene.active_element_id = Some(el_id);

        Ok(AnimLabEngine {
            is_ready: true, canvas_width: 0.0, canvas_height: 0.0, renderer: None,
            active_tool: Box::new(BrushTool::new()), scene, history: CommandHistory::new(), id_allocator,
            cursor_manager: CursorManager::new(), clipboard: Vec::new(),
        })
    }

    #[wasm_bindgen] pub fn get_system_status(&self) -> String { if self.is_ready { String::from("AnimLab Rust Core: Online.") } else { String::from("AnimLab Rust Core: FATAL OFFLINE.") } }
    #[wasm_bindgen] pub fn set_brush_settings(&mut self, thickness: f32, r: f32, g: f32, b: f32, a: f32) { settings::update_settings(settings::EngineSettings { brush_thickness: thickness, brush_color: [r, g, b, a], smoothing_level: settings::get_settings().smoothing_level }); }
    #[wasm_bindgen] pub fn trigger_undo(&mut self) { self.history.undo(&mut self.scene, self.canvas_width, self.canvas_height); }
    #[wasm_bindgen] pub fn trigger_redo(&mut self) { self.history.redo(&mut self.scene, self.canvas_width, self.canvas_height); }

    #[wasm_bindgen]
    pub fn set_active_tool(&mut self, tool_name: &str) {
        if let Some(window) = web_sys::window() { if let Some(doc) = window.document() { if let Some(body) = doc.body() { let _ = body.style().set_property("cursor", "crosshair"); } } }
        match tool_name {
            "ToolBrush" => self.active_tool = Box::new(BrushTool::new()),
            "ToolPencil" => self.active_tool = Box::new(PencilTool::new()), 
            "ToolEraser" => self.active_tool = Box::new(EraserTool::new()), 
            "ToolCutter" => self.active_tool = Box::new(CutterTool::new()), 
            "ToolSelect" => self.active_tool = Box::new(SelectTool::new()), 
            _ => { self.active_tool = Box::new(BrushTool::new()); }
        }
        let target_cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(target_cursor);
    }

    #[wasm_bindgen]
    pub async fn attach_canvas(&mut self, canvas: HtmlCanvasElement, physical_width: u32, physical_height: u32) -> Result<(), JsValue> {
        self.canvas_width = physical_width as f32; self.canvas_height = physical_height as f32;
        let renderer = WebGpuRenderer::new(canvas, physical_width, physical_height).await.map_err(|e| JsValue::from_str(&e))?;
        self.renderer = Some(renderer); Ok(())
    }

    #[wasm_bindgen] 
    pub fn resize_surface(&mut self, physical_width: u32, physical_height: u32) {
        let safe_width = physical_width.max(1); let safe_height = physical_height.max(1);
        self.canvas_width = safe_width as f32; self.canvas_height = safe_height as f32;
        if let Some(renderer) = &mut self.renderer { renderer.resize(safe_width, safe_height); }
    }

    #[wasm_bindgen] pub fn hover(&mut self, x: f32, y: f32, constrain: bool, center: bool) -> Result<(), JsValue> {
        let tool = &mut self.active_tool; let scene = &mut self.scene;
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_hover(x, y, constrain, center, scene); })).unwrap_or_else(|_| {});
        Ok(())
    }

    #[wasm_bindgen] pub fn begin_stroke(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool) -> Result<(), JsValue> { 
        self.scene.ensure_drawing_exists(&mut self.id_allocator);
        let settings = settings::get_settings(); 
        self.active_tool.on_pointer_down(x, y, pressure, constrain, center, settings, &mut self.scene, &mut self.id_allocator); 
        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor); Ok(()) 
    }
    
    #[wasm_bindgen] pub fn push_point(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool) -> Result<(), JsValue> {
        let canvas_w = self.canvas_width; let canvas_h = self.canvas_height; let tool = &mut self.active_tool; let scene = &mut self.scene;
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_move(x, y, pressure, constrain, center, scene, canvas_w, canvas_h); }));
        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor); Ok(())
    }
    
    #[wasm_bindgen] pub fn end_stroke(&mut self) -> Result<(), JsValue> {
        let canvas_w = self.canvas_width; let canvas_h = self.canvas_height; let tool = &mut self.active_tool; let allocator = &mut self.id_allocator; let scene = &mut self.scene;
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_up(allocator, canvas_w, canvas_h, scene) }));
        if let Ok(Some(command)) = result { self.history.push_and_execute(command, &mut self.scene, canvas_w, canvas_h); }
        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor); Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&mut self) {
        let target_cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(target_cursor);
        if let Some(renderer) = &mut self.renderer { renderer.render(&self.scene, self.active_tool.as_ref(), self.canvas_width, self.canvas_height); }
    }

    #[wasm_bindgen] pub fn has_selection(&self) -> bool { !self.scene.selected_strokes.is_empty() }
    
    #[wasm_bindgen]
    pub fn select_all(&mut self) {
        self.scene.selected_strokes.clear();
        let ids: Vec<_> = if let Some((_, layer)) = self.scene.get_active_art_layer() { layer.vector_elements.keys().copied().collect() } else { Vec::new() };
        for id in ids { self.scene.selected_strokes.insert(id); }
    }

    #[wasm_bindgen]
    pub fn flip_selection(&mut self, flip_h: bool, flip_v: bool) -> Result<(), JsValue> {
        if self.scene.selected_strokes.is_empty() { return Ok(()); }
        let element_id = self.scene.active_element_id.unwrap_or(1);
        let drawing_id = self.scene.elements.get(&element_id).unwrap().exposures.get(&self.scene.current_frame).copied().unwrap_or(1);

        if let Some(aabb) = self.scene.get_selection_aabb() {
            let cx = aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0; let cy = aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0;
            let sx = if flip_h { -1.0 } else { 1.0 }; let sy = if flip_v { -1.0 } else { 1.0 };
            let mut old_elements = Vec::new(); let mut new_elements = Vec::new();

            let selected: Vec<_> = self.scene.selected_strokes.iter().copied().collect();

            if let Some((_, layer)) = self.scene.get_active_art_layer_mut() {
                for id in &selected {
                    if let Some(el) = layer.vector_elements.get(id) {
                        old_elements.push((*id, el.clone()));
                        let mut new_el = el.clone(); new_el.transform(0.0, 0.0, sx, sy, 0.0, cx, cy, self.canvas_width, self.canvas_height);
                        new_elements.push((*id, new_el.clone())); layer.vector_elements.insert(*id, new_el);
                    }
                }
            }
            if !new_elements.is_empty() {
                let cmd = Box::new(AffineCommand { element_id, drawing_id, art_layer: self.scene.active_art_layer, old_elements, new_elements });
                self.history.push_and_execute(cmd, &mut self.scene, self.canvas_width, self.canvas_height);
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn delete_selection(&mut self) -> Result<(), JsValue> {
        if self.scene.selected_strokes.is_empty() { return Ok(()); }
        let element_id = self.scene.active_element_id.unwrap_or(1);
        let drawing_id = self.scene.elements.get(&element_id).unwrap().exposures.get(&self.scene.current_frame).copied().unwrap_or(1);
        let mut severed_fragments = Vec::new();
        
        let selected: Vec<_> = self.scene.selected_strokes.iter().copied().collect();
        if let Some((_, layer)) = self.scene.get_active_art_layer() {
            for id in &selected { if let Some(el) = layer.vector_elements.get(id) { severed_fragments.push((*id, el.clone())); } }
        }
        
        for (stroke_id, original_element) in severed_fragments {
            let cmd = Box::new(CutCommand { element_id, drawing_id, art_layer: self.scene.active_art_layer, severed_stroke_id: stroke_id, original_element, new_fragments: Vec::new() });
            self.history.push_and_execute(cmd, &mut self.scene, self.canvas_width, self.canvas_height);
        }
        self.scene.selected_strokes.clear(); Ok(())
    }

    #[wasm_bindgen]
    pub fn copy_selection(&mut self) {
        self.clipboard.clear();
        let selected: Vec<_> = self.scene.selected_strokes.iter().copied().collect();
        if let Some((_, layer)) = self.scene.get_active_art_layer() {
            for id in &selected { if let Some(el) = layer.vector_elements.get(id) { self.clipboard.push(el.clone()); } }
        }
    }

    #[wasm_bindgen] pub fn cut_selection(&mut self) { self.copy_selection(); let _ = self.delete_selection(); }

    #[wasm_bindgen]
    pub fn paste_clipboard(&mut self) -> Result<(), JsValue> {
        if self.clipboard.is_empty() { return Ok(()); }
        self.scene.ensure_drawing_exists(&mut self.id_allocator);
        
        let element_id = self.scene.active_element_id.unwrap_or(1);
        let drawing_id = self.scene.elements.get(&element_id).unwrap().exposures.get(&self.scene.current_frame).copied().unwrap_or(1);
        let mut commands: Vec<Box<dyn crate::command::Command>> = Vec::new();
        
        self.scene.selected_strokes.clear();
        for el in &self.clipboard {
            let new_id = self.id_allocator.generate();
            self.scene.selected_strokes.insert(new_id);
            let mut pasted_el = el.clone(); pasted_el.translate(20.0, 20.0, self.canvas_width, self.canvas_height); 
            commands.push(Box::new(AddStrokeCommand { element_id, drawing_id, art_layer: self.scene.active_art_layer, stroke_id: new_id, element: pasted_el }));
        }
        let batch = Box::new(BatchCommand { commands });
        self.history.push_and_execute(batch, &mut self.scene, self.canvas_width, self.canvas_height); Ok(())
    }

    // AAA ARCHITECTURE: Layer and UI Integration Endpoints
    #[wasm_bindgen]
    pub fn set_active_art_layer(&mut self, layer_index: u8) {
        use crate::graph::ArtLayerType;
        self.scene.active_art_layer = match layer_index {
            0 => ArtLayerType::Overlay,
            1 => ArtLayerType::LineArt,
            2 => ArtLayerType::ColorArt,
            3 => ArtLayerType::Underlay,
            _ => ArtLayerType::LineArt,
        };
    }

    #[wasm_bindgen]
    pub fn set_layer_opacity(&mut self, element_id: u64, opacity: f32) {
        if let Some(el) = self.scene.elements.get_mut(&element_id) {
            el.opacity = opacity.clamp(0.0, 1.0);
        }
        self.render(); // This will visually update once the WebGPU composite pipeline is built in the next step
    }

    #[wasm_bindgen]
    pub fn set_layer_visibility(&mut self, element_id: u64, is_visible: bool) {
        if let Some(el) = self.scene.elements.get_mut(&element_id) {
            el.is_visible = is_visible;
        }
        self.render();
    }

    #[wasm_bindgen] pub fn group_selection(&mut self) {}
    #[wasm_bindgen] pub fn ungroup_selection(&mut self) {}
}