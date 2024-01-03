pub use crate::{
    math::cg::Vec3,
    math::geometry::{primitives, vertex},
    resources::mesh,
};

type Error = Box<dyn std::error::Error>;

pub(super) use primitives::{cuboid::*, plane::*, sphere::*};
pub struct Primitive {
    pub positions: Vec<vertex::Position>,
    pub normals: Option<Vec<vertex::Normal>>,
    pub indices: Option<Vec<u16>>,
    pub radius: f32,
}

impl Primitive {
    pub fn create_mesh(&self) -> Result<mesh::BuiltMesh, Error> {
        let mut mesh_builder = mesh::MeshBuilder::new();

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

    pub fn cube(size: f32) -> Self {
        primitives::cube(size)
    }

    pub fn cuboid<V: Into<Vec3>>(dimensions: V) -> Self {
        primitives::cuboid(vertex::VertexTypes::empty(), dimensions)
    }

    pub fn plane(size: f32) -> Self {
        primitives::plane(size)
    }

    pub fn sphere(radius: f32, detail: usize) -> Self {
        primitives::sphere(vertex::VertexTypes::empty(), radius, detail)
    }
}
