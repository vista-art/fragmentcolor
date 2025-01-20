use std::path::Path;

use crate::{
    math::geometry::Quad,
    resources::texture::{Texture, TextureId},
    transform::TransformId,
    Renderer,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Sprite {
    pub texture_id: Option<TextureId>,
    pub image_size: Quad,
    pub clip_region: Option<Quad>,
    pub transform_id: TransformId,
}

impl Sprite {
    pub fn new(renderer: &Renderer, image_path: impl AsRef<Path>) -> Sprite {
        let (texture_id, texture_size) = Self::load_image(renderer, image_path);

        Self {
            texture_id,
            image_size: texture_size,
            clip_region: None,
            transform_id: Default::default(),
        }
    }

    pub fn load_image(renderer: &Renderer, path: impl AsRef<Path>) -> (Option<TextureId>, Quad) {
        let path = path.as_ref();

        if let Ok((texture_id, size)) = Texture::from_file(renderer, path) {
            (Some(texture_id), size)
        } else {
            log::warn!("Image {:?} not found!", path);
            (None, Quad::default())
        }
    }

    pub fn set_image(&mut self, renderer: &Renderer, bytes: &[u8]) -> &mut Self {
        if let Ok((image, size)) = Texture::from_bytes(renderer, bytes) {
            self.set_texture(image, size)
        } else {
            log::warn!("Sprite::set_image() failed to parse Image bytes! Image will be empty.");
            self
        }
    }

    fn set_texture(&mut self, image: TextureId, texture_size: Quad) -> &mut Self {
        self.texture_id = Some(image);
        self.image_size = texture_size;

        self
    }

    pub fn set_clip_region(&mut self, clip_region: Quad) -> &mut Self {
        self.clip_region = Some(clip_region);
        self
    }
}
