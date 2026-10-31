//! `Texture`, an image on the GPU that shaders sample, with its id, mipmaps and
//! the modules that declare, load, register, read, write and resize textures.

use crate::UniformData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod sampler;
pub use sampler::*;

mod format;
pub use format::*;

mod usage;
pub use usage::TextureUsage;

mod platform;
#[cfg(python)]
pub(crate) use platform::{
    PySamplerOptions, PyTextureBytes, PyTextureId, PyTextureSource, PyTextureUsage, read_ndarray,
};
#[cfg(mobile)]
pub use platform::{TextureInputMobile, TextureRegionMobile};
#[cfg(wasm)]
pub(crate) use platform::{
    js_to_texture_bytes, js_to_texture_id, parse_compare, read_format, read_texture_input,
    read_usage,
};

use crate::{RenderContext, Size};

mod error;
pub use error::*;

mod input;
pub use input::*;

pub(crate) mod loading;
mod mipmap;
mod object;
pub(crate) mod read;
mod region;
mod registration;
mod resize;
mod write;

pub(crate) use loading::decode::swap_bgra;
pub(crate) use loading::{SourceData, TextureSource};
pub use mipmap::Mipmap;
pub(crate) use mipmap::{mip_level_count_for, supports_cpu_mipmaps};
pub(crate) use object::{TextureObject, pick_dimension};
#[cfg(python)]
pub(crate) use read::read_pixels;
pub(crate) use read::{copy_pixels, read_pixels_from};
pub use region::TextureRegion;
pub(crate) use registration::Registration;
pub(crate) use resize::{check_extent, check_plane, check_size, resize};
pub(crate) use usage::check_usage;

mod declaration;
pub use declaration::{
    TextureClass, TextureDim, TextureMeta, TextureScalarKind, TextureStorageAccess,
    TextureStorageFormat,
};

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object, get_all, eq, hash, frozen))]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TextureId {
    /// Names a texture while it stays registered, in the `Renderer` that made it.
    /// A Material map's id names the copy in each `Renderer` that loaded that map.
    // Read-only in JavaScript, as Python's frozen class is.
    // A uniffi Record is a copy, so Swift and Kotlin never change the original.
    #[cfg_attr(wasm, wasm_bindgen(readonly))]
    pub id: u64,
}

impl TextureId {
    /// Takes a new id from the counter that registrations and Material maps share,
    /// so no two textures in the process ever get the same one.
    pub(crate) fn create() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl From<u64> for TextureId {
    fn from(value: u64) -> Self {
        TextureId { id: value }
    }
}

impl From<&TextureId> for TextureId {
    fn from(value: &TextureId) -> Self {
        *value
    }
}

impl From<TextureId> for u64 {
    fn from(value: TextureId) -> Self {
        value.id
    }
}

crate::impl_fc_kind!(TextureId, "TextureId");
crate::impl_js_bridge!(TextureId, crate::texture::TextureError);

impl std::fmt::Display for TextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Clone, Debug)]
#[lsp_doc("docs/api/texture/texture/texture.md")]
pub struct Texture {
    pub(crate) context: Arc<RenderContext>,
    pub(crate) registration: Arc<Registration>,
}

crate::impl_fc_kind!(Texture, "Texture");

impl Texture {
    /// Creates a new Texture from a RenderContext.
    pub(crate) fn new(context: Arc<RenderContext>, registration: Arc<Registration>) -> Self {
        Self {
            context,
            registration,
        }
    }

    /// The texture this handle names now. A resize swaps it for every handle.
    pub(crate) fn object(&self) -> Arc<TextureObject> {
        self.registration.object()
    }

    /// Return the stable TextureId for this texture.
    /// The id is valid within the Renderer that created it.
    #[lsp_doc("docs/api/texture/texture/id.md")]
    pub fn id(&self) -> &TextureId {
        self.registration.id()
    }

    #[lsp_doc("docs/api/texture/texture/size.md")]
    pub fn size(&self) -> crate::Size {
        self.object().size()
    }

    #[lsp_doc("docs/api/texture/texture/aspect.md")]
    pub fn aspect(&self) -> f32 {
        self.object().aspect()
    }

    #[lsp_doc("docs/api/texture/texture/set_sampler_options.md")]
    pub fn set_sampler_options(&self, options: SamplerOptions) {
        self.registration
            .apply(|object| object.set_sampler_options(&self.context.device, options));
    }

    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write(&self, data: &[u8]) -> Result<(), TextureError> {
        write::write(self, data, None)
    }

    #[lsp_doc("docs/api/texture/texture/write_region.md")]
    pub fn write_region<R>(&self, data: &[u8], region: R) -> Result<(), TextureError>
    where
        R: TryInto<TextureRegion>,
        TextureError: From<R::Error>,
    {
        write::write(self, data, Some(region.try_into()?))
    }

    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub async fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        read::get_image(self).await
    }

    #[lsp_doc("docs/api/texture/texture/resize.md")]
    pub fn resize(&self, size: impl Into<Size>) -> Result<(), TextureError> {
        resize::resize(&self.context, &self.registration, size.into())
    }
}

// UniformData conversion: allow shader.set("key", &Texture)
impl From<&Texture> for UniformData {
    fn from(texture: &Texture) -> Self {
        // Provide a placeholder meta; storage.update will merge with shader-parsed meta at set time.
        let meta = TextureMeta::from(*texture.id());
        UniformData::Texture(meta)
    }
}

crate::impl_js_bridge!(Texture, TextureError);

#[cfg(test)]
mod tests;
