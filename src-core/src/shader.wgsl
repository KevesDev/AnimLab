// --- VERTEX SHADER ---
// This reads the raw memory byte-arrays we created in Rust.
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// The Vertex Shader executes once for every single corner of every triangle.
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Our Rust tessellator already did the heavy math to convert to WebGPU's Clip Space!
    // We just pass the X and Y coordinates in, and hardcode Z (0.0) and W (1.0).
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;
    return out;
}

// --- FRAGMENT SHADER ---
// The Fragment Shader executes once for every single PIXEL inside the triangle.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simply output the color we defined in Rust.
    return in.color;
}