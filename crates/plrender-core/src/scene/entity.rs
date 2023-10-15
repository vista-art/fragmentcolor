use crate::renderer::resources::mesh::MeshRef;
use crate::scene::{builder::ObjectBuilder, node::NodeRef, space::Space};

pub type EntityRef = hecs::Entity;

pub struct EntityBuilder {
    pub(super) raw: hecs::EntityBuilder,
    pub(super) mesh: MeshRef,
}

pub struct Entity {
    pub node: NodeRef,
    pub mesh: MeshRef,
}

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
                self.scene.add_node_impl(&mut self.node)
            },
            mesh: self.kind.mesh,
        };
        let built = self.kind.raw.add(entity).build();
        self.scene.world.spawn(built)
    }
}
