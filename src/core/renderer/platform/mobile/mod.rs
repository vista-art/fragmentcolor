//! Uniffi bindings shared by iOS (Swift) and Android (Kotlin).
//!
//! Core types (`Renderer`, `Shader`, `WindowTarget`, `TextureTarget`) are
//! already annotated `#[cfg_attr(mobile, derive(uniffi::Object))]` where they
//! live. This module adds the mobile-only constructor and helper methods.
//!
//! Android's `Surface` → `WindowTarget` constructor cannot go through uniffi
//! alone (uniffi cannot marshal `JNIEnv*`). The `android` submodule exposes a
//! raw `#[jni_fn]` entry point that returns the surface's `ANativeWindow*`;
//! Kotlin hands that pointer to the uniffi `createTarget` on the same `Renderer`.
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
use crate::error::platform::mobile::FragmentColorError;
use crate::{Renderer, Size};

#[uniffi::export]
impl Renderer {
    /// uniffi makes a constructor named `new` the primary constructor,
    /// so Swift and Kotlin both call `Renderer()`.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/core/renderer/hidden/new_mobile.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Set the process log level. uniffi has no static methods, so this constructor
    /// sets it and returns a new `Renderer`, and the Swift and Kotlin extensions
    /// spell it `Renderer.setLogLevel(level)` with no return value.
    #[uniffi::constructor(name = "useLogLevel")]
    #[lsp_doc("docs/api/core/renderer/hidden/use_log_level_mobile.md")]
    pub fn use_log_level_mobile(level: crate::LogLevel) -> Arc<Self> {
        Self::set_log_level(level);
        Arc::new(Self::new())
    }

    /// Create a headless `TextureTarget` of `size`. Uniffi can't marshal
    /// `impl Into<Size>`, so the mobile entry point takes the `Size` record,
    /// layers included, and checks it the way every binding does.
    #[uniffi::method(name = "createTextureTarget")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_texture_target_mobile.md")]
    pub async fn create_texture_target_mobile(
        self: Arc<Self>,
        size: Size,
    ) -> Result<Arc<MobileTextureTarget>, FragmentColorError> {
        let tex = self.create_texture_target(size).await?;
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
        let tex = self.create_texture(spec).await?;
        Ok(Arc::new(tex))
    }

    /// Allocate a storage-class texture, optionally pre-seeded with `data`.
    /// Mirrors the canonical `Renderer::create_storage_texture(impl Into<TextureInput>)`
    /// — uniffi can't marshal `impl Into`, so the mobile shim takes the
    /// fields directly and the body builds a `TextureInput` from them.
    /// `usage_bits = None` defaults to STORAGE | TEXTURE | COPY_SRC | COPY_DST.
    /// Both trailing arguments default to `None`, so Swift and Kotlin may leave them out.
    #[uniffi::method(name = "createStorageTexture", default(data = None, usage_bits = None))]
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
        let tex = self.create_storage_texture(input).await?;
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
    #[uniffi::method(name = "load")]
    #[lsp_doc("docs/api/core/renderer/load.md")]
    pub async fn load_mobile(
        self: Arc<Self>,
        renderable: crate::RenderableHandle,
    ) -> Result<(), FragmentColorError> {
        Ok(self.load(&renderable).await?)
    }

    #[uniffi::method(name = "readStorage")]
    #[lsp_doc("docs/api/core/renderer/read_storage.md")]
    pub async fn read_storage_mobile(
        self: Arc<Self>,
        shader: Arc<crate::Shader>,
        binding: String,
    ) -> Result<Vec<u8>, FragmentColorError> {
        Ok(self.read_storage(&shader, &binding).await?)
    }

    #[uniffi::method(name = "render")]
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_mobile(
        self: Arc<Self>,
        renderable: crate::RenderableHandle,
        target: crate::TargetHandle,
    ) -> Result<(), FragmentColorError> {
        match target {
            crate::TargetHandle::Window(window_target) => {
                Ok(self.render(&renderable, window_target.as_ref())?)
            }
            crate::TargetHandle::Texture(texture_target) => {
                Ok(self.render(&renderable, texture_target.as_ref())?)
            }
        }
    }

    /// Create a depth texture (Depth32Float) of `size`. Uniffi can't marshal
    /// `impl Into<Size>`, so the mobile entry point takes the `Size` record,
    /// layers included, and checks it the way every binding does.
    #[uniffi::method(name = "createDepthTexture")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_depth_texture_mobile.md")]
    pub async fn create_depth_texture_mobile(
        self: Arc<Self>,
        size: Size,
    ) -> Result<Arc<crate::texture::Texture>, FragmentColorError> {
        let tex = self.create_depth_texture(size).await?;
        Ok(Arc::new(tex))
    }

    /// Read back the mip-0 contents of a registered texture as tightly-packed
    /// bytes in the texture's native format. Uniffi exposes this as a Swift
    /// `async throws` / Kotlin `suspend fun` automatically. Foreign callers
    /// await this in a coroutine or `Task`. It runs `Renderer::read_texture`,
    /// whose GPU readback goes through `texture::read::copy_pixels`.
    #[uniffi::method(name = "readTexture")]
    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub async fn read_texture_mobile(
        self: Arc<Self>,
        texture_id: u64,
    ) -> Result<Vec<u8>, FragmentColorError> {
        Ok(self
            .read_texture(crate::texture::TextureId { id: texture_id })
            .await?)
    }

    /// Remove a texture from the renderer's registry by its raw numeric ID.
    /// The Swift and Kotlin extensions take the `TextureId` record and pass
    /// its `id` here.
    #[uniffi::method(name = "unregisterTexture")]
    #[lsp_doc("docs/api/core/renderer/hidden/unregister_texture_mobile.md")]
    pub fn unregister_texture_mobile(
        self: Arc<Self>,
        texture_id: u64,
    ) -> Result<(), FragmentColorError> {
        Ok(self.unregister_texture(crate::texture::TextureId { id: texture_id })?)
    }
}

#[cfg(ios)]
pub mod ios;

#[cfg(android)]
pub mod android;
