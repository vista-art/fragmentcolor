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
//! and are wrapped by idiomatic Swift/Kotlin extension files on the binding
//! side. The hidden per-language docs under
//! `docs/api/core/{object}/hidden/<method>.md` satisfy the build-time
//! documentation validator without polluting the main website.

use std::sync::Arc;

use lsp_doc::lsp_doc;

use crate::{Renderer, Shader, Size, TextureTarget, WindowTarget};

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

#[uniffi::export]
impl Renderer {
    /// Mobile-only constructor. Swift/Kotlin extension files re-expose this
    /// as the natural `Renderer()` default initializer.
    #[uniffi::constructor]
    #[lsp_doc("docs/api/core/renderer/hidden/new_mobile.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Create a headless `TextureTarget` sized `width` × `height`. Uniffi-friendly
    /// concrete-typed variant of `create_texture_target`.
    #[lsp_doc("docs/api/core/renderer/hidden/create_texture_target_mobile.md")]
    pub async fn create_texture_target_mobile(
        self: Arc<Self>,
        width: u32,
        height: u32,
    ) -> Result<Arc<TextureTarget>, FragmentColorError> {
        let tex = self
            .create_texture_target(Size::new(width, height, None))
            .await
            .map_err(FragmentColorError::from)?;
        Ok(Arc::new(tex))
    }

    /// Mobile variant of `render` targeting a `WindowTarget`. Since uniffi
    /// cannot marshal `&impl Renderable` / `&impl Target`, mobile bindings
    /// get concrete method pairs per renderable × target combination. A
    /// Swift/Kotlin extension file recombines them into a single idiomatic
    /// `render(shader, target)` overload.
    #[lsp_doc("docs/api/core/renderer/hidden/render_shader_mobile.md")]
    pub fn render_shader_mobile(
        &self,
        shader: Arc<Shader>,
        target: Arc<WindowTarget>,
    ) -> Result<(), FragmentColorError> {
        self.render(shader.as_ref(), target.as_ref())
            .map_err(FragmentColorError::from)
    }

    /// Mobile variant of `render` targeting a `TextureTarget`.
    #[lsp_doc("docs/api/core/renderer/hidden/render_shader_texture_mobile.md")]
    pub fn render_shader_texture_mobile(
        &self,
        shader: Arc<Shader>,
        target: Arc<TextureTarget>,
    ) -> Result<(), FragmentColorError> {
        self.render(shader.as_ref(), target.as_ref())
            .map_err(FragmentColorError::from)
    }
}

#[cfg(ios)]
pub mod ios;

#[cfg(android)]
pub mod android;
