#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::MaterialMap;
use crate::bindings::numbers::{read_finite, read_vector};
use crate::errors::platform::mobile::FragmentColorError;
use crate::texture::{MobileTextureInput, Texture, TextureInput, TextureOptions};

#[uniffi::export]
impl MaterialMap {
    #[uniffi::method(name = "texture")]
    #[lsp_doc("docs/api/scene/material_map/texture.md")]
    pub fn texture_mobile(
        self: Arc<Self>,
        texture: Arc<Texture>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(self.texture(&*texture)?))
    }

    /// The texture input form of `texture`, uploaded when the scene loads.
    /// Swift and Kotlin reach it through the `texture` overloads, which match `createTexture`.
    #[uniffi::method(name = "source", default(options = None))]
    #[lsp_doc("docs/api/scene/material_map/hidden/source_mobile.md")]
    pub fn source_mobile(
        self: Arc<Self>,
        input: MobileTextureInput,
        options: Option<TextureOptions>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let input = TextureInput {
            data: input.into(),
            options: options.unwrap_or_default(),
        };
        Ok(Arc::new(self.texture(input)?))
    }

    #[uniffi::method(name = "uv")]
    #[lsp_doc("docs/api/scene/material_map/uv.md")]
    pub fn uv_mobile(self: Arc<Self>, set: u32) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(self.uv(set)?))
    }

    #[uniffi::method(name = "transform")]
    #[lsp_doc("docs/api/scene/material_map/transform.md")]
    pub fn transform_mobile(
        self: Arc<Self>,
        offset: Vec<f32>,
        scale: Vec<f32>,
        rotation: f32,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let offset = read_vector::<2>(offset, "MaterialMap.transform: offset")?;
        let scale = read_vector::<2>(scale, "MaterialMap.transform: scale")?;
        let rotation = read_finite(rotation, "MaterialMap.transform: rotation")?;
        Ok(Arc::new(self.transform(offset, scale, rotation)?))
    }
}
