use wasm_bindgen::prelude::*;
use log::{info, warn, error, Level};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt; 
use std::collections::HashMap;
use std::panic;

pub mod math;
pub mod geometry; 
pub mod settings; 
pub mod graph; 
pub mod command; 
pub mod tools; 

use graph::{AnimGraph, IdAllocator};
use command::CommandHistory;
use tools::{CanvasTool, brush::BrushTool, pencil::PencilTool, eraser::EraserTool, cutter::CutterTool, select::SelectTool};
use geometry::VectorElement;

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
    
    #[wasm_bindgen(skip)] pub device: Option<wgpu::Device>,
    #[wasm_bindgen(skip)] pub queue: Option<wgpu::Queue>,
    #[wasm_bindgen(skip)] pub surface: Option<wgpu::Surface<'static>>,
    #[wasm_bindgen(skip)] pub config: Option<wgpu::SurfaceConfiguration>,
    
    #[wasm_bindgen(skip)] pub standard_pipeline: Option<wgpu::RenderPipeline>,
    #[wasm_bindgen(skip)] pub stencil_write_pipeline: Option<wgpu::RenderPipeline>, 
    #[wasm_bindgen(skip)] pub stencil_read_0_pipeline: Option<wgpu::RenderPipeline>, 
    #[wasm_bindgen(skip)] pub stencil_read_1_pipeline: Option<wgpu::RenderPipeline>, 
    #[wasm_bindgen(skip)] pub depth_stencil_texture: Option<wgpu::TextureView>,
    
    #[wasm_bindgen(skip)] pub raster_pipeline: Option<wgpu::RenderPipeline>,
    #[wasm_bindgen(skip)] pub raster_bind_group_layout: Option<wgpu::BindGroupLayout>,
    #[wasm_bindgen(skip)] pub raster_cache: HashMap<graph::NodeId, (wgpu::Texture, wgpu::BindGroup)>,
    #[wasm_bindgen(skip)] pub quad_vb: Option<wgpu::Buffer>,
    #[wasm_bindgen(skip)] pub quad_ib: Option<wgpu::Buffer>,
    
    #[wasm_bindgen(skip)] pub active_tool: Box<dyn CanvasTool>,
    #[wasm_bindgen(skip)] pub graph: AnimGraph,
    #[wasm_bindgen(skip)] pub history: CommandHistory,
    #[wasm_bindgen(skip)] pub id_allocator: IdAllocator,
    #[wasm_bindgen(skip)] pub cursor_manager: CursorManager,
}

