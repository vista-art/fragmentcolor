use crate::renderer::texture::TextureId;
use crate::scene::{entity::EntityRef, node::NodeId, space::Space, ObjectBuilder};
use std::ops::Range;

pub type UvRange = Range<mint::Point2<i16>>;

pub struct Sprite {
    pub node: NodeId,
    pub image: TextureId,
    pub uv: Option<UvRange>,
}

pub struct SpriteBuilder {
    pub(super) raw: hecs::EntityBuilder,
    pub(super) image: TextureId,
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
                self.scene.set_node_id(&mut self.node)
            },
            image: self.kind.image,
            uv: self.kind.uv.take(),
        };

        // DESIGN note:
        // The pattern below is what I want to expose in my public API
        // with nicer names.
        // let object = plr::SomeObject()
        // scene.add(object);

        // @TODO nitpick: this line joins the two words that I want to remove
        // from the engine: "kind" and "raw". I wanted to replace them before,
        // and now I want to replace them even more!
        let built = self.kind.raw.add(sprite).build();

        self.scene.world.spawn(built)
    }
}
