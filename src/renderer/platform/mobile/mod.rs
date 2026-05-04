//! Uniffi bindings shared by iOS (Swift) and Android (Kotlin).
//!
//! Core types (`Renderer`, `Shader`, `WindowTarget`, `TextureTarget`) are
//! already annotated `#[cfg_attr(mobile, derive(uniffi::Object))]` where they
//! live. This module adds the mobile-only constructor and helper methods.
//!
//! Android's `Surface` → `WindowTarget` constructor cannot go through uniffi
//! (uniffi cannot marshal `JNIEnv*`). It is implemented in the `android`
//! submodule as a raw `#[jni_fn]` entry point that returns an `Arc` pointer;
//! the Kotlin side reconstructs the `WindowTarget` from that pointer.
//!
//! Naming convention mirrors the Web/Python platform modules: mobile-specific
//! methods carry a `_mobile` / `_ios` / `_android` suffix on the Rust side
//! so the build-time doc scanner can keep them separate from the Rust-only
//! API, and every uniffi export carries an explicit `name = "..."` attribute
//! to expose an idiomatic camelCase name in Swift and Kotlin. The hidden
//! per-language docs under `docs/api/core/{object}/hidden/<method>.md`
//! satisfy the build-time documentation validator without polluting the
//! main website.

use std::sync::Arc;

use lsp_doc::lsp_doc;

use crate::MobileTextureTarget;
use crate::{Renderer, Size};

/// Mobile-facing error type. Flattens every internal error to its `Display`
/// representation so Swift/Kotlin callers get a single typed error to match on.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FragmentColorError {
    #[error("{0}")]
    Init(String),
    #[error("{0}")]
    Render(String),
    #[error("{0}")]
    Shader(String),
}

impl From<crate::InitializationError> for FragmentColorError {
    fn from(e: crate::InitializationError) -> Self {
        FragmentColorError::Init(e.to_string())
    }
}

impl From<crate::RendererError> for FragmentColorError {
    fn from(e: crate::RendererError) -> Self {
        FragmentColorError::Render(e.to_string())
    }
}

impl From<crate::ShaderError> for FragmentColorError {
    fn from(e: crate::ShaderError) -> Self {
        FragmentColorError::Shader(e.to_string())
    }
}

impl From<crate::texture::TextureError> for FragmentColorError {
    fn from(e: crate::texture::TextureError) -> Self {
        FragmentColorError::Render(e.to_string())
    }
}

impl From<crate::pass::PassError> for FragmentColorError {
    fn from(e: crate::pass::PassError) -> Self {
        FragmentColorError::Render(e.to_string())
    }
}

#[uniffi::export]
impl Renderer {
    /// Foreign bindings see this as `Renderer.new()`. On Swift, uniffi
    /// generates a `convenience init` when the name is `new`, so callers
    /// write `let r = Renderer()`. Kotlin gets a companion `Renderer.new()`
    /// factory that the extension in `RendererExtensions.kt` wraps.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/core/renderer/hidden/new_mobile.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Create a headless `TextureTarget` sized `width` × `height`. Uniffi
    /// cannot marshal `impl Into<Size>`, so the mobile entry point accepts
    /// width/height as `u32` primitives and builds the `Size` internally.
    /// Returns a `MobileTextureTarget` wrapper that exposes `size()`,
    /// `resize()`, and `get_image()` on the Swift / Kotlin side.
    #[uniffi::method(name = "createTextureTarget")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_texture_target_mobile.md")]
    pub async fn create_texture_target_mobile(
        self: Arc<Self>,
        width: u32,
        height: u32,
    ) -> Result<Arc<MobileTextureTarget>, FragmentColorError> {
        let tex = self
            .create_texture_target(Size::new(width, height, None))
            .await
            .map_err(FragmentColorError::from)?;
        Ok(MobileTextureTarget::new(tex))
    }

    /// Single mobile entry point for texture creation. Mirrors the canonical
    /// Rust `Renderer::create_texture(impl Into<TextureInput>)` — uniffi
    /// can't marshal `impl Into`, so the mobile shim takes the input as a
    /// concrete enum + an optional `TextureOptions`. Swift / Kotlin
    /// extension files supply the natural overloads (e.g. `renderer.createTexture(bytes)`,
    /// `renderer.createTexture(chain)`) by wrapping the enum invisibly.
    #[uniffi::method(name = "createTexture")]
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub async fn create_texture_mobile(
        self: Arc<Self>,
        input: crate::texture::TextureInputMobile,
        options: Option<crate::texture::TextureOptions>,
    ) -> Result<Arc<crate::texture::Texture>, FragmentColorError> {
        let spec = crate::texture::TextureInput {
            data: input.into(),
            options: options.unwrap_or_default(),
        };
        let tex = self
            .create_texture(spec)
            .await
            .map_err(FragmentColorError::from)?;
        Ok(Arc::new(tex))
    }

    /// Allocate a storage-class texture, optionally pre-seeded with `data`.
    /// Mirrors the canonical `Renderer::create_storage_texture(impl Into<TextureInput>)`
    /// — uniffi can't marshal `impl Into`, so the mobile shim takes the
    /// fields directly and the body builds a `TextureInput` from them.
    /// `usage_bits = None` defaults to STORAGE | TEXTURE | COPY_SRC | COPY_DST.
    #[uniffi::method(name = "createStorageTexture")]
    #[lsp_doc("docs/api/core/renderer/create_storage_texture.md")]
    pub async fn create_storage_texture_mobile(
        self: Arc<Self>,
        size: Size,
        format: crate::TextureFormat,
        data: Option<Vec<u8>>,
        usage_bits: Option<u32>,
    ) -> Result<Arc<crate::texture::Texture>, FragmentColorError> {
        let input = crate::TextureInput {
            data: match data {
                Some(bytes) => crate::TextureData::Bytes(bytes),
                None => crate::TextureData::Empty,
            },
            options: crate::TextureOptions {
                size: Some(size),
                format,
                usage: usage_bits,
                ..Default::default()
            },
        };
        let tex = self
            .create_storage_texture(input)
            .await
            .map_err(FragmentColorError::from)?;
        Ok(Arc::new(tex))
    }

