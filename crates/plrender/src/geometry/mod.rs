use crate::renderer::{resources::mesh, Renderer};
use crate::MeshBuilder;

pub mod cuboid;
pub mod plane;
#[cfg(feature = "shape")]
pub mod shape;
pub mod sphere;
pub mod vertex;

pub use vertex::*;

bitflags::bitflags!(
    /// Optional vertex types.
    pub struct VertexTypes: u32 {
        const NORMAL = 1 << 1;
    }
);

pub struct Geometry {
    pub positions: Vec<vertex::Position>,
    pub normals: Option<Vec<vertex::Normal>>,
    pub indices: Option<Vec<u16>>,
    pub radius: f32,
}

impl Geometry {
    pub fn build_mesh(&self, renderer: &mut Renderer) -> mesh::MeshPrototype {
        let mut mesh_builder = MeshBuilder::new(renderer);

        mesh_builder.radius(self.radius);
        mesh_builder.vertex(&self.positions);
        if let Some(ref normals) = self.normals {
            mesh_builder.vertex(normals);
        }
        if let Some(ref indices) = self.indices {
            mesh_builder.index(indices);
        }
        mesh_builder.build()
    }
}