#[wasm_bindgen]
impl AnimLabEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<AnimLabEngine, JsValue> {
        Ok(AnimLabEngine {
            is_ready: true, canvas_width: 0.0, canvas_height: 0.0,
            device: None, queue: None, surface: None, config: None,
            standard_pipeline: None, stencil_write_pipeline: None, stencil_read_0_pipeline: None, stencil_read_1_pipeline: None, depth_stencil_texture: None,
            raster_pipeline: None, raster_bind_group_layout: None,
            raster_cache: HashMap::new(), quad_vb: None, quad_ib: None,
            active_tool: Box::new(BrushTool::new()), 
            graph: AnimGraph::new(), history: CommandHistory::new(), id_allocator: IdAllocator::new(),
            cursor_manager: CursorManager::new(),
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
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas)).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, compatible_surface: Some(&surface), force_fallback_adapter: false }).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { label: Some("AnimLab GPU Device"), required_features: wgpu::Features::empty(), required_limits: adapter.limits(), ..Default::default() }).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let caps = surface.get_capabilities(&adapter); let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]); 
        
        let safe_width = physical_width.max(1); let safe_height = physical_height.max(1);
        let config = wgpu::SurfaceConfiguration { usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format, width: safe_width, height: safe_height, present_mode: wgpu::PresentMode::Fifo, alpha_mode: caps.alpha_modes[0], view_formats: vec![], desired_maximum_frame_latency: 2 };
        surface.configure(&device, &config);
        
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Vector Pipeline Layout"), bind_group_layouts: &[], immediate_size: 0 });
        let blend_state_normal = wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add }, alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add } };

        let standard_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Standard Vector Pipeline"), layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[math::Vertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state_normal), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth24PlusStencil8, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: wgpu::StencilState::default(), bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        // AAA FIX: Stencil States updated to act on BOTH Front and Back faces
        let stencil_write_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Replace,
            depth_fail_op: wgpu::StencilOperation::Replace,
            pass_op: wgpu::StencilOperation::Replace
        };
        let stencil_write_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stencil Write Pipeline"), layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[math::Vertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state_normal), write_mask: wgpu::ColorWrites::empty() })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth24PlusStencil8, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: wgpu::StencilState { front: stencil_write_face.clone(), back: stencil_write_face, read_mask: !0, write_mask: !0 }, bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        let stencil_read_0_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep
        };
        let stencil_read_0_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stencil Read 0 Pipeline"), layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[math::Vertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state_normal), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth24PlusStencil8, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: wgpu::StencilState { front: stencil_read_0_face.clone(), back: stencil_read_0_face, read_mask: !0, write_mask: 0 }, bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        let stencil_read_1_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep
        };
        let stencil_read_1_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stencil Read 1 Pipeline"), layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[math::Vertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state_normal), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth24PlusStencil8, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: wgpu::StencilState { front: stencil_read_1_face.clone(), back: stencil_read_1_face, read_mask: !0, write_mask: 0 }, bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        let depth_stencil_texture = device.create_texture(&wgpu::TextureDescriptor { size: wgpu::Extent3d { width: safe_width, height: safe_height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Depth24PlusStencil8, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, label: Some("Depth Stencil Texture"), view_formats: &[] }).create_view(&wgpu::TextureViewDescriptor::default());

        let raster_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { entries: &[ wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2, sample_type: wgpu::TextureSampleType::Float { filterable: true } }, count: None }, wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None } ], label: Some("raster_bind_group_layout") });
        let raster_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Raster Pipeline Layout"), bind_group_layouts: &[&raster_bind_group_layout], immediate_size: 0 });
        let raster_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raster Pipeline"), layout: Some(&raster_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_raster"), buffers: &[math::RasterVertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_raster"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state_normal), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth24PlusStencil8, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: wgpu::StencilState::default(), bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        let quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Quad VB"), contents: bytemuck::cast_slice(math::FULLSCREEN_QUAD_VERTS), usage: wgpu::BufferUsages::VERTEX });
        let quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Quad IB"), contents: bytemuck::cast_slice(math::FULLSCREEN_QUAD_INDS), usage: wgpu::BufferUsages::INDEX });
        
        self.device = Some(device); self.queue = Some(queue); self.surface = Some(surface); self.config = Some(config); 
        self.standard_pipeline = Some(standard_pipeline); self.stencil_write_pipeline = Some(stencil_write_pipeline); self.stencil_read_0_pipeline = Some(stencil_read_0_pipeline); self.stencil_read_1_pipeline = Some(stencil_read_1_pipeline); self.depth_stencil_texture = Some(depth_stencil_texture);
        self.raster_pipeline = Some(raster_pipeline); self.raster_bind_group_layout = Some(raster_bind_group_layout); self.quad_vb = Some(quad_vb); self.quad_ib = Some(quad_ib);
        Ok(())
    }

    #[wasm_bindgen] pub fn resize_surface(&mut self, physical_width: u32, physical_height: u32) {
        let safe_width = physical_width.max(1); let safe_height = physical_height.max(1);
        self.canvas_width = safe_width as f32; self.canvas_height = safe_height as f32;
        if let (Some(device), Some(surface), Some(config)) = (&self.device, &self.surface, &mut self.config) {
            config.width = safe_width; config.height = safe_height; surface.configure(device, config);
            self.depth_stencil_texture = Some(device.create_texture(&wgpu::TextureDescriptor { size: wgpu::Extent3d { width: safe_width, height: safe_height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Depth24PlusStencil8, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, label: Some("Depth Stencil Texture"), view_formats: &[] }).create_view(&wgpu::TextureViewDescriptor::default()));
        }
    }

    #[wasm_bindgen] pub fn hover(&mut self, x: f32, y: f32, constrain: bool, center: bool) -> Result<(), JsValue> {
        let active_node_id = self.graph.active_layer_node.expect("Fatal Engine Error: Missing active layer.");
        let tool = &mut self.active_tool;
        let graph = &mut self.graph;
        
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            tool.on_pointer_hover(x, y, constrain, center, active_node_id, graph);
        })).unwrap_or_else(|_| {});
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
        let canvas_w = self.canvas_width;
        let canvas_h = self.canvas_height;
        let tool = &mut self.active_tool;
        let graph = &mut self.graph;

        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            tool.on_pointer_move(x, y, pressure, constrain, center, active_node_id, graph, canvas_w, canvas_h);
        })).unwrap_or_else(|_| { error!("AAA Safety Net: Handled math panic in push_point."); });
        
        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor);
        Ok(())
    }
    
    #[wasm_bindgen] pub fn end_stroke(&mut self) -> Result<(), JsValue> {
        let target_node_id = self.graph.active_layer_node.expect("Fatal Engine Error: Missing active layer.");
        let canvas_w = self.canvas_width;
        let canvas_h = self.canvas_height;
        let tool = &mut self.active_tool;
        let allocator = &mut self.id_allocator;
        let graph = &mut self.graph;

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            tool.on_pointer_up(target_node_id, allocator, canvas_w, canvas_h, graph)
        }));

        match result {
            Ok(Some(command)) => self.history.push_and_execute(command, &mut self.graph, canvas_w, canvas_h),
            Ok(None) => {}, 
            Err(_) => { error!("AAA Safety Net: Handled geometric panic during final slice."); }
        }

        let cursor = self.active_tool.get_cursor(); self.cursor_manager.apply(cursor);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&mut self) {
        let target_cursor = self.active_tool.get_cursor();
        self.cursor_manager.apply(target_cursor);

        if let (Some(device), Some(queue), Some(surface), Some(depth_stencil)) = (&self.device, &self.queue, &self.surface, &self.depth_stencil_texture) {
            
            let output = match surface.get_current_texture() { Ok(frame) => frame, Err(wgpu::SurfaceError::Outdated) => return, Err(e) => { error!("Surface error: {:?}", e); return; } };
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            
            {
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"), 
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.09, b: 0.10, a: 1.0 }), store: wgpu::StoreOp::Store }, depth_slice: None })], 
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: depth_stencil, depth_ops: None, stencil_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0), store: wgpu::StoreOp::Store }) }), 
                    timestamp_writes: None, occlusion_query_set: None, ..Default::default()
                });
                
                let draw = |rp_ref: &mut wgpu::RenderPass<'_>, vertices: &[math::Vertex], indices: &[u16]| {
                    if vertices.is_empty() { return; }
                    let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("VB"), contents: bytemuck::cast_slice(vertices), usage: wgpu::BufferUsages::VERTEX });
                    let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("IB"), contents: bytemuck::cast_slice(indices), usage: wgpu::BufferUsages::INDEX });
                    rp_ref.set_vertex_buffer(0, vb.slice(..)); rp_ref.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16); rp_ref.draw_indexed(0..indices.len() as u32, 0, 0..1);
                };

                if let (Some(standard_pipe), Some(stencil_write), Some(stencil_read_0), Some(stencil_read_1)) = 
                    (&self.standard_pipeline, &self.stencil_write_pipeline, &self.stencil_read_0_pipeline, &self.stencil_read_1_pipeline) {
                    
                    let render_list = self.graph.collect_renderable_elements();
                    for element in render_list { 
                        match element {
                            VectorElement::Centerline(c) => {
                                rp.set_pipeline(standard_pipe); draw(&mut rp, &c.vertices, &c.indices);
                            }
                            VectorElement::Contour(c) => {
                                if c.clip_masks.is_empty() && c.eraser_masks.is_empty() {
                                    rp.set_pipeline(standard_pipe); draw(&mut rp, &c.vertices, &c.indices);
                                } else if c.clip_masks.is_empty() {
                                    rp.set_pipeline(stencil_write);
                                    rp.set_stencil_reference(1);
                                    for mask in &c.eraser_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                                    
                                    rp.set_pipeline(stencil_read_0);
                                    rp.set_stencil_reference(0);
                                    draw(&mut rp, &c.vertices, &c.indices);

                                    rp.set_pipeline(stencil_write);
                                    rp.set_stencil_reference(0);
                                    for mask in &c.eraser_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                                } else {
                                    rp.set_pipeline(stencil_write);
                                    rp.set_stencil_reference(1);
                                    for mask in &c.clip_masks { draw(&mut rp, &mask.vertices, &mask.indices); }

                                    rp.set_stencil_reference(0);
                                    for mask in &c.eraser_masks { draw(&mut rp, &mask.vertices, &mask.indices); }

                                    rp.set_pipeline(stencil_read_1);
                                    rp.set_stencil_reference(1);
                                    draw(&mut rp, &c.vertices, &c.indices);

                                    rp.set_pipeline(stencil_write);
                                    rp.set_stencil_reference(0);
                                    for mask in &c.clip_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                                }
                            }
                        }
                    }
                    
                    let target_node_id = self.graph.active_layer_node.unwrap_or(1);
                    if let Some(aabb) = self.graph.get_selection_aabb(target_node_id) {
                        let mut bb_verts = Vec::new();
                        let mut bb_inds = Vec::new();
                        let tb_orange = [1.0, 0.45, 0.0, 1.0];
                        
                        let pts = vec![
                            geometry::Point { x: aabb.min_x, y: aabb.min_y, pressure: 1.0 },
                            geometry::Point { x: aabb.max_x, y: aabb.min_y, pressure: 1.0 },
                            geometry::Point { x: aabb.max_x, y: aabb.max_y, pressure: 1.0 },
                            geometry::Point { x: aabb.min_x, y: aabb.max_y, pressure: 1.0 },
                            geometry::Point { x: aabb.min_x, y: aabb.min_y, pressure: 1.0 },
                        ];
                        
                        let (box_v, box_i, _) = geometry::tessellator::Extruder::extrude_centerline(&pts, 1.0, tb_orange, self.canvas_width, self.canvas_height);
                        
                        let offset = bb_verts.len() as u16;
                        bb_verts.extend(box_v);
                        for idx in box_i { bb_inds.push(idx + offset); }

                        let mut add_quad = |x: f32, y: f32, w: f32, h: f32, color: [f32; 4]| {
                            let left = (x / self.canvas_width) * 2.0 - 1.0;
                            let right = ((x + w) / self.canvas_width) * 2.0 - 1.0;
                            let top = 1.0 - (y / self.canvas_height) * 2.0;
                            let bottom = 1.0 - ((y + h) / self.canvas_height) * 2.0;

                            let start_idx = bb_verts.len() as u16;
                            bb_verts.push(math::Vertex { position: [left, top], color, tex_coords: [0.0, 0.0] });
                            bb_verts.push(math::Vertex { position: [right, top], color, tex_coords: [1.0, 0.0] });
                            bb_verts.push(math::Vertex { position: [right, bottom], color, tex_coords: [1.0, 1.0] });
                            bb_verts.push(math::Vertex { position: [left, bottom], color, tex_coords: [0.0, 1.0] });
                            bb_inds.extend_from_slice(&[start_idx, start_idx + 1, start_idx + 2, start_idx, start_idx + 2, start_idx + 3]);
                        };

                        let hs = 3.0; 
                        let b_hs = 4.0; 
                        let coords = [
                            (aabb.min_x, aabb.min_y), (aabb.max_x, aabb.min_y), (aabb.max_x, aabb.max_y), (aabb.min_x, aabb.max_y),
                            ((aabb.min_x + aabb.max_x) / 2.0, aabb.min_y), ((aabb.min_x + aabb.max_x) / 2.0, aabb.max_y),
                            (aabb.min_x, (aabb.min_y + aabb.max_y) / 2.0), (aabb.max_x, (aabb.min_y + aabb.max_y) / 2.0)
                        ];

                        for (hx, hy) in coords {
                            add_quad(hx - b_hs, hy - b_hs, b_hs * 2.0, b_hs * 2.0, tb_orange); 
                            add_quad(hx - hs, hy - hs, hs * 2.0, hs * 2.0, [1.0, 1.0, 1.0, 1.0]); 
                        }

                        let (px, py) = self.active_tool.get_custom_pivot().unwrap_or((
                            aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0,
                            aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0
                        ));
                        
                        add_quad(px - 5.0, py - 5.0, 10.0, 10.0, [0.0, 0.6, 1.0, 1.0]); 
                        add_quad(px - 2.0, py - 2.0, 4.0, 4.0, [1.0, 1.0, 1.0, 1.0]);   

                        rp.set_pipeline(standard_pipe);
                        draw(&mut rp, &bb_verts, &bb_inds);
                    }

                    let preview_mesh_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        self.active_tool.get_preview_mesh(self.canvas_width, self.canvas_height)
                    }));
                    
                    let (preview_verts, preview_inds) = match preview_mesh_result {
                        Ok(mesh) => mesh,
                        Err(_) => {
                            error!("AAA Safety Net: Handled rendering panic in get_preview_mesh.");
                            (Vec::new(), Vec::new())
                        }
                    };

                    rp.set_pipeline(standard_pipe);
                    draw(&mut rp, &preview_verts, &preview_inds);
                }
            }
            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    }
}