    /// Single mobile `render` entry. Mirrors the canonical Rust
    /// `Renderer::render(&impl Renderable, &impl Target)` — uniffi can't
    /// marshal `&impl Trait`, so the mobile shim takes the concrete enums
    /// `RenderableHandle` (Shader / Pass / Mesh / Passes) and `TargetHandle`
    /// (Window / Texture). Swift and Kotlin extension files supply natural
    /// overloads (`renderer.render(shader, target)`,
    /// `renderer.render(pass, target)`, etc.) that wrap the concrete value
    /// into the matching enum variant invisibly, so callers never see the
    /// mobile-only mirror types.
    #[uniffi::method(name = "render")]
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_mobile(
        self: Arc<Self>,
        renderable: crate::renderer::renderable::RenderableHandle,
        target: crate::renderer::renderable::TargetHandle,
    ) -> Result<(), FragmentColorError> {
        match target {
            crate::renderer::renderable::TargetHandle::Window(window_target) => self
                .render(&renderable, window_target.as_ref())
                .map_err(FragmentColorError::from),
            crate::renderer::renderable::TargetHandle::Texture(texture_target) => self
                .render(&renderable, texture_target.as_ref())
                .map_err(FragmentColorError::from),
        }
    }

    /// Create a depth texture (Depth32Float) at the given size.
    ///
    /// Uniffi cannot marshal `impl Into<Size>`, so width/height are taken as
    /// `u32` primitives and a `Size` is constructed internally.
    #[uniffi::method(name = "createDepthTexture")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_depth_texture_mobile.md")]
    pub async fn create_depth_texture_mobile(
        self: Arc<Self>,
        width: u32,
        height: u32,
    ) -> Result<Arc<crate::texture::Texture>, FragmentColorError> {
        let tex = self
            .create_depth_texture(crate::Size::new(width, height, None))
            .await
            .map_err(FragmentColorError::from)?;
        Ok(Arc::new(tex))
    }

    /// Read back the mip-0 contents of a registered texture as tightly-packed
    /// bytes in the texture's native format. Uniffi exposes this as a Swift
    /// `async throws` / Kotlin `suspend fun` automatically. Foreign callers
    /// await this in a coroutine or `Task`; the underlying GPU readback is
    /// driven by the async `read_texture_object_async` path.
    #[uniffi::method(name = "readTexture")]
    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub async fn read_texture_mobile(
        self: Arc<Self>,
        texture_id: u64,
    ) -> Result<Vec<u8>, FragmentColorError> {
        self.read_texture(crate::texture::TextureId { id: texture_id })
            .await
            .map_err(FragmentColorError::from)
    }

    /// Remove a texture from the renderer's registry by its raw numeric ID.
    ///
    /// TextureId wraps a `u64`; passing the raw value here avoids the need for
    /// a separate uniffi::Object binding for TextureId while the texture agent
    /// lands that binding on a parallel branch.
    #[uniffi::method(name = "unregisterTexture")]
    #[lsp_doc("docs/api/core/renderer/hidden/unregister_texture_mobile.md")]
    pub fn unregister_texture_mobile(
        self: Arc<Self>,
        texture_id: u64,
    ) -> Result<(), FragmentColorError> {
        self.unregister_texture(crate::texture::TextureId { id: texture_id })
            .map_err(FragmentColorError::from)
    }

    /// Block until all GPU submissions on this device have finished.
    ///
    /// Useful before readbacks (`read_texture`, `Texture.id`, `TextureTarget.getImage`)
    /// to ensure deterministic ordering. This is a genuine blocking call on
    /// native platforms; on web it is a no-op.
    #[uniffi::method(name = "waitIdle")]
    #[lsp_doc("docs/api/core/renderer/hidden/wait_idle_mobile.md")]
    pub fn wait_idle_mobile(self: Arc<Self>) -> Result<(), FragmentColorError> {
        self.wait_idle().map_err(FragmentColorError::from)
    }

    /// Wrap a native platform video-frame source as an external texture.
    /// The Web binding accepts an `HTMLVideoElement`; the mobile bindings
    /// take a raw pointer (`u64`) to a `CVPixelBuffer` (iOS) or
    /// `SurfaceTexture` (Android) because uniffi cannot marshal those
    /// types directly. Currently returns `FragmentColorError::Render`
    /// — the per-platform plumbing to convert the native source into a
    /// `wgpu::ExternalTexture` is a follow-up.
    #[uniffi::method(name = "createExternalTexture")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_external_texture_mobile.md")]
    pub fn create_external_texture_mobile(
        self: Arc<Self>,
        source_ptr: u64,
    ) -> Result<Arc<crate::renderer::external_texture::ExternalTextureHandle>, FragmentColorError>
    {
        crate::renderer::external_texture::create_external_texture_from_native(&self, source_ptr)
            .map_err(FragmentColorError::from)
    }
}

#[cfg(ios)]
pub mod ios;

#[cfg(android)]
pub mod android;
