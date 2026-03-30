use std::sync::RwLock;
use lazy_static::lazy_static;

/// Represents all user-adjustable preferences within the engine.
/// We strictly use #[repr(C)] to guarantee memory alignment so this entire struct 
/// can be safely copied directly into WebGPU Uniform Buffers without corruption.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EngineSettings {
    pub brush_thickness: f32,
    pub brush_color: [f32; 4],
    pub smoothing_level: f32, // 0.0 = Raw Hardware, 1.0 = Max Bezier Interpolation
}

// We use lazy_static to create a Global Variable that lives as long as the WASM module.
// RwLock (Read-Write Lock) guarantees thread-safe interior mutability, meaning the 
// WebGPU render loop can read this safely 60 times a second, and the React UI can 
// safely write to it without causing memory panics.
lazy_static! {
    pub static ref SETTINGS: RwLock<EngineSettings> = RwLock::new(EngineSettings {
        brush_thickness: 12.0,
        brush_color: [0.9, 0.9, 0.9, 1.0],
        smoothing_level: 0.5,
    });
}

/// Global API for the Engine's math and render pipelines to read current settings.
pub fn get_settings() -> EngineSettings {
    *SETTINGS.read().unwrap()
}

/// Global API for the React UI bridge to inject new settings.
pub fn update_settings(new_settings: EngineSettings) {
    let mut settings = SETTINGS.write().unwrap();
    *settings = new_settings;
}