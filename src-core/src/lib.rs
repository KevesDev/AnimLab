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
pub mod operations;

use graph::{SceneManager, IdAllocator, DrawingElement, DrawingData, ExposureBlock};
use command::CommandHistory;
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
    is_ready: bool, canvas_width: f32, canvas_height: f32,
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
        
        // AAA FIX: Start with a single, clean 'Drawing' layer (No hardcoded placeholders)
        let el_id = id_allocator.generate();
        let mut el = DrawingElement::new(el_id, "Drawing".to_string());
        
        let draw_id = id_allocator.generate();
        el.library.insert(draw_id, DrawingData::new());
        el.exposures.insert(1, ExposureBlock { drawing_id: draw_id, start_frame: 1, duration: 1 });
        
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
    
    #[wasm_bindgen] pub fn set_brush_settings(&mut self, thickness: f32, r: f32, g: f32, b: f32, a: f32) { settings::update_settings(settings::EngineSettings { brush_thickness: thickness, brush_color: [r, g, b, a], smoothing_level: settings::get_settings().smoothing_level }); self.render(); }
    #[wasm_bindgen] pub fn trigger_undo(&mut self) { self.history.undo(&mut self.scene, self.canvas_width, self.canvas_height); self.render(); }
    #[wasm_bindgen] pub fn trigger_redo(&mut self) { self.history.redo(&mut self.scene, self.canvas_width, self.canvas_height); self.render(); }

    #[wasm_bindgen]
    pub fn set_active_tool(&mut self, tool_name: &str) {
        if let Some(window) = web_sys::window() { if let Some(doc) = window.document() { if let Some(body) = doc.body() { let _ = body.style().set_property("cursor", "crosshair"); } } }
        match tool_name {
            "ToolBrush" => self.active_tool = Box::new(BrushTool::new()), "ToolPencil" => self.active_tool = Box::new(PencilTool::new()), 
            "ToolEraser" => self.active_tool = Box::new(EraserTool::new()), "ToolCutter" => self.active_tool = Box::new(CutterTool::new()), 
            "ToolSelect" => self.active_tool = Box::new(SelectTool::new()), _ => { self.active_tool = Box::new(BrushTool::new()); }
        }
        let target_cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(target_cursor); self.render();
    }

    #[wasm_bindgen]
    pub async fn attach_canvas(&mut self, canvas: HtmlCanvasElement, physical_width: u32, physical_height: u32) -> Result<(), JsValue> {
        self.canvas_width = physical_width as f32; self.canvas_height = physical_height as f32;
        let renderer = WebGpuRenderer::new(canvas, physical_width, physical_height).await.map_err(|e| JsValue::from_str(&e))?;
        self.renderer = Some(renderer); Ok(())
    }

    #[wasm_bindgen] pub fn resize_surface(&mut self, physical_width: u32, physical_height: u32) { let safe_width = physical_width.max(1); let safe_height = physical_height.max(1); self.canvas_width = safe_width as f32; self.canvas_height = safe_height as f32; if let Some(renderer) = &mut self.renderer { renderer.resize(safe_width, safe_height); } }
    #[wasm_bindgen] pub fn hover(&mut self, x: f32, y: f32, constrain: bool, center: bool) -> Result<(), JsValue> { let tool = &mut self.active_tool; let scene = &mut self.scene; let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_hover(x, y, constrain, center, scene); })).unwrap_or_else(|_| {}); self.render(); Ok(()) }
    #[wasm_bindgen] pub fn begin_stroke(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool) -> Result<(), JsValue> { self.scene.ensure_drawing_exists(&mut self.id_allocator); let settings = settings::get_settings(); self.active_tool.on_pointer_down(x, y, pressure, constrain, center, settings, &mut self.scene, &mut self.id_allocator); let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor); self.render(); Ok(()) }
    #[wasm_bindgen] pub fn push_point(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool) -> Result<(), JsValue> { let canvas_w = self.canvas_width; let canvas_h = self.canvas_height; let tool = &mut self.active_tool; let scene = &mut self.scene; let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_move(x, y, pressure, constrain, center, scene, canvas_w, canvas_h); })); let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor); self.render(); Ok(()) }
    #[wasm_bindgen] pub fn end_stroke(&mut self) -> Result<(), JsValue> { let canvas_w = self.canvas_width; let canvas_h = self.canvas_height; let tool = &mut self.active_tool; let allocator = &mut self.id_allocator; let scene = &mut self.scene; let result = panic::catch_unwind(panic::AssertUnwindSafe(|| { tool.on_pointer_up(allocator, canvas_w, canvas_h, scene) })); if let Ok(Some(command)) = result { self.history.push_and_execute(command, &mut self.scene, canvas_w, canvas_h); } let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor); self.render(); Ok(()) }
    #[wasm_bindgen] pub fn render(&mut self) { let target_cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(target_cursor); if let Some(renderer) = &mut self.renderer { renderer.render(&self.scene, self.active_tool.as_ref(), self.canvas_width, self.canvas_height); } }

    #[wasm_bindgen] pub fn has_selection(&self) -> bool { !self.scene.selected_strokes.is_empty() }
    #[wasm_bindgen] pub fn select_all(&mut self) { operations::selection::select_all(&mut self.scene); self.render(); }
    #[wasm_bindgen] pub fn flip_selection(&mut self, flip_h: bool, flip_v: bool) -> Result<(), JsValue> { operations::selection::flip_selection(&mut self.scene, &mut self.history, self.canvas_width, self.canvas_height, flip_h, flip_v); self.render(); Ok(()) }
    #[wasm_bindgen] pub fn delete_selection(&mut self) -> Result<(), JsValue> { operations::selection::delete_selection(&mut self.scene, &mut self.history, self.canvas_width, self.canvas_height); self.render(); Ok(()) }
    #[wasm_bindgen] pub fn copy_selection(&mut self) { operations::selection::copy_selection(&self.scene, &mut self.clipboard); }
    #[wasm_bindgen] pub fn cut_selection(&mut self) { self.copy_selection(); let _ = self.delete_selection(); }
    #[wasm_bindgen] pub fn paste_clipboard(&mut self) -> Result<(), JsValue> { operations::selection::paste_clipboard(&mut self.scene, &mut self.history, &mut self.id_allocator, &self.clipboard, self.canvas_width, self.canvas_height); self.render(); Ok(()) }
    #[wasm_bindgen] pub fn group_selection(&mut self) {} #[wasm_bindgen] pub fn ungroup_selection(&mut self) {}

    #[wasm_bindgen] pub fn set_active_art_layer(&mut self, layer_index: u8) { operations::layers::set_active_art_layer(&mut self.scene, layer_index); self.render(); }
    #[wasm_bindgen] pub fn set_layer_opacity(&mut self, element_id: u64, opacity: f32) { operations::layers::set_opacity(&mut self.scene, element_id, opacity); self.render(); }
    #[wasm_bindgen] pub fn set_layer_visibility(&mut self, element_id: u64, is_visible: bool) { operations::layers::set_visibility(&mut self.scene, element_id, is_visible); self.render(); }

    #[wasm_bindgen] pub fn set_current_frame(&mut self, frame: u32) { self.scene.current_frame = frame; self.render(); }
    #[wasm_bindgen] pub fn set_exposure(&mut self, element_id: u64, start_frame: u32, duration: u32, drawing_id: u64) { operations::exposure::set_exposure(&mut self.scene, element_id, start_frame, duration, drawing_id); self.render(); }
    #[wasm_bindgen] pub fn split_exposure(&mut self, element_id: u64, cut_frame: u32) { operations::exposure::split_exposure(&mut self.scene, element_id, cut_frame); self.render(); }
    #[wasm_bindgen] pub fn clear_exposure(&mut self, element_id: u64, start_frame: u32, duration: u32) { operations::exposure::clear_exposure(&mut self.scene, element_id, start_frame, duration); self.render(); }
    #[wasm_bindgen] pub fn update_exposure(&mut self, element_id: u64, old_start: u32, new_start: u32, new_duration: u32) { operations::exposure::update_exposure(&mut self.scene, element_id, old_start, new_start, new_duration); self.render(); }

    // AAA FIX: WASM bindings for the Wizard commands and dynamic lengths
    #[wasm_bindgen] pub fn get_scene_length(&self) -> u32 { self.scene.get_scene_length() }
    
    #[wasm_bindgen]
    pub fn add_drawing_layer(&mut self, name: &str, init_mode: u8) {
        if self.scene.z_stack.len() >= 100 { return; } // Strict hard cap
        let el_id = self.id_allocator.generate();
        let mut el = DrawingElement::new(el_id, name.to_string());
        
        let scene_length = self.scene.get_scene_length();
        if init_mode == 1 || init_mode == 2 {
            let draw_id = self.id_allocator.generate();
            el.library.insert(draw_id, DrawingData::new());
            let duration = if init_mode == 2 { scene_length } else { 1 };
            let start_frame = if init_mode == 2 { 1 } else { self.scene.current_frame };
            el.exposures.insert(start_frame, ExposureBlock { drawing_id: draw_id, start_frame, duration });
        }
        
        let cmd = Box::new(crate::command::LayerCommand { element_id: el_id, element: el, is_add: true });
        self.history.push_and_execute(cmd, &mut self.scene, self.canvas_width, self.canvas_height);
        self.scene.active_element_id = Some(el_id);
        self.render();
    }

    #[wasm_bindgen]
    pub fn delete_drawing_layer(&mut self, element_id: u64) {
        if let Some(el) = self.scene.elements.get(&element_id) {
            let cmd = Box::new(crate::command::LayerCommand { element_id, element: el.clone(), is_add: false });
            self.history.push_and_execute(cmd, &mut self.scene, self.canvas_width, self.canvas_height);
            if self.scene.active_element_id == Some(element_id) {
                self.scene.active_element_id = self.scene.z_stack.last().copied();
            }
            self.render();
        }
    }

    #[wasm_bindgen]
    pub fn get_timeline_layers(&self) -> String {
        let mut layers_json = Vec::new();
        for el_id in self.scene.z_stack.iter().rev() {
            if let Some(el) = self.scene.elements.get(el_id) {
                let name = el.name.replace("\"", "\\\"");
                layers_json.push(format!(r#"{{"id": "{}", "name": "{}"}}"#, el_id, name));
            }
        }
        format!("[{}]", layers_json.join(","))
    }

    #[wasm_bindgen]
    pub fn get_timeline_blocks(&self) -> Vec<u64> {
        let mut flat_data = Vec::with_capacity(1024);
        for el_id in self.scene.z_stack.iter().rev() {
            if let Some(el) = self.scene.elements.get(el_id) {
                flat_data.push(*el_id);
                flat_data.push(el.exposures.len() as u64);
                for (_, block) in &el.exposures {
                    flat_data.push(block.start_frame as u64);
                    flat_data.push(block.duration as u64);
                    flat_data.push(block.drawing_id);
                }
            }
        }
        flat_data
    }
}