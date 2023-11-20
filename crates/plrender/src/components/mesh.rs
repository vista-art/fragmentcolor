use crate::{
    math::geometry::Primitive,
    resources::mesh::{BuiltMesh, MeshId},
    scene::{macros::spatial_object, node::NodeId, SceneObject},
};

/// The Mesh component
#[derive(Debug, Default, Clone, Copy)]
pub struct Mesh {
    pub mesh_id: MeshId,
    pub(crate) node_id: NodeId,
}

spatial_object!(Mesh);

impl Mesh {
    pub fn new(built_mesh: &BuiltMesh) -> SceneObject<Self> {
        let mut mesh = SceneObject::new(Mesh {
            node_id: NodeId::root(),
            mesh_id: built_mesh.id,
        });
        mesh.add_components(built_mesh);

        mesh
    }
}

pub struct Box;
impl Box {
    pub fn new(dimensions: mint::Vector3<f32>) -> SceneObject<Mesh> {
        let cube = Primitive::cuboid(dimensions).create_mesh();

        Mesh::new(&cube)
    }
}

pub struct Cube;
impl Cube {
    pub fn new(size: f32) -> SceneObject<Mesh> {
        let cube = Primitive::cube(size).create_mesh();

        Mesh::new(&cube)
    }
}

pub struct Plane;
impl Plane {
    pub fn new(size: f32) -> SceneObject<Mesh> {
        let plane = Primitive::plane(size).create_mesh();

        Mesh::new(&plane)
    }
}

pub struct Sphere;
impl Sphere {
    pub fn new(radius: f32, detail: usize) -> SceneObject<Mesh> {
        let sphere = Primitive::sphere(radius, detail).create_mesh();

        Mesh::new(&sphere)
    }
}

// @TODO [ ] Custom Mesh Shapes (2D Tesselator)

// @TODO [ ] SVG Loading
