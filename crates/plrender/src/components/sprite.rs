use crate::{
    renderer::texture::TextureId,
    scene::{macros::has_node_id, node::NodeId, SceneObject},
};
use std::ops::Range;

pub type UvRange = Range<mint::Point2<i16>>;

pub struct Sprite {
    pub node_id: NodeId,
    pub image: TextureId,
    pub uv: Option<UvRange>,
}

has_node_id!(Sprite);

// @TODO maybe not needed...
impl SceneObject<Sprite> {
    pub fn uv(&mut self, uv: UvRange) -> &mut Self {
        self.object.uv(uv);
        self
    }
}

impl Sprite {
    pub fn new(image: TextureId, uv: UvRange) -> Self {
        Sprite {
            node_id: NodeId::root(),
            image: image,
            uv: Some(uv), // NOTE: used to be uv.take() in the old builder
        }
    }

    pub fn uv(&mut self, uv: UvRange) -> &mut Self {
        self.uv = Some(uv);
        self
    }
}
