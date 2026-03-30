#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2], // AAA UPGRADE: UV memory allocated
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress, shader_location: 1, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RasterVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl RasterVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RasterVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            ],
        }
    }
}

pub const FULLSCREEN_QUAD_VERTS: &[RasterVertex] = &[
    RasterVertex { position: [-1.0, 1.0], tex_coords: [0.0, 0.0] }, 
    RasterVertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] }, 
    RasterVertex { position: [1.0, -1.0], tex_coords: [1.0, 1.0] }, 
    RasterVertex { position: [1.0, 1.0], tex_coords: [1.0, 0.0] }, 
];

pub const FULLSCREEN_QUAD_INDS: &[u16] = &[0, 1, 2, 0, 2, 3];

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min_x: f32, pub min_y: f32, pub max_x: f32, pub max_y: f32,
}

impl AABB {
    pub fn empty() -> Self { Self { min_x: f32::MAX, min_y: f32::MAX, max_x: f32::MIN, max_y: f32::MIN } }

    pub fn expand_to_include(&mut self, x: f32, y: f32, radius: f32) {
        self.min_x = self.min_x.min(x - radius); self.min_y = self.min_y.min(y - radius);
        self.max_x = self.max_x.max(x + radius); self.max_y = self.max_y.max(y + radius);
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min_x <= other.max_x && self.max_x >= other.min_x &&
        self.min_y <= other.max_y && self.max_y >= other.min_y
    }
}