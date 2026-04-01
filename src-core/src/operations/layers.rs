use crate::graph::{SceneManager, ArtLayerType};

pub fn set_active_art_layer(scene: &mut SceneManager, layer_index: u8) {
    scene.active_art_layer = match layer_index {
        0 => ArtLayerType::Overlay,
        1 => ArtLayerType::LineArt,
        2 => ArtLayerType::ColorArt,
        3 => ArtLayerType::Underlay,
        _ => ArtLayerType::LineArt,
    };
}

pub fn set_opacity(scene: &mut SceneManager, element_id: u64, opacity: f32) {
    if let Some(el) = scene.elements.get_mut(&element_id) {
        el.opacity = opacity.clamp(0.0, 1.0);
    }
}

pub fn set_visibility(scene: &mut SceneManager, element_id: u64, is_visible: bool) {
    if let Some(el) = scene.elements.get_mut(&element_id) {
        el.is_visible = is_visible;
    }
}