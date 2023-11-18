use std::path::Path;

use crate::{
    app::error::READ_LOCK_ERROR,
    math::geometry::Quad,
    resources::texture::TextureId,
    scene::{macros::spatial_object, node::NodeId, SceneObject},
    PLRender,
};

type Error = Box<dyn std::error::Error>;

#[derive(Debug, Default, Clone, Copy)]
pub struct Sprite {
    pub node_id: NodeId,
    pub image: Option<TextureId>,
    pub clip_region: Option<Quad>,
}

spatial_object!(Sprite);

impl SceneObject<Sprite> {
    pub fn set_uv(&mut self, clip_region: Quad) -> &mut Self {
        let sprite = self.object();

        self.add_component(Sprite {
            clip_region: Some(clip_region),
            ..sprite
        });

        self
    }

    pub fn set_image(&mut self, image: TextureId) -> &mut Self {
        let sprite = self.object();

        self.add_component(Sprite {
            image: Some(image),
            ..sprite
        });

        self
    }

    pub fn clear_image(&mut self) -> &mut Self {
        let sprite = self.object();

        self.add_component(Sprite {
            image: None,
            ..sprite
        });

        self
    }
}

impl Sprite {
    pub fn new(image: TextureId) -> SceneObject<Sprite> {
        SceneObject::new(Sprite {
            node_id: NodeId::root(),
            image: Some(image),
            clip_region: None,
        })
    }

    pub fn from_image(image_path: impl AsRef<Path>) -> Result<SceneObject<Sprite>, Error> {
        let renderer = PLRender::renderer().read().expect(READ_LOCK_ERROR);
        let image = renderer.load_image(image_path)?;

        Ok(Self::new(image))
    }

    pub fn with_clip_region(image: TextureId, clip_region: Quad) -> SceneObject<Sprite> {
        SceneObject::new(Sprite {
            node_id: NodeId::root(),
            image: Some(image),
            clip_region: Some(clip_region),
        })
    }
}
