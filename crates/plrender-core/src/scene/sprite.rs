use crate::renderer::texture::ImageRef;
use crate::scene::{entity::EntityRef, node::NodeRef, space::Space, ObjectBuilder};
use std::ops::Range;

pub type UvRange = Range<mint::Point2<i16>>;

pub struct Sprite {
    pub node: NodeRef,
    pub image: ImageRef,
    pub uv: Option<UvRange>,
}

pub struct SpriteBuilder {
    pub(super) raw: hecs::EntityBuilder,
    pub(super) image: ImageRef,
    pub(super) uv: Option<UvRange>,
}

impl ObjectBuilder<'_, SpriteBuilder> {
    pub fn uv(&mut self, uv: UvRange) -> &mut Self {
        self.kind.uv = Some(uv);
        self
    }

    /// Register additional data for this sprite.
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.kind.raw.add(component);
        self
    }

    pub fn build(&mut self) -> EntityRef {
        let sprite = Sprite {
            node: if self.node.local == Space::default() {
                self.node.parent
            } else {
                self.scene.add_node_impl(&mut self.node)
            },
            image: self.kind.image,
            uv: self.kind.uv.take(),
        };
        let built = self.kind.raw.add(sprite).build();
        self.scene.world.spawn(built)
    }
}
