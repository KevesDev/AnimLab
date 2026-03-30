use wasm_bindgen::prelude::*;
use log::{info, error, Level};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt; 
use std::collections::HashMap;

pub mod math;
pub mod stroke;
pub mod settings; 
pub mod graph; 
pub mod command; 

use stroke::Stroke;
use graph::AnimGraph;
use command::{CommandHistory, AddStrokeCommand};

#[derive(Debug)]
pub enum EngineError {
    LoggerInitFailed(String),
    InputWithoutActiveStroke,
}

impl From<EngineError> for JsValue {
    fn from(err: EngineError) -> JsValue {
        match err {
            EngineError::LoggerInitFailed(msg) => JsValue::from_str(&format!("AnimLab Fatal: {}", msg)),
            EngineError::InputWithoutActiveStroke => JsValue::from_str("AnimLab Math Error: Attempted to add points to a null stroke."),
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
    
    #[wasm_bindgen(skip)]
    pub device: Option<wgpu::Device>,
    #[wasm_bindgen(skip)]
    pub queue: Option<wgpu::Queue>,
    #[wasm_bindgen(skip)]
    pub surface: Option<wgpu::Surface<'static>>,
    #[wasm_bindgen(skip)]
    pub config: Option<wgpu::SurfaceConfiguration>,
    
    #[wasm_bindgen(skip)]
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    #[wasm_bindgen(skip)]
    pub raster_pipeline: Option<wgpu::RenderPipeline>,
    #[wasm_bindgen(skip)]
    pub raster_bind_group_layout: Option<wgpu::BindGroupLayout>,
    #[wasm_bindgen(skip)]
    pub raster_cache: HashMap<graph::NodeId, (wgpu::Texture, wgpu::BindGroup)>,
    #[wasm_bindgen(skip)]
    pub quad_vb: Option<wgpu::Buffer>,
    #[wasm_bindgen(skip)]
    pub quad_ib: Option<wgpu::Buffer>,
    
    #[wasm_bindgen(skip)]
    pub active_stroke: Option<Stroke>,
    #[wasm_bindgen(skip)]
    pub graph: AnimGraph,
    #[wasm_bindgen(skip)]
    pub history: CommandHistory,
}

#[wasm_bindgen]
impl AnimLabEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<AnimLabEngine, JsValue> {
        Ok(AnimLabEngine {
            is_ready: true,
            canvas_width: 0.0,
            canvas_height: 0.0,
            device: None,
            queue: None,
            surface: None,
            config: None,
            render_pipeline: None,
            raster_pipeline: None,
            raster_bind_group_layout: None,
            raster_cache: HashMap::new(),
            quad_vb: None,
            quad_ib: None,
            active_stroke: None,
            graph: AnimGraph::new(),
            history: CommandHistory::new(),
        })
    }

    #[wasm_bindgen]
    pub fn get_system_status(&self) -> String {
        if self.is_ready { String::from("AnimLab Rust Core: Online.") } 
        else { String::from("AnimLab Rust Core: FATAL OFFLINE.") }
    }

    #[wasm_bindgen]
    pub fn set_brush_settings(&mut self, thickness: f32, r: f32, g: f32, b: f32, a: f32) {
        settings::update_settings(settings::EngineSettings {
            brush_thickness: thickness,
            brush_color: [r, g, b, a],
            smoothing_level: settings::get_settings().smoothing_level,
        });
    }

    #[wasm_bindgen]
    pub fn trigger_undo(&mut self) { self.history.undo(&mut self.graph); }

    #[wasm_bindgen]
    pub fn trigger_redo(&mut self) { self.history.redo(&mut self.graph); }

    #[wasm_bindgen]
    pub async fn attach_canvas(&mut self, canvas: HtmlCanvasElement, physical_width: u32, physical_height: u32) -> Result<(), JsValue> {
        self.canvas_width = physical_width as f32;
        self.canvas_height = physical_height as f32;
        
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas)).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, compatible_surface: Some(&surface), force_fallback_adapter: false }).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { label: Some("AnimLab GPU Device"), required_features: wgpu::Features::empty(), required_limits: adapter.limits(), ..Default::default() }).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]); 
        
        let safe_width = physical_width.max(1);
        let safe_height = physical_height.max(1);

        let config = wgpu::SurfaceConfiguration { usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format, width: safe_width, height: safe_height, present_mode: wgpu::PresentMode::Fifo, alpha_mode: caps.alpha_modes[0], view_formats: vec![], desired_maximum_frame_latency: 2 };
        surface.configure(&device, &config);
        
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        
        let blend_state = wgpu::BlendState {
            color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add },
            alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add },
        };

        // 1. Compile Vector Pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Vector Pipeline Layout"), bind_group_layouts: &[], immediate_size: 0 });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Vector Pipeline"), layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[math::Vertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        // 2. Compile Raster Pipeline
        let raster_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2, sample_type: wgpu::TextureSampleType::Float { filterable: true } }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
            label: Some("raster_bind_group_layout"),
        });

        let raster_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Raster Pipeline Layout"), bind_group_layouts: &[&raster_bind_group_layout], immediate_size: 0 });
        let raster_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raster Pipeline"), layout: Some(&raster_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_raster"), buffers: &[math::RasterVertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_raster"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        let quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Quad VB"), contents: bytemuck::cast_slice(math::FULLSCREEN_QUAD_VERTS), usage: wgpu::BufferUsages::VERTEX });
        let quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Quad IB"), contents: bytemuck::cast_slice(math::FULLSCREEN_QUAD_INDS), usage: wgpu::BufferUsages::INDEX });
        
        self.device = Some(device); self.queue = Some(queue); self.surface = Some(surface); self.config = Some(config); 
        self.render_pipeline = Some(render_pipeline);
        self.raster_pipeline = Some(raster_pipeline);
        self.raster_bind_group_layout = Some(raster_bind_group_layout);
        self.quad_vb = Some(quad_vb);
        self.quad_ib = Some(quad_ib);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn resize_surface(&mut self, physical_width: u32, physical_height: u32) {
        let safe_width = physical_width.max(1);
        let safe_height = physical_height.max(1);
        self.canvas_width = safe_width as f32;
        self.canvas_height = safe_height as f32;

        if let (Some(device), Some(surface), Some(config)) = (&self.device, &self.surface, &mut self.config) {
            config.width = safe_width; config.height = safe_height;
            surface.configure(device, config);
        }
    }

    #[wasm_bindgen]
    pub fn begin_stroke(&mut self, x: f32, y: f32, pressure: f32) -> Result<(), JsValue> {
        self.active_stroke = Some(Stroke::new());
        self.push_point(x, y, pressure)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn push_point(&mut self, x: f32, y: f32, pressure: f32) -> Result<(), JsValue> {
        if let Some(stroke) = &mut self.active_stroke {
            stroke.add_point(x, y, pressure);
            stroke.build_mesh(self.canvas_width, self.canvas_height);
            Ok(())
        } else { Err(EngineError::InputWithoutActiveStroke.into()) }
    }

    #[wasm_bindgen]
    pub fn end_stroke(&mut self) -> Result<(), JsValue> {
        if let Some(mut stroke) = self.active_stroke.take() {
            stroke.build_mesh(self.canvas_width, self.canvas_height);
            let target_node_id = self.graph.active_layer_node.expect("Fatal Engine Error: Missing active layer.");
            let stroke_id = self.history.generate_stroke_id();
            let command = Box::new(AddStrokeCommand { target_node_id, stroke_id, stroke });
            self.history.push_and_execute(command, &mut self.graph);
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&mut self) {
        if let (Some(device), Some(queue), Some(surface), Some(bind_group_layout)) = (&self.device, &self.queue, &self.surface, &self.raster_bind_group_layout) {
            
            // --- RESOURCE MANAGER SYNC ---
            for (node_id, node) in &mut self.graph.nodes {
                if let graph::NodeType::RasterLayer { width, height, pixels, is_dirty } = &mut node.payload {
                    if *is_dirty && *width > 0 && *height > 0 {
                        let texture_size = wgpu::Extent3d { width: *width, height: *height, depth_or_array_layers: 1 };
                        
                        let recreate = match self.raster_cache.get(node_id) {
                            Some((tex, _)) => tex.width() != *width || tex.height() != *height,
                            None => true,
                        };

                        if recreate {
                            let texture = device.create_texture(&wgpu::TextureDescriptor {
                                size: texture_size, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8UnormSrgb, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                                label: Some(&format!("Raster Node {}", node_id)), view_formats: &[],
                            });

                            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                            
                            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                                address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, address_mode_w: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, mipmap_filter: wgpu::MipmapFilterMode::Nearest, ..Default::default()
                            });

                            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: bind_group_layout, entries: &[
                                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                                ], label: Some("Raster Bind Group"),
                            });

                            self.raster_cache.insert(*node_id, (texture, bind_group));
                        }

                        if let Some((texture, _)) = self.raster_cache.get(node_id) {
                            queue.write_texture(
                                wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                                pixels,
                                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * *width), rows_per_image: Some(*height) },
                                texture_size,
                            );
                        }
                        *is_dirty = false;
                    }
                }
            }

            // --- THE COMPOSITOR RENDER PASS ---
            let output = match surface.get_current_texture() { 
                Ok(frame) => frame, Err(wgpu::SurfaceError::Outdated) => return, Err(e) => { error!("Surface error: {:?}", e); return; }
            };
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            
            {
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.09, b: 0.10, a: 1.0 }), store: wgpu::StoreOp::Store }, depth_slice: None,
                    })], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, ..Default::default()
                });
                
                // 1. Draw Raster Layers
                if let (Some(raster_pipe), Some(quad_vb), Some(quad_ib)) = (&self.raster_pipeline, &self.quad_vb, &self.quad_ib) {
                    rp.set_pipeline(raster_pipe);
                    rp.set_vertex_buffer(0, quad_vb.slice(..));
                    rp.set_index_buffer(quad_ib.slice(..), wgpu::IndexFormat::Uint16);
                    
                    for (node_id, node) in &self.graph.nodes {
                        if let graph::NodeType::RasterLayer { .. } = &node.payload {
                            if let Some((_, bind_group)) = self.raster_cache.get(node_id) {
                                rp.set_bind_group(0, bind_group, &[]);
                                rp.draw_indexed(0..6, 0, 0..1);
                            }
                        }
                    }
                }

                // 2. Draw Vector Layers
                if let Some(vector_pipe) = &self.render_pipeline {
                    rp.set_pipeline(vector_pipe);
                    let mut draw = |stroke: &Stroke| {
                        if stroke.vertices.is_empty() { return; }
                        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("VB"), contents: bytemuck::cast_slice(&stroke.vertices), usage: wgpu::BufferUsages::VERTEX });
                        let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("IB"), contents: bytemuck::cast_slice(&stroke.indices), usage: wgpu::BufferUsages::INDEX });
                        rp.set_vertex_buffer(0, vb.slice(..)); rp.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16); rp.draw_indexed(0..stroke.indices.len() as u32, 0, 0..1);
                    };
                    
                    let render_list = self.graph.collect_renderable_strokes();
                    for s in render_list { draw(s); }
                    if let Some(s) = &self.active_stroke { draw(s); }
                }
            }
            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    }
}