use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt; 
use std::collections::HashMap;

use crate::math;
use crate::graph::{SceneManager, ElementId};
use crate::tools::CanvasTool;

pub struct WebGpuRenderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    
    pub standard_pipeline: wgpu::RenderPipeline,
    pub stencil_write_pipeline: wgpu::RenderPipeline, 
    pub stencil_read_0_pipeline: wgpu::RenderPipeline, 
    pub stencil_read_1_pipeline: wgpu::RenderPipeline, 
    pub depth_stencil_texture: wgpu::TextureView,
    
    pub raster_pipeline: wgpu::RenderPipeline,
    pub raster_bind_group_layout: wgpu::BindGroupLayout,
    pub raster_cache: HashMap<ElementId, (wgpu::Texture, wgpu::BindGroup)>,
    pub quad_vb: wgpu::Buffer,
    pub quad_ib: wgpu::Buffer,
}

impl WebGpuRenderer {
    pub async fn new(canvas: HtmlCanvasElement, physical_width: u32, physical_height: u32) -> Result<Self, String> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas)).map_err(|_| String::from("Failed to create WebGPU surface"))?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, compatible_surface: Some(&surface), force_fallback_adapter: false }).await.map_err(|_| String::from("Failed to request WebGPU adapter"))?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { label: Some("AnimLab GPU Device"), required_features: wgpu::Features::empty(), required_limits: adapter.limits(), ..Default::default() }).await.map_err(|_| String::from("Failed to request WebGPU device"))?;
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

        let stencil_write_face = wgpu::StencilFaceState { compare: wgpu::CompareFunction::Always, fail_op: wgpu::StencilOperation::Replace, depth_fail_op: wgpu::StencilOperation::Replace, pass_op: wgpu::StencilOperation::Replace };
        let stencil_write_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stencil Write Pipeline"), layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[math::Vertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state_normal), write_mask: wgpu::ColorWrites::empty() })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth24PlusStencil8, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: wgpu::StencilState { front: stencil_write_face.clone(), back: stencil_write_face, read_mask: !0, write_mask: !0 }, bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        let stencil_read_0_face = wgpu::StencilFaceState { compare: wgpu::CompareFunction::Equal, fail_op: wgpu::StencilOperation::Keep, depth_fail_op: wgpu::StencilOperation::Keep, pass_op: wgpu::StencilOperation::Keep };
        let stencil_read_0_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stencil Read 0 Pipeline"), layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[math::Vertex::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend_state_normal), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth24PlusStencil8, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: wgpu::StencilState { front: stencil_read_0_face.clone(), back: stencil_read_0_face, read_mask: !0, write_mask: 0 }, bias: wgpu::DepthBiasState::default() }),
            multisample: wgpu::MultisampleState::default(), multiview_mask: None, cache: None,
        });

        let stencil_read_1_face = wgpu::StencilFaceState { compare: wgpu::CompareFunction::Equal, fail_op: wgpu::StencilOperation::Keep, depth_fail_op: wgpu::StencilOperation::Keep, pass_op: wgpu::StencilOperation::Keep };
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
        
        Ok(Self { device, queue, surface, config, standard_pipeline, stencil_write_pipeline, stencil_read_0_pipeline, stencil_read_1_pipeline, depth_stencil_texture, raster_pipeline, raster_bind_group_layout, raster_cache: HashMap::new(), quad_vb, quad_ib, })
    }

    pub fn resize(&mut self, physical_width: u32, physical_height: u32) {
        let safe_width = physical_width.max(1); let safe_height = physical_height.max(1);
        self.config.width = safe_width; self.config.height = safe_height; 
        self.surface.configure(&self.device, &self.config);
        self.depth_stencil_texture = self.device.create_texture(&wgpu::TextureDescriptor { size: wgpu::Extent3d { width: safe_width, height: safe_height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Depth24PlusStencil8, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, label: Some("Depth Stencil Texture"), view_formats: &[] }).create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn render(&mut self, scene: &SceneManager, active_tool: &dyn CanvasTool, canvas_width: f32, canvas_height: f32) {
        let output = match self.surface.get_current_texture() { Ok(frame) => frame, Err(_) => return };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"), 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.09, b: 0.10, a: 1.0 }), store: wgpu::StoreOp::Store }, depth_slice: None })], 
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &self.depth_stencil_texture, depth_ops: None, stencil_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0), store: wgpu::StoreOp::Store }) }), 
                timestamp_writes: None, occlusion_query_set: None, ..Default::default()
            });
            
            let draw = |rp_ref: &mut wgpu::RenderPass<'_>, vertices: &[math::Vertex], indices: &[u16]| {
                if vertices.is_empty() { return; }
                let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("VB"), contents: bytemuck::cast_slice(vertices), usage: wgpu::BufferUsages::VERTEX });
                let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("IB"), contents: bytemuck::cast_slice(indices), usage: wgpu::BufferUsages::INDEX });
                rp_ref.set_vertex_buffer(0, vb.slice(..)); rp_ref.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16); rp_ref.draw_indexed(0..indices.len() as u32, 0, 0..1);
            };

            let render_list = scene.collect_renderable_elements();
            for element in render_list { 
                use crate::geometry::VectorElement;
                match element {
                    VectorElement::Centerline(c) => { rp.set_pipeline(&self.standard_pipeline); draw(&mut rp, &c.vertices, &c.indices); }
                    VectorElement::Contour(c) => {
                        if c.clip_masks.is_empty() && c.eraser_masks.is_empty() {
                            rp.set_pipeline(&self.standard_pipeline); draw(&mut rp, &c.vertices, &c.indices);
                        } else if c.clip_masks.is_empty() {
                            rp.set_pipeline(&self.stencil_write_pipeline); rp.set_stencil_reference(1);
                            for mask in &c.eraser_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                            rp.set_pipeline(&self.stencil_read_0_pipeline); rp.set_stencil_reference(0);
                            draw(&mut rp, &c.vertices, &c.indices);
                            rp.set_pipeline(&self.stencil_write_pipeline); rp.set_stencil_reference(0);
                            for mask in &c.eraser_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                        } else {
                            rp.set_pipeline(&self.stencil_write_pipeline); rp.set_stencil_reference(1);
                            for mask in &c.clip_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                            rp.set_stencil_reference(0);
                            for mask in &c.eraser_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                            rp.set_pipeline(&self.stencil_read_1_pipeline); rp.set_stencil_reference(1);
                            draw(&mut rp, &c.vertices, &c.indices);
                            rp.set_pipeline(&self.stencil_write_pipeline); rp.set_stencil_reference(0);
                            for mask in &c.clip_masks { draw(&mut rp, &mask.vertices, &mask.indices); }
                        }
                    }
                }
            }
            
            if let Some(aabb) = scene.get_selection_aabb() {
                let mut bb_verts = Vec::new(); let mut bb_inds = Vec::new(); let tb_orange = [1.0, 0.45, 0.0, 1.0];
                let pts = vec![ crate::geometry::Point { x: aabb.min_x, y: aabb.min_y, pressure: 1.0 }, crate::geometry::Point { x: aabb.max_x, y: aabb.min_y, pressure: 1.0 }, crate::geometry::Point { x: aabb.max_x, y: aabb.max_y, pressure: 1.0 }, crate::geometry::Point { x: aabb.min_x, y: aabb.max_y, pressure: 1.0 }, crate::geometry::Point { x: aabb.min_x, y: aabb.min_y, pressure: 1.0 } ];
                let (box_v, box_i, _) = crate::geometry::tessellator::Extruder::extrude_centerline(&pts, 1.0, tb_orange, canvas_width, canvas_height);
                let offset = bb_verts.len() as u16; bb_verts.extend(box_v); for idx in box_i { bb_inds.push(idx + offset); }

                let mut add_quad = |x: f32, y: f32, w: f32, h: f32, color: [f32; 4]| {
                    let left = (x / canvas_width) * 2.0 - 1.0; let right = ((x + w) / canvas_width) * 2.0 - 1.0; let top = 1.0 - (y / canvas_height) * 2.0; let bottom = 1.0 - ((y + h) / canvas_height) * 2.0; let start_idx = bb_verts.len() as u16;
                    bb_verts.push(math::Vertex { position: [left, top], color, tex_coords: [0.0, 0.0] }); bb_verts.push(math::Vertex { position: [right, top], color, tex_coords: [1.0, 0.0] }); bb_verts.push(math::Vertex { position: [right, bottom], color, tex_coords: [1.0, 1.0] }); bb_verts.push(math::Vertex { position: [left, bottom], color, tex_coords: [0.0, 1.0] }); bb_inds.extend_from_slice(&[start_idx, start_idx + 1, start_idx + 2, start_idx, start_idx + 2, start_idx + 3]);
                };

                let hs = 3.0; let b_hs = 4.0; 
                let coords = [ (aabb.min_x, aabb.min_y), (aabb.max_x, aabb.min_y), (aabb.max_x, aabb.max_y), (aabb.min_x, aabb.max_y), ((aabb.min_x + aabb.max_x) / 2.0, aabb.min_y), ((aabb.min_x + aabb.max_x) / 2.0, aabb.max_y), (aabb.min_x, (aabb.min_y + aabb.max_y) / 2.0), (aabb.max_x, (aabb.min_y + aabb.max_y) / 2.0) ];
                for (hx, hy) in coords { add_quad(hx - b_hs, hy - b_hs, b_hs * 2.0, b_hs * 2.0, tb_orange); add_quad(hx - hs, hy - hs, hs * 2.0, hs * 2.0, [1.0, 1.0, 1.0, 1.0]); }

                let (px, py) = active_tool.get_custom_pivot().unwrap_or((aabb.min_x + (aabb.max_x - aabb.min_x) / 2.0, aabb.min_y + (aabb.max_y - aabb.min_y) / 2.0));
                add_quad(px - 5.0, py - 5.0, 10.0, 10.0, [0.0, 0.6, 1.0, 1.0]); add_quad(px - 2.0, py - 2.0, 4.0, 4.0, [1.0, 1.0, 1.0, 1.0]);   

                rp.set_pipeline(&self.standard_pipeline); draw(&mut rp, &bb_verts, &bb_inds);
            }

            let preview_mesh_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { active_tool.get_preview_mesh(canvas_width, canvas_height) }));
            let (preview_verts, preview_inds) = match preview_mesh_result { Ok(mesh) => mesh, Err(_) => { (Vec::new(), Vec::new()) } };
            rp.set_pipeline(&self.standard_pipeline); draw(&mut rp, &preview_verts, &preview_inds);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}