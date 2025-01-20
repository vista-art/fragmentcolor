use crate::{
    math::{cg::Vec3, geometry::Primitive},
    resources::mesh::{BuiltMesh, MeshId},
    scene::transform::TransformId,
    Renderer,
};

/// The Mesh component
#[derive(Debug, Default, Clone, Copy)]
pub struct Mesh {
    pub mesh_id: MeshId,
    pub transform_id: TransformId,
}

impl Mesh {
    pub fn new(built_mesh: Option<BuiltMesh>) -> Self {
        if let Some(built_mesh) = built_mesh {
            Mesh {
                transform_id: TransformId::root(),
                mesh_id: built_mesh.id,
            }
        } else {
            log::warn!(
                "Mesh::new() called with None! Did the BuiltMesh failed to load?
                Creating an Empty object."
            );
            Mesh::default()
        }
    }
}

impl Mesh {
    pub fn id(&self) -> MeshId {
        self.mesh_id
    }

    pub fn set_mesh(&mut self, built_mesh: Option<BuiltMesh>) -> &mut Self {
        if let Some(built_mesh) = built_mesh {
            self.mesh_id = built_mesh.id;
        } else {
            log::warn!(
                "Mesh::set_mesh() called with None! Did the BuiltMesh failed to load?
                Keeping the current mesh."
            );
        }
        self
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Box;
impl Box {
    pub fn new<V: Into<Vec3>>(renderer: &Renderer, dimensions: V) -> Mesh {
        let cuboid = Primitive::cuboid(dimensions).create_mesh(renderer).ok();
        Mesh::new(cuboid)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Cube;
impl Cube {
    pub fn new(renderer: &Renderer, size: f32) -> Mesh {
        let cube = Primitive::cube(size).create_mesh(renderer).ok();
        Mesh::new(cube)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Plane;
impl Plane {
    pub fn new(renderer: &Renderer, size: f32) -> Mesh {
        let plane = Primitive::plane(size).create_mesh(renderer).ok();
        Mesh::new(plane)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Sphere;
impl Sphere {
    pub fn new(renderer: &Renderer, radius: f32, detail: usize) -> Mesh {
        let sphere = Primitive::sphere(radius, detail).create_mesh(renderer).ok();
        Mesh::new(sphere)
    }
}

// @TODO [ ] Custom Mesh Shapes (2D Tesselator)

// @TODO [ ] SVG Loading
