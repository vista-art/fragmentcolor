use crate::{
    renderer::resources::mesh::MeshId,
    scene::{macros::has_node_id, node::NodeId, SceneObject},
};

/// The Renderable component
#[derive(Debug)]
pub struct Renderable {
    pub node_id: NodeId,
    pub mesh_id: MeshId,
}

has_node_id!(Renderable);

impl SceneObject<Renderable> {
    pub fn set_mesh(&mut self, mesh_id: MeshId) -> &mut Self {
        self.object.mesh_id = mesh_id;
        self
    }
}

impl Renderable {
    pub fn new(mesh_id: MeshId) -> Self {
        Renderable {
            node_id: NodeId::root(),
            mesh_id,
        }
    }
}
