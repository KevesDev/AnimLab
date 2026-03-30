use super::Point;
use crate::math::{Vertex, AABB};
use geo::{Polygon, MultiPolygon, LineString, Coord};
use lyon_tessellation::{
    VertexBuffers, FillTessellator, FillOptions, 
    math::point as lyon_point, path::Path, BuffersBuilder, FillVertex
};

pub struct Extruder;

impl Extruder {
    pub fn tessellate_multipolygon(
        multipoly: &MultiPolygon<f32>, color: [f32; 4], canvas_width: f32, canvas_height: f32
    ) -> (Vec<Vertex>, Vec<u16>, AABB) {
        let mut buffers: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        let mut options = FillOptions::default();
        options.fill_rule = lyon_tessellation::FillRule::NonZero; 

        let mut aabb = AABB::empty();

        for poly in multipoly.iter() {
            let mut builder = Path::builder();
            let ext = poly.exterior();
            if !ext.0.is_empty() {
                for pt in ext.0.iter() { aabb.expand_to_include(pt.x, pt.y, 0.0); }
                builder.begin(lyon_point(ext.0[0].x, ext.0[0].y));
                for pt in ext.0.iter().skip(1) { builder.line_to(lyon_point(pt.x, pt.y)); }
                builder.end(true);
            }
            for int in poly.interiors() {
                if int.0.is_empty() { continue; }
                builder.begin(lyon_point(int.0[0].x, int.0[0].y));
                for pt in int.0.iter().skip(1) { builder.line_to(lyon_point(pt.x, pt.y)); }
                builder.end(true);
            }

            tessellator.tessellate_path(
                &builder.build(), &options,
                &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                    let px = vertex.position().x; let py = vertex.position().y;
                    let clip_x = (px / canvas_width) * 2.0 - 1.0; let clip_y = 1.0 - (py / canvas_height) * 2.0;
                    Vertex { position: [clip_x, clip_y], color, tex_coords: [0.0, 0.0] }
                }),
            ).unwrap_or_else(|_| ());
        }
        (buffers.vertices, buffers.indices, aabb)
    }

    pub fn extrude_contour(
        points: &[Point], base_thickness: f32, color: [f32; 4], canvas_width: f32, canvas_height: f32
    ) -> (MultiPolygon<f32>, Vec<Vertex>, Vec<u16>, AABB) {
        if points.len() < 2 { return (MultiPolygon::new(vec![]), Vec::new(), Vec::new(), AABB::empty()); }

        let mut top_points = Vec::new(); let mut bot_points = Vec::new();
        for i in 0..points.len() {
            let current = &points[i];
            let (dx, dy) = if i < points.len() - 1 { let next = &points[i + 1]; (next.x - current.x, next.y - current.y) } 
                           else { let prev = &points[i - 1]; (current.x - prev.x, current.y - prev.y) };

            let length = (dx * dx + dy * dy).sqrt();
            let (nx, ny) = if length > 0.0001 { (-dy / length, dx / length) } else { (0.0, 1.0) };

            let radius = (base_thickness * current.pressure) / 2.0;
            top_points.push(Coord { x: current.x + nx * radius, y: current.y + ny * radius });
            bot_points.push(Coord { x: current.x - nx * radius, y: current.y - ny * radius });
        }

        let mut exterior_coords = top_points;
        for pt in bot_points.into_iter().rev() { exterior_coords.push(pt); }
        if let Some(first) = exterior_coords.first().cloned() { exterior_coords.push(first); }

        let poly = Polygon::new(LineString::new(exterior_coords), vec![]);
        let multipoly = MultiPolygon::new(vec![poly]);

        let (vertices, indices, aabb) = Self::tessellate_multipolygon(&multipoly, color, canvas_width, canvas_height);
        (multipoly, vertices, indices, aabb)
    }

    // AAA UPGRADE: Turns a Lasso line into a solid GPU Stencil Mask
    pub fn tessellate_lasso(points: &[Point], canvas_width: f32, canvas_height: f32) -> (Vec<Vertex>, Vec<u16>) {
        if points.len() < 3 { return (Vec::new(), Vec::new()); }
        let mut buffers: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        let mut options = FillOptions::default();
        options.fill_rule = lyon_tessellation::FillRule::NonZero;

        let mut builder = Path::builder();
        builder.begin(lyon_point(points[0].x, points[0].y));
        for pt in points.iter().skip(1) { builder.line_to(lyon_point(pt.x, pt.y)); }
        builder.end(true);

        tessellator.tessellate_path(
            &builder.build(), &options,
            &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                let px = vertex.position().x; let py = vertex.position().y;
                let clip_x = (px / canvas_width) * 2.0 - 1.0; let clip_y = 1.0 - (py / canvas_height) * 2.0;
                Vertex { position: [clip_x, clip_y], color: [1.0; 4], tex_coords: [0.0, 0.0] }
            }),
        ).unwrap_or_else(|_| ());

        (buffers.vertices, buffers.indices)
    }
    
    pub fn extrude_centerline(
        points: &[Point], thickness: f32, color: [f32; 4], canvas_width: f32, canvas_height: f32
    ) -> (Vec<Vertex>, Vec<u16>, AABB) {
        let mut vertices = Vec::with_capacity(points.len() * 2);
        let mut indices = Vec::with_capacity(points.len() * 6);
        let mut aabb = AABB::empty();

        if points.len() < 2 { return (vertices, indices, aabb); }
        let max_radius = thickness / 2.0;

        for i in 0..points.len() {
            let current = &points[i];
            aabb.expand_to_include(current.x, current.y, max_radius);
            
            let (dx, dy) = if i < points.len() - 1 { let next = &points[i + 1]; (next.x - current.x, next.y - current.y) } 
                           else { let prev = &points[i - 1]; (current.x - prev.x, current.y - prev.y) };

            let length = (dx * dx + dy * dy).sqrt();
            let (nx, ny) = if length > 0.0001 { (-dy / length, dx / length) } else { (0.0, 1.0) };

            let radius = (thickness * current.pressure) / 2.0;
            let top_x = current.x + nx * radius; let top_y = current.y + ny * radius;
            let bot_x = current.x - nx * radius; let bot_y = current.y - ny * radius;

            let clip_top_x = (top_x / canvas_width) * 2.0 - 1.0; let clip_top_y = 1.0 - (top_y / canvas_height) * 2.0; 
            let clip_bot_x = (bot_x / canvas_width) * 2.0 - 1.0; let clip_bot_y = 1.0 - (bot_y / canvas_height) * 2.0; 

            vertices.push(Vertex { position: [clip_top_x, clip_top_y], color, tex_coords: [0.0, 0.0] });
            vertices.push(Vertex { position: [clip_bot_x, clip_bot_y], color, tex_coords: [0.0, 1.0] });

            if i < points.len() - 1 {
                let base_idx = (i * 2) as u16;
                indices.push(base_idx); indices.push(base_idx + 1); indices.push(base_idx + 2);
                indices.push(base_idx + 1); indices.push(base_idx + 3); indices.push(base_idx + 2);
            }
        }
        (vertices, indices, aabb)
    }
}