pub struct EngineSettings {
    pub brush_thickness: f32,
    pub brush_color: [f32; 4],
    pub smoothing_level: f32,
}

static mut SETTINGS: EngineSettings = EngineSettings {
    brush_thickness: 4.0,
    // AAA FIX: Default to dark charcoal ink so it contrasts against the 0.9 white paper canvas
    brush_color: [0.08, 0.08, 0.08, 1.0], 
    smoothing_level: 0.5,
};

pub fn get_settings() -> EngineSettings {
    unsafe { EngineSettings { brush_thickness: SETTINGS.brush_thickness, brush_color: SETTINGS.brush_color, smoothing_level: SETTINGS.smoothing_level } }
}

pub fn update_settings(new_settings: EngineSettings) {
    unsafe { SETTINGS = new_settings; }
}