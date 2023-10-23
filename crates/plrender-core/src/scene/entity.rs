use crate::renderer::resources::mesh::MeshId;
use crate::scene::{builder::ObjectBuilder, node::NodeId, space::Space};

pub type EntityRef = hecs::Entity;

pub struct EntityBuilder {
    pub(super) raw: hecs::EntityBuilder,
    pub(super) mesh: MeshId,
}

pub struct Entity {
    pub node: NodeId,
    pub mesh: MeshId,
}

// ACHEI O QUE EU QUERIA!
// Provavelmente essa é a parte que vou MANTER
impl ObjectBuilder<'_, EntityBuilder> {
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.kind.raw.add(component);
        self
    }

    pub fn build(&mut self) -> EntityRef {
        let entity = Entity {
            node: if self.node.local == Space::default() {
                self.node.parent
            } else {
                self.scene.set_node_id(&mut self.node)
            },
            mesh: self.kind.mesh,
        };
        let built = self.kind.raw.add(entity).build();
        self.scene.world.spawn(built)
    }
}
