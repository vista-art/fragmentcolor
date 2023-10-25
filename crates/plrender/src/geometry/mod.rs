use crate::renderer;
use crate::MeshBuilder;

pub mod cuboid;
pub mod plane;
#[cfg(feature = "shape")]
pub mod shape;
pub mod sphere;
pub mod vertex;

pub use vertex::*;

bitflags::bitflags!(
    /// Types of optional vertex streams.
    pub struct Streams: u32 {
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
    pub fn bake(&self, renderer: &mut renderer::Renderer) -> renderer::resources::mesh::Prototype {
        // Provisory until we refactor the Mesh API to remove the builder
        // and implement a regular new() constructor method.
        let mut mb = MeshBuilder::new(renderer);

        mb.radius(self.radius);
        mb.vertex(&self.positions);
        if let Some(ref stream) = self.normals {
            mb.vertex(stream);
        }
        if let Some(ref indices) = self.indices {
            mb.index(indices);
        }
        mb.build()
    }
}
