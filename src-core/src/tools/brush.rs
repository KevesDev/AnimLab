use crate::tools::CanvasTool;
use crate::stroke::Stroke;
use crate::settings::EngineSettings;
use crate::graph::{AnimGraph, NodeId, StrokeId};
use crate::command::{Command, AddStrokeCommand};
use crate::math::{Vertex, Tessellator, smooth_points};

pub struct BrushTool {
    active_stroke: Option<Stroke>,
    settings_snapshot: Option<EngineSettings>,
}

impl BrushTool {
    pub fn new() -> Self {
        Self {
            active_stroke: None,
            settings_snapshot: None,
        }
    }
}

impl CanvasTool for BrushTool {
    fn on_pointer_down(&mut self, x: f32, y: f32, pressure: f32, settings: EngineSettings) {
        // AAA: Snapshot the UI preferences so the hot-loop doesn't query global state.
        self.settings_snapshot = Some(settings);
        let mut stroke = Stroke::new();
        stroke.add_point(x, y, pressure);
        self.active_stroke = Some(stroke);
    }

    fn on_pointer_move(&mut self, x: f32, y: f32, pressure: f32, _graph: &AnimGraph) {
        if let Some(stroke) = &mut self.active_stroke {
            stroke.add_point(x, y, pressure);
        }
    }

    fn on_pointer_up(
        &mut self, 
        active_node_id: NodeId, 
        next_stroke_id: StrokeId, 
        canvas_width: f32, 
        canvas_height: f32, 
        _graph: &AnimGraph
    ) -> Option<Box<dyn Command>> {
        if let (Some(mut stroke), Some(settings)) = (self.active_stroke.take(), self.settings_snapshot.take()) {
            
            // Finalize the mathematical footprint
            stroke.build_mesh(settings.brush_thickness, settings.brush_color, settings.smoothing_level, canvas_width, canvas_height);
            
            // Output the Delta Command for the timeline
            Some(Box::new(AddStrokeCommand {
                target_node_id: active_node_id,
                stroke_id: next_stroke_id,
                stroke,
            }))
        } else {
            None
        }
    }

    fn get_preview_mesh(&self, canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if let (Some(stroke), Some(settings)) = (&self.active_stroke, &self.settings_snapshot) {
            // Generate a non-destructive temporary mesh for the live GPU render pass
            let smoothed = smooth_points(&stroke.points, settings.smoothing_level);
            Tessellator::extrude_stroke(&smoothed, settings.brush_thickness, settings.brush_color, canvas_width, canvas_height)
        } else {
            (Vec::new(), Vec::new())
        }
    }
}