use crate::renderer::resources::mesh::MeshId;
use crate::scene::{builder::ObjectBuilder, node::NodeId, space::Space};

pub type EntityId = hecs::Entity;

pub struct EntityBuilder {
    pub(super) builder: hecs::EntityBuilder,
    pub(super) mesh: MeshId,
}

pub struct Entity {
    pub node: NodeId,
    pub mesh: MeshId,
}

impl EntityBuilder {
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.builder.add(component);
        self
    }
}

// ACHEI O QUE EU QUERIA!
// Provavelmente essa é a parte que vou MANTER
impl ObjectBuilder<'_, EntityBuilder> {
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.object.builder.add(component);
        self
    }

    pub fn build(&mut self) -> EntityId {
        let entity = Entity {
            node: if self.node.local == Space::default() {
                self.node.parent
            } else {
                self.scene.set_node_id(&mut self.node)
            },
            mesh: self.object.mesh,
        };
        // This is the BuiltEntity object from hecs
        // suitable as an input to World::spawn
        let built = self.object.builder.add(entity).build();
        self.scene.world.spawn(built)
    }
}
