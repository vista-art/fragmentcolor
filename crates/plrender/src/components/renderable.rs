use crate::{
    components::transform::Transform,
    renderer::resources::mesh::MeshId,
    scene::{node::NodeId, object::SceneObject},
    EntityId,
};

pub struct RenderableBuilder {
    pub builder: hecs::EntityBuilder,
    pub mesh_id: MeshId,
}

/// The Renderable component
#[derive(Debug)]
pub struct Renderable {
    pub node_id: NodeId,
    pub mesh_id: MeshId,
}

impl Renderable {
    pub fn new(node_id: NodeId, mesh_id: MeshId) -> Self {
        Self { node_id, mesh_id }
    }
}

impl RenderableBuilder {
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.builder.add(component);
        self
    }
}

impl SceneObject<'_, RenderableBuilder> {
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.object.builder.add(component);
        self
    }

    pub fn add_to_scene(&mut self) -> EntityId {
        let renderable = Renderable {
            node_id: if self.node.local() == Transform::default() {
                self.node.parent()
            } else {
                self.scene.insert_scene_tree_node(&mut self.node)
            },
            mesh_id: self.object.mesh_id,
        };

        let entity = self.object.builder.add(renderable).build();
        self.scene.add(entity)
    }
}
