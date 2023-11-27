use crate::{
    math::cg::Vec3,
    math::geometry::Primitive,
    resources::mesh::{BuiltMesh, MeshId},
    scene::{macros::api_object, transform::TransformId, Object},
};

/// The Mesh component
#[derive(Debug, Default, Clone, Copy)]
pub struct Mesh {
    pub mesh_id: MeshId,
    pub(crate) transform_id: TransformId,
}

api_object!(Mesh);

impl Mesh {
    pub fn new(built_mesh: Option<BuiltMesh>) -> Object<Self> {
        if let Some(built_mesh) = built_mesh {
            let mut mesh = Object::new(Mesh {
                transform_id: TransformId::root(),
                mesh_id: built_mesh.id,
            });
            mesh.add_components(built_mesh);
            mesh
        } else {
            log::warn!(
                "Mesh::new() called with None! Did the BuiltMesh failed to load?
                Creating an Empty object."
            );
            Object::new(Mesh::default())
        }
    }
}

impl Object<Mesh> {
    pub fn mesh(&self) -> MeshId {
        self.object().mesh_id
    }

    pub fn set_mesh(&mut self, built_mesh: Option<BuiltMesh>) -> &mut Self {
        let mesh = self.object();

        if let Some(built_mesh) = built_mesh {
            self.add_component(Mesh {
                mesh_id: built_mesh.id,
                ..mesh
            });

            self.add_components(built_mesh);
        } else {
            log::warn!(
                "Object<Mesh>.set_mesh() called with None! Did the Mesh failed to load?
                Object's internal Mesh could not be updated: {:?}",
                self
            );
            self.add_component(Mesh::default());
        }

        self
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Box;
impl Box {
    pub fn new<V: Into<Vec3>>(dimensions: V) -> Object<Mesh> {
        let cuboid = Primitive::cuboid(dimensions).create_mesh().ok();
        Mesh::new(cuboid)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Cube;
impl Cube {
    pub fn new(size: f32) -> Object<Mesh> {
        let cube = Primitive::cube(size).create_mesh().ok();
        Mesh::new(cube)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Plane;
impl Plane {
    pub fn new(size: f32) -> Object<Mesh> {
        let plane = Primitive::plane(size).create_mesh().ok();
        Mesh::new(plane)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Sphere;
impl Sphere {
    pub fn new(radius: f32, detail: usize) -> Object<Mesh> {
        let sphere = Primitive::sphere(radius, detail).create_mesh().ok();
        Mesh::new(sphere)
    }
}

// @TODO [ ] Custom Mesh Shapes (2D Tesselator)

// @TODO [ ] SVG Loading
