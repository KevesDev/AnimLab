use crate::stroke::Point;

/// Represents a single corner of a triangle in GPU memory.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    /// Describes the exact byte-layout of this struct so the GPU Shader knows how to read it.
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Tessellator;

impl Tessellator {
    /// Extrudes a sequence of points into a continuous triangle ribbon.
    pub fn extrude_stroke(points: &[Point], base_thickness: f32, color: [f32; 4], canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::with_capacity(points.len() * 2);
        let mut indices = Vec::with_capacity(points.len() * 6);

        if points.len() < 2 {
            return (vertices, indices);
        }

        for i in 0..points.len() {
            let current = &points[i];
            
            let (dx, dy) = if i < points.len() - 1 {
                let next = &points[i + 1];
                (next.x - current.x, next.y - current.y)
            } else {
                let prev = &points[i - 1];
                (current.x - prev.x, current.y - prev.y)
            };

            let length = (dx * dx + dy * dy).sqrt();
            let (nx, ny) = if length > 0.0001 {
                (-dy / length, dx / length)
            } else {
                (0.0, 1.0)
            };

            let radius = (base_thickness * current.pressure) / 2.0;

            let top_x = current.x + nx * radius;
            let top_y = current.y + ny * radius;
            
            let bot_x = current.x - nx * radius;
            let bot_y = current.y - ny * radius;

            let clip_top_x = (top_x / canvas_width) * 2.0 - 1.0;
            let clip_top_y = 1.0 - (top_y / canvas_height) * 2.0; 
            
            let clip_bot_x = (bot_x / canvas_width) * 2.0 - 1.0;
            let clip_bot_y = 1.0 - (bot_y / canvas_height) * 2.0; 

            vertices.push(Vertex { position: [clip_top_x, clip_top_y], color });
            vertices.push(Vertex { position: [clip_bot_x, clip_bot_y], color });

            if i < points.len() - 1 {
                let base_idx = (i * 2) as u16;
                
                indices.push(base_idx);     
                indices.push(base_idx + 1); 
                indices.push(base_idx + 2);
                
                indices.push(base_idx + 1); 
                indices.push(base_idx + 3); 
                indices.push(base_idx + 2);
            }
        }

        (vertices, indices)
    }
}

/// Applies a Catmull-Rom spline interpolation to generate smooth curves between sparse hardware coordinates.
pub fn smooth_points(points: &[Point], smoothing_level: f32) -> Vec<Point> {
    if points.len() < 3 || smoothing_level <= 0.0 {
        return points.to_vec();
    }

    // Determine the number of mathematical steps to inject between each hardware capture.
    // A smoothing_level of 1.0 will calculate up to 10 additional curve points per segment.
    let steps = (smoothing_level * 10.0).max(1.0) as usize;
    let mut smoothed = Vec::with_capacity(points.len() * steps);

    smoothed.push(points[0]);

    for i in 0..(points.len() - 1) {
        // A Catmull-Rom spline requires 4 control points (p0, p1, p2, p3) to calculate the momentum of the curve.
        // We clamp the indices to the bounds of the array for the first and last segments.
        let p0 = if i == 0 { &points[0] } else { &points[i - 1] };
        let p1 = &points[i];
        let p2 = &points[i + 1];
        let p3 = if i + 2 < points.len() { &points[i + 2] } else { &points[i + 1] };

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let t2 = t * t;
            let t3 = t2 * t;

            // The core polynomial equation for Catmull-Rom curve generation
            let interpolate = |v0: f32, v1: f32, v2: f32, v3: f32| -> f32 {
                0.5 * (
                    (2.0 * v1) +
                    (-v0 + v2) * t +
                    (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2 +
                    (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3
                )
            };

            smoothed.push(Point {
                x: interpolate(p0.x, p1.x, p2.x, p3.x),
                y: interpolate(p0.y, p1.y, p2.y, p3.y),
                // We must also interpolate the pressure data to maintain a smooth stroke thickness
                pressure: interpolate(p0.pressure, p1.pressure, p2.pressure, p3.pressure),
            });
        }
    }

    smoothed
}