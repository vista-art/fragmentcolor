use crate::renderer::resources::mesh::MeshId;
use crate::scene::{builder::ObjectBuilder, components::transform::Transform, node::NodeId};

pub type RenderableId = hecs::Entity;

pub struct RenderableBuilder {
    pub(crate) builder: hecs::EntityBuilder,
    pub(crate) mesh_id: MeshId,
}

pub struct Renderable {
    pub node_id: NodeId,
    pub mesh_id: MeshId,
}

impl RenderableBuilder {
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.builder.add(component);
        self
    }
}

// ACHEI O QUE EU QUERIA!
// Provavelmente essa é a parte que vou MANTER
impl ObjectBuilder<'_, RenderableBuilder> {
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.object.builder.add(component);
        self
    }

    pub fn build(&mut self) -> RenderableId {
        let entity = Renderable {
            node_id: if self.node.local == Transform::default() {
                self.node.parent
            } else {
                self.scene.set_node_id(&mut self.node)
            },
            mesh_id: self.object.mesh_id,
        };
        // BuiltRenderable object from hecs suitable as an input to World::spawn
        let built = self.object.builder.add(entity).build();
        self.scene.world.spawn(built)
    }
}
