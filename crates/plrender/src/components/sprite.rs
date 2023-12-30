use std::path::Path;

use crate::{
    math::geometry::Quad,
    panics,
    resources::texture::{Texture, TextureId, DEFAULT_IMAGE_SIZE},
    scene::{macros::api_object, Object},
    Border, Bounds, Color, Renderable2D, SceneObject, ShapeFlag,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Sprite {
    pub image: TextureId, // this is the only thing a Sprite should care about.
    pub image_size: Quad,
    pub clip_region: Option<Quad>,
}

api_object!(Sprite);

impl Object<Sprite> {
    pub fn load_image(&mut self, image_path: impl AsRef<Path>) -> &mut Self {
        let (image, size) = Sprite::load_image(image_path);
        self.set_texture(image, size)
    }

    pub fn set_image(&mut self, bytes: &[u8]) -> &mut Self {
        if let Ok((image, size)) = Texture::from_bytes(bytes) {
            self.set_texture(image, size)
        } else {
            log::warn!("Sprite::set_image() failed to parse Image bytes! Image will not update.");
            self
        }
    }

    fn set_texture(&mut self, image: TextureId, texture_size: Quad) -> &mut Self {
        let sprite = self.object();
        let bounds = sprite.clip_region.unwrap_or(texture_size);

        self.add_components((
            Sprite {
                image,
                image_size: texture_size,
                ..sprite
            },
            Bounds(bounds),
        ));

        self
    }

    pub fn set_clip_region(&mut self, clip_region: Quad) -> &mut Self {
        let sprite = self.object();

        self.add_component(Sprite {
            clip_region: Some(clip_region),
            ..sprite
        });

        self
    }

    pub fn image(&self) -> TextureId {
        self.object().image
    }

    pub fn clip_region(&self) -> Option<Quad> {
        self.object().clip_region
    }
}

impl Sprite {
    pub fn new(image_path: impl AsRef<Path>) -> Object<Sprite> {
        let (texture_id, texture_size) = Self::load_image(image_path);

        let mut sprite = Object::new(Self {
            image: texture_id,
            image_size: texture_size,
            clip_region: None,
        });

        // Sprite bounds is clip region or image size
        let bounds = sprite.clip_region().unwrap_or(texture_size);

        let components = Renderable2D {
            transform: sprite.transform_id(),
            bounds: Bounds(bounds),
            image: Some(texture_id),
            color: Color(0x00000000),
            border: Border(0.0),
            sdf_flags: ShapeFlag(0.0),
        };

        sprite.add_components(components);

        sprite
    }

    pub fn load_image(path: impl AsRef<Path>) -> (TextureId, Quad) {
        let path = path.as_ref();

        if let Ok((texture_id, size)) = Texture::from_file(path) {
            (texture_id, size)
        } else {
            log::warn!("Image {:?} not found! Using default image.", path);
            Self::load_default_image()
        }
    }

    fn load_default_image() -> (TextureId, Quad) {
        let (image, _) = Texture::image_not_found().expect(panics::DEFAULT_IMAGE_NOT_FOUND);
        (image, Quad::from_tuple(DEFAULT_IMAGE_SIZE))
    }
}
