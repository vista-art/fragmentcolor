use crate::renderer::texture::TextureId;
use crate::scene::{entity::EntityId, node::NodeId, space::Space, ObjectBuilder};
use std::ops::Range;

pub type UvRange = Range<mint::Point2<i16>>;

pub struct Sprite {
    pub node: NodeId,
    pub image: TextureId,
    pub uv: Option<UvRange>,
}

pub struct SpriteBuilder {
    pub(crate) raw: hecs::EntityBuilder,
    pub(crate) image: TextureId,
    pub(crate) uv: Option<UvRange>,
}

impl ObjectBuilder<'_, SpriteBuilder> {
    pub fn uv(&mut self, uv: UvRange) -> &mut Self {
        self.object.uv = Some(uv);
        self
    }

    /// Register additional data for this sprite.
    pub fn component<T: hecs::Component>(&mut self, component: T) -> &mut Self {
        self.object.raw.add(component);
        self
    }

    pub fn build(&mut self) -> EntityId {
        let sprite = Sprite {
            node: if self.node.local == Space::default() {
                self.node.parent
            } else {
                self.scene.set_node_id(&mut self.node)
            },
            image: self.object.image,
            uv: self.object.uv.take(),
        };

        // DESIGN note:
        // The pattern below is what I want to expose in my public API
        // with nicer names.
        // let object = plr::SomeObject()
        // scene.add(object);

        // @TODO nitpick: this line joins the two words that I want to remove
        // from the engine: "kind" and "raw". I wanted to replace them before,
        // and now I want to replace them even more!

        // In this context, "kind" is the type of object (Sprite in this case),
        // and "raw" is the hecs::EntityBuilder that is used to build the object.
        // The method "add" is used to add Components to the Entity.
        let built = self.object.raw.add(sprite).build();

        self.scene.world.spawn(built)
    }
}
