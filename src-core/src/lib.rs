// --- IMPORTS ---
use wasm_bindgen::prelude::*;
use log::{info, error, Level};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt; 

pub mod math;
pub mod stroke;
pub mod settings; 
pub mod graph; // Initialize our new DAG architecture

use stroke::Stroke;
use graph::AnimGraph;

// --- ERROR HANDLING ARCHITECTURE ---
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

// --- INITIALIZATION PIPELINE ---
#[wasm_bindgen(start)]
pub fn init_core() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).map_err(|e| EngineError::LoggerInitFailed(e.to_string()))?;
    info!("AnimLab Core System: WASM Binary loaded.");
    Ok(())
}

// --- THE MAIN ENGINE CONTROLLER ---
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
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    
    #[wasm_bindgen(skip)]
    pub active_stroke: Option<Stroke>,
    
    // AAA FIX: Replaced flat array with the Directed Acyclic Graph (DAG)
    #[wasm_bindgen(skip)]
    pub graph: AnimGraph,
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
            render_pipeline: None,
            active_stroke: None,
            graph: AnimGraph::new(), // Boot up the Node Engine
        })
    }

    #[wasm_bindgen]
    pub fn get_system_status(&self) -> String {
        if self.is_ready { String::from("AnimLab Rust Core: Online & Memory Safe.") } 
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
    pub async fn attach_canvas(&mut self, canvas: HtmlCanvasElement, width: u32, height: u32) -> Result<(), JsValue> {
        self.canvas_width = width as f32;
        self.canvas_height = height as f32;
        
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas)).map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance, 
            compatible_surface: Some(&surface), 
            force_fallback_adapter: false,
        }).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("AnimLab GPU Device"), 
            required_features: wgpu::Features::empty(), 
            required_limits: adapter.limits(), 
            ..Default::default()
        }).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]); 
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, 
            format, 
            width, 
            height,
            present_mode: wgpu::PresentMode::Fifo, 
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![], 
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"), 
            bind_group_layouts: &[], 
            immediate_size: 0,
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"), 
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point: Some("vs_main"), 
                buffers: &[math::Vertex::desc()], 
                compilation_options: Default::default() 
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader, 
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { 
                    format: config.format, 
                    blend: Some(wgpu::BlendState::REPLACE), 
                    write_mask: wgpu::ColorWrites::ALL 
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None, 
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: None, 
                unclipped_depth: false, 
                polygon_mode: wgpu::PolygonMode::Fill, 
                conservative: false 
            },
            depth_stencil: None, 
            multisample: wgpu::MultisampleState::default(), 
            multiview_mask: None, 
            cache: None,
        });
        
        self.device = Some(device); 
        self.queue = Some(queue); 
        self.surface = Some(surface); 
        self.render_pipeline = Some(render_pipeline);
        Ok(())
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
        } else {
            Err(EngineError::InputWithoutActiveStroke.into())
        }
    }

    #[wasm_bindgen]
    pub fn end_stroke(&mut self) -> Result<(), JsValue> {
        if let Some(mut stroke) = self.active_stroke.take() {
            stroke.build_mesh(self.canvas_width, self.canvas_height);
            // AAA FIX: Push the stroke safely into the Node Graph hierarchy
            self.graph.inject_stroke(stroke);
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&self) {
        if let (Some(device), Some(queue), Some(surface), Some(pipeline)) = (&self.device, &self.queue, &self.surface, &self.render_pipeline) {
            
            let output = match surface.get_current_texture() { 
                Ok(frame) => frame, 
                Err(wgpu::SurfaceError::Outdated) => return,
                Err(e) => { error!("Surface error: {:?}", e); return; }
            };
            
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            
            {
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view, 
                        resolve_target: None,
                        ops: wgpu::Operations { 
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.09, b: 0.10, a: 1.0 }), 
                            store: wgpu::StoreOp::Store 
                        },
                        depth_slice: None,
                    })], 
                    depth_stencil_attachment: None, 
                    timestamp_writes: None, 
                    occlusion_query_set: None,
                    ..Default::default()
                });
                
                rp.set_pipeline(pipeline);
                
                let mut draw = |stroke: &Stroke| {
                    if stroke.vertices.is_empty() { return; }
                    let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
                        label: Some("Vertex Buffer"), 
                        contents: bytemuck::cast_slice(&stroke.vertices), 
                        usage: wgpu::BufferUsages::VERTEX 
                    });
                    let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
                        label: Some("Index Buffer"), 
                        contents: bytemuck::cast_slice(&stroke.indices), 
                        usage: wgpu::BufferUsages::INDEX 
                    });
                    
                    rp.set_vertex_buffer(0, vb.slice(..));
                    rp.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
                    rp.draw_indexed(0..stroke.indices.len() as u32, 0, 0..1);
                };
                
                // Ask the Graph to compile the render sequence
                let render_list = self.graph.collect_renderable_strokes();
                for s in render_list { draw(s); }
                
                if let Some(s) = &self.active_stroke { draw(s); }
            }
            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    }
}