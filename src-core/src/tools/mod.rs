pub mod brush;
pub mod pencil;
pub mod eraser;
pub mod cutter;
pub mod select;

use crate::settings::EngineSettings;
use crate::graph::{SceneManager, IdAllocator};
use crate::command::Command;
use crate::math::Vertex;

pub trait CanvasTool: Send + Sync {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool, settings: EngineSettings, scene: &mut SceneManager, id_allocator: &mut IdAllocator);
    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, constrain: bool, center: bool, scene: &mut SceneManager, canvas_width: f32, canvas_height: f32);
    fn on_pointer_up(&mut self, id_allocator: &mut IdAllocator, canvas_width: f32, canvas_height: f32, scene: &mut SceneManager) -> Option<Box<dyn Command>>;
    
    fn on_pointer_hover(&mut self, _x: f32, _y: f32, _constrain: bool, _center: bool, _scene: &SceneManager) {}
    
    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>);
    fn get_cursor(&self) -> &'static str { "crosshair" }
    fn get_custom_pivot(&self) -> Option<(f32, f32)> { None }
}