use crate::{
    components::Transform,
    renderer::texture::TextureId,
    scene::{node::NodeId, EntityId, SceneObject},
};
use std::ops::Range;

pub type UvRange = Range<mint::Point2<i16>>;

pub struct Sprite {
    pub node: NodeId,
    pub image: TextureId,
    pub uv: Option<UvRange>,
}

pub struct SpriteBuilder {
    pub(crate) builder: hecs::EntityBuilder,
    pub(crate) image: TextureId,
    pub(crate) uv: Option<UvRange>,
}

impl SceneObject<'_, SpriteBuilder> {
    pub fn uv(&mut self, uv: UvRange) -> &mut Self {
        self.object.uv = Some(uv);
        self
    }

    /// Register additional data for this sprite.
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.object.builder.add(component);
        self
    }

    pub fn build(&mut self) -> EntityId {
        let sprite = Sprite {
            node: if self.node.local() == Transform::default() {
                self.node.parent()
            } else {
                self.scene.insert_scene_tree_node(&mut self.node)
            },
            image: self.object.image,
            uv: self.object.uv.take(),
        };

        // In this context, "object" is the type of object (Sprite in this case),
        // and "builder" is the hecs::EntityBuilder that is used to build the object.
        // The method "add" is used to add Components to the Renderable.
        let built = self.object.builder.add(sprite).build();

        self.scene.add(built)
    }
}
