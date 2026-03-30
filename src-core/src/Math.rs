use crate::stroke::Point;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress, shader_location: 1, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}

// AAA ARCHITECTURE: The Bounding Box
// A lightweight mathematical envelope used for hyper-fast spatial sorting.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl AABB {
    /// Creates an infinitely inverted box, ready to swallow coordinates and expand.
    pub fn empty() -> Self {
        Self { min_x: f32::MAX, min_y: f32::MAX, max_x: f32::MIN, max_y: f32::MIN }
    }

    /// Expands the box to engulf a given point and its physical thickness radius.
    pub fn expand_to_include(&mut self, x: f32, y: f32, radius: f32) {
        self.min_x = self.min_x.min(x - radius);
        self.min_y = self.min_y.min(y - radius);
        self.max_x = self.max_x.max(x + radius);
        self.max_y = self.max_y.max(y + radius);
    }

    /// Checks if two bounding boxes overlap in 2D space.
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min_x <= other.max_x && self.max_x >= other.min_x &&
        self.min_y <= other.max_y && self.max_y >= other.min_y
    }
}

pub struct Tessellator;

impl Tessellator {
    pub fn extrude_stroke(points: &[Point], base_thickness: f32, color: [f32; 4], canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::with_capacity(points.len() * 2);
        let mut indices = Vec::with_capacity(points.len() * 6);

        if points.len() < 2 { return (vertices, indices); }

        for i in 0..points.len() {
            let current = &points[i];
            
            let (dx, dy) = if i < points.len() - 1 {
                let next = &points[i + 1]; (next.x - current.x, next.y - current.y)
            } else {
                let prev = &points[i - 1]; (current.x - prev.x, current.y - prev.y)
            };

            let length = (dx * dx + dy * dy).sqrt();
            let (nx, ny) = if length > 0.0001 { (-dy / length, dx / length) } else { (0.0, 1.0) };

            let radius = (base_thickness * current.pressure) / 2.0;
            let top_x = current.x + nx * radius; let top_y = current.y + ny * radius;
            let bot_x = current.x - nx * radius; let bot_y = current.y - ny * radius;

            let clip_top_x = (top_x / canvas_width) * 2.0 - 1.0;
            let clip_top_y = 1.0 - (top_y / canvas_height) * 2.0; 
            let clip_bot_x = (bot_x / canvas_width) * 2.0 - 1.0;
            let clip_bot_y = 1.0 - (bot_y / canvas_height) * 2.0; 

            vertices.push(Vertex { position: [clip_top_x, clip_top_y], color });
            vertices.push(Vertex { position: [clip_bot_x, clip_bot_y], color });

            if i < points.len() - 1 {
                let base_idx = (i * 2) as u16;
                indices.push(base_idx); indices.push(base_idx + 1); indices.push(base_idx + 2);
                indices.push(base_idx + 1); indices.push(base_idx + 3); indices.push(base_idx + 2);
            }
        }
        (vertices, indices)
    }
}

pub fn smooth_points(points: &[Point], smoothing_level: f32) -> Vec<Point> {
    if points.len() < 3 || smoothing_level <= 0.0 { return points.to_vec(); }
    let steps = (smoothing_level * 10.0).max(1.0) as usize;
    let mut smoothed = Vec::with_capacity(points.len() * steps);
    smoothed.push(points[0]);

    for i in 0..(points.len() - 1) {
        let p0 = if i == 0 { &points[0] } else { &points[i - 1] };
        let p1 = &points[i]; let p2 = &points[i + 1];
        let p3 = if i + 2 < points.len() { &points[i + 2] } else { &points[i + 1] };

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let t2 = t * t; let t3 = t2 * t;

            let interpolate = |v0: f32, v1: f32, v2: f32, v3: f32| -> f32 {
                0.5 * ((2.0 * v1) + (-v0 + v2) * t + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2 + (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3)
            };

            smoothed.push(Point {
                x: interpolate(p0.x, p1.x, p2.x, p3.x),
                y: interpolate(p0.y, p1.y, p2.y, p3.y),
                pressure: interpolate(p0.pressure, p1.pressure, p2.pressure, p3.pressure),
            });
        }
    }
    smoothed
}