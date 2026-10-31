//! `Renderer`: the entry point that draws passes
//! into targets, handing each call to its render context.

use crate::texture::{Texture, TextureId, TextureInput};
use crate::{RenderTarget, Shader, Size, Target, TextureTarget};
use futures::lock::Mutex as AsyncMutex;
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use pyo3::prelude::*;

pub(crate) mod platform;
#[cfg(any(python, mobile))]
pub use platform::*;

mod error;
pub use error::*;

mod renderable;
pub use renderable::*;

mod logging;
pub use logging::LogLevel;

mod pools;
pub(crate) use pools::*;

mod checks;
pub(crate) use checks::check_limit;

mod context;
pub(crate) use context::RenderContext;

mod surface;
pub use surface::HasDisplaySize;

pub(crate) mod device;
mod pipelines;
pub(crate) use pipelines::vertex_layouts::VertexLayouts;
mod frame;
pub(crate) mod lights;
mod storage_readback;
mod texture_registry;

mod device_errors;
pub(crate) use device_errors::Start;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
#[cfg_attr(python, pyclass)]
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[lsp_doc("docs/api/core/renderer/renderer.md")]
pub struct Renderer {
    // Shared, so a JS export can hand a future a handle that outlives its borrow.
    instance: Arc<RwLock<Option<Arc<wgpu::Instance>>>>,
    adapter: Arc<RwLock<Option<wgpu::Adapter>>>,
    context: Arc<RwLock<Option<Arc<RenderContext>>>>,
    /// Held while the instance is created, so calls that race create it once.
    instance_lock: Arc<AsyncMutex<()>>,
    /// Held while the context is created, so calls that race share one device.
    context_lock: Arc<AsyncMutex<()>>,
}

crate::impl_fc_kind!(Renderer, "Renderer");

impl Renderer {
    #[lsp_doc("docs/api/core/renderer/new.md")]
    pub fn new() -> Self {
        Self::default()
    }

    #[lsp_doc("docs/api/core/renderer/set_log_level.md")]
    pub fn set_log_level(level: LogLevel) {
        log::set_max_level(level.to_filter());
    }

    /// A second handle on this renderer's instance, adapter and context,
    /// which takes the same locks to create them.
    #[cfg(wasm)]
    pub(crate) fn share(&self) -> Self {
        Renderer {
            instance: self.instance.clone(),
            adapter: self.adapter.clone(),
            context: self.context.clone(),
            instance_lock: self.instance_lock.clone(),
            context_lock: self.context_lock.clone(),
        }
    }

    /// The start of a call made now, for one that waits for earlier frames.
    pub(crate) fn start(&self) -> Start {
        Start::now(self.context.read().as_deref())
    }

    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    pub async fn create_target(
        &self,
        window: impl HasWindowHandle + HasDisplayHandle + HasDisplaySize + Send + Sync + 'static,
    ) -> Result<RenderTarget, InitializationError> {
        self.open_target(window).await
    }

    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    pub async fn create_texture_target(
        &self,
        size: impl Into<Size>,
    ) -> Result<TextureTarget, InitializationError> {
        self.allocate_target(size.into()).await
    }

    /// Create a [Texture](crate::Texture) from a unified spec. The single
    /// entry point for every shape — bare bytes / path / URL / file (encoded),
    /// `(input, [w, h])` or `(input, Size)` for raw pixel bytes,
    /// `(input, TextureFormat)` for an explicit format override,
    /// `(input, TextureOptions)` for full control, or a `Mipmap`
    /// (built off the renderer thread) for a GPU-only upload.
    ///
    /// The CPU work (decode, mipmap chain, raw-byte wrap) runs on a background
    /// worker on every native target; the calling thread is never pinned.
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub async fn create_texture(
        &self,
        spec: impl Into<TextureInput>,
    ) -> Result<Texture, RendererError> {
        self.upload_texture(spec.into()).await
    }

    /// Create a storage-class texture from a [`crate::TextureInput`], the
    /// same input type `create_texture` takes. The `From<T>` impls cover
    /// the common shapes:
    ///
    /// - `(size, format)` → empty storage texture, no initial data.
    /// - `(size, format, bytes)` → storage texture pre-seeded with `bytes`.
    ///
    /// `options.usage` overrides the default storage usage mask
    /// (`STORAGE | TEXTURE | COPY_SRC | COPY_DST`); use
    /// `TextureOptions::with_usage(...)` for the typed builder. `options.size`
    /// is **required** for this entry, since there's no source to infer
    /// dimensions from. Returns [`RendererError::CreateTextureError`] when
    /// missing.
    #[lsp_doc("docs/api/core/renderer/create_storage_texture.md")]
    pub async fn create_storage_texture(
        &self,
        input: impl Into<TextureInput>,
    ) -> Result<Texture, RendererError> {
        self.allocate_storage(input.into()).await
    }

    /// Create a depth texture (Depth32Float).
    #[lsp_doc("docs/api/core/renderer/create_depth_texture.md")]
    pub async fn create_depth_texture(
        &self,
        size: impl Into<Size>,
    ) -> Result<Texture, InitializationError> {
        self.allocate_depth(size.into()).await
    }

    #[lsp_doc("docs/api/core/renderer/unregister_texture.md")]
    pub fn unregister_texture(
        &self,
        texture_id: impl Into<TextureId>,
    ) -> Result<(), RendererError> {
        texture_registry::unregister_texture(self, texture_id.into())
    }

    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub async fn read_texture(
        &self,
        texture_id: impl Into<TextureId>,
    ) -> Result<Vec<u8>, RendererError> {
        self.read_texture_from(texture_id.into(), self.start())
            .await
    }

    /// Uploads into this Renderer every Material map of `renderable` that it lacks
    /// or holds from an older set, so `render` against the same renderable is GPU-only.
    ///
    /// On native targets `render` runs `load` first, so calling it is optional
    /// there. On wasm `render` can't block, so await `load` before the first frame.
    #[lsp_doc("docs/api/core/renderer/load.md")]
    pub async fn load(&self, renderable: &impl Renderable) -> Result<(), RendererError> {
        self.load_from(renderable, self.start()).await
    }

    /// Read the GPU-side bytes of a storage binding back to the CPU.
    ///
    /// Submits any pending CPU→GPU uploads, copies the storage buffer into a
    /// pooled readback staging buffer, maps it, and returns the raw bytes.
    /// The caller is expected to `bytemuck::cast_slice::<T>` (or equivalent)
    /// to recover the typed view.
    ///
    /// Returns [`RendererError::StorageBindingNotFound`] when the shader does
    /// not declare `binding`, or when no render pass has yet materialized the
    /// GPU-side buffer (the registry is populated lazily on first bind).
    ///
    /// This is intentionally explicit and async because the underlying
    /// `map_async` is async on every platform — keeping it visible at the
    /// call site so callers don't accidentally pay a GPU sync per frame.
    /// [`Shader::get`] stays the cheap read of what was last set on the CPU side.
    #[lsp_doc("docs/api/core/renderer/read_storage.md")]
    pub async fn read_storage(
        &self,
        shader: &Shader,
        binding: &str,
    ) -> Result<Vec<u8>, RendererError> {
        self.read_storage_from(shader, binding, self.start()).await
    }

    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), RendererError> {
        self.draw(renderable, target)
    }
}
