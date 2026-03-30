use std::sync::RwLock;

#[derive(Debug, Clone, Copy)]
pub struct EngineSettings {
    pub brush_thickness: f32,
    pub brush_color: [f32; 4],
    pub smoothing_level: f32,
}

// AAA SAFETY: Boot State Synchronization.
// These values MUST exactly match the default state of the React PreferencesStore 
// to prevent zero-thickness hardware panics before the first WASM sync.
impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            brush_thickness: 12.0,
            brush_color: [0.9, 0.9, 0.9, 1.0],
            smoothing_level: 0.5,
        }
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_SETTINGS: RwLock<EngineSettings> = RwLock::new(EngineSettings::default());
}

pub fn update_settings(new_settings: EngineSettings) {
    if let Ok(mut settings) = GLOBAL_SETTINGS.write() {
        *settings = new_settings;
    }
}

pub fn get_settings() -> EngineSettings {
    if let Ok(settings) = GLOBAL_SETTINGS.read() {
        *settings
    } else {
        EngineSettings::default()
    }
}