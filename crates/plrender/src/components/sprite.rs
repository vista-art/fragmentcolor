use std::path::Path;

use crate::{
    app,
    math::geometry::Quad,
    resources::texture::{Texture, TextureId},
    scene::{macros::spatial_object, node::NodeId, SceneObject},
};

const DEFAULT_IMAGE: &str = "resources/images/default.jpg";

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
    pub fn new(image_path: impl AsRef<Path>) -> SceneObject<Sprite> {
        let texture_id = Self::load_image(image_path);
        Self::from_texture_id(texture_id)
    }

    pub fn from_texture_id(texture_id: TextureId) -> SceneObject<Sprite> {
        SceneObject::new(Sprite {
            node_id: NodeId::root(),
            image: Some(texture_id),
            clip_region: None,
        })
    }

    pub fn with_clip_region(
        image_path: impl AsRef<Path>,
        clip_region: Quad,
    ) -> SceneObject<Sprite> {
        let texture_id = Self::load_image(image_path);
        SceneObject::new(Sprite {
            node_id: NodeId::root(),
            image: Some(texture_id),
            clip_region: Some(clip_region),
        })
    }

    fn load_image(path: impl AsRef<Path>) -> TextureId {
        // @TODO set root path in the build.rs file
        let default = format!("{}/src/{}", app::ROOT, DEFAULT_IMAGE);
        let full_path = format!("{}{}", app::ROOT, path.as_ref().display());

        if let Ok(texture_id) = Texture::from_file(path) {
            texture_id
        } else if let Ok(texture_id) = Texture::from_file(full_path) {
            texture_id
        } else {
            Texture::from_file(default).expect("Default image not found!")
        }
    }
}
