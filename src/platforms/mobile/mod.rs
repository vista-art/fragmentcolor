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

use std::sync::Arc;

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
    /// Create a new Renderer (Swift/Kotlin default constructor).
    #[uniffi::constructor]
    pub fn create() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Create a headless `TextureTarget` sized `width` × `height`.
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

    /// Render a Shader into a WindowTarget.
    pub fn render_shader_to_window(
        &self,
        shader: Arc<Shader>,
        target: Arc<WindowTarget>,
    ) -> Result<(), FragmentColorError> {
        self.render(shader.as_ref(), target.as_ref())
            .map_err(FragmentColorError::from)
    }

    /// Render a Shader into a TextureTarget.
    pub fn render_shader_to_texture(
        &self,
        shader: Arc<Shader>,
        target: Arc<TextureTarget>,
    ) -> Result<(), FragmentColorError> {
        self.render(shader.as_ref(), target.as_ref())
            .map_err(FragmentColorError::from)
    }
}

#[uniffi::export]
impl Shader {
    /// Build a Shader from source (WGSL or GLSL).
    #[uniffi::constructor]
    pub fn from_source(source: String) -> Result<Arc<Self>, FragmentColorError> {
        Shader::new(&source)
            .map(Arc::new)
            .map_err(FragmentColorError::from)
    }
}

#[cfg(ios)]
pub mod ios;

#[cfg(android)]
pub mod android;
