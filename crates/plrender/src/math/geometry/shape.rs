use crate::math::geometry::{primitives::Primitive, vertex};
use lyon::lyon_tessellation::*;

type PositionBuilder = VertexBuffers<vertex::Position, u16>;

pub struct ShapeBuilder;

fn fill_position(vertex: lyon::tessellation::FillVertex) -> vertex::Position {
    let p = vertex.position();
    vertex::Position([p.x, p.y, 0.0])
}

fn stroke_position(vertex: lyon::tessellation::StrokeVertex) -> vertex::Position {
    let p = vertex.position();
    vertex::Position([p.x, p.y, 0.0])
}

fn bounding_radius(path: &lyon::path::Path) -> f32 {
    path.iter().fold(0.0, |accum, item| {
        let p = item.from();
        accum.max(p.x.abs().max(p.y.abs()))
    })
}

impl ShapeBuilder {
    pub fn fill(path: &lyon::path::Path) -> Primitive {
        let mut buffer = PositionBuilder::new();
        let builder = &mut BuffersBuilder::new(&mut buffer, fill_position);
        let mut tessellator = FillTessellator::new();
        tessellator
            .tessellate_path(path, &FillOptions::default(), builder)
            .unwrap();

        let radius = bounding_radius(path);

        Primitive {
            positions: buffer.vertices,
            indices: Some(buffer.indices),
            normals: None,
            radius,
        }
    }

    pub fn stroke(path: &lyon::path::Path, options: &StrokeOptions) -> Primitive {
        let mut buffer = PositionBuilder::new();
        let builder = &mut BuffersBuilder::new(&mut buffer, stroke_position);
        let mut tessellator = StrokeTessellator::new();
        tessellator.tessellate_path(path, options, builder).unwrap();

        let radius = bounding_radius(path);

        Primitive {
            positions: buffer.vertices,
            indices: Some(buffer.indices),
            normals: None,
            radius,
        }
    }
}
