//! Mobile (Swift / Kotlin) uniffi bindings for `WindowTarget` and `TextureTarget`.
//!
//! Because uniffi `Object`s are always Arc-wrapped, methods cannot take
//! `&mut self` directly — all mutation must go through interior mutability.
//! This module follows the web `CanvasTarget` pattern: each concrete target
//! type is wrapped in a `parking_lot::Mutex` / `RwLock` so that Swift and
//! Kotlin callers can safely invoke `size()`, `resize()`, and `get_image()`
//! on a shared `Arc`.
//!
//! Swift and Kotlin see each wrapper under the name of the type it wraps:
//! `MobileWindowTarget` exports as `WindowTarget` and `MobileTextureTarget`
//! as `TextureTarget`. The extension files keep the old names as typealiases.
//!
//! The iOS and Android constructors return these wrapper types; `TargetHandle`
//! references them so that `Renderer::render_mobile` continues to work.

#![cfg(mobile)]

use std::sync::Arc;

use lsp_doc::lsp_doc;
use parking_lot::{Mutex, RwLock};

use crate::core::renderer::platform::mobile::FragmentColorError;
use crate::texture::TextureError;
use crate::{RenderContext, Size, SurfaceError, Target, TargetFrame, TextureTarget, WindowTarget};

// ── WindowTarget wrapper ────────────────────────────────────────────────────

/// Mobile-facing wrapper around [`WindowTarget`].
///
/// Exposes `size()`, `resize(width, height)`, and `get_image()` to Swift and
/// Kotlin. `get_image()` throws, because a window surface can't be read back.
/// Render to a `TextureTarget` to read pixels.
#[derive(Debug, uniffi::Object)]
#[uniffi(name = "WindowTarget")]
pub struct MobileWindowTarget {
    inner: Mutex<WindowTarget>,
}

impl MobileWindowTarget {
    /// Construct from an already-configured [`WindowTarget`].
    pub fn new(target: WindowTarget) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(target),
        })
    }
}

#[uniffi::export(name = "WindowTarget")]
impl MobileWindowTarget {
    /// Returns the current size of the window surface in pixels.
    #[uniffi::method(name = "size")]
    #[lsp_doc("docs/api/targets/window_target/size.md")]
    pub fn size_mobile(&self) -> Size {
        Target::size(&*self.inner.lock())
    }

    /// Resizes the window surface to `size`. A width or height of zero, such as
    /// a minimized window's, keeps the last size. Throws for a side past the device limit.
    ///
    /// Uniffi cannot express `&mut self` on an `Arc`-wrapped object, so this
    /// method acquires the inner mutex, applies the resize, and releases it.
    #[uniffi::method(name = "resize")]
    #[lsp_doc("docs/api/targets/window_target/resize.md")]
    pub fn resize_mobile(&self, size: Size) -> Result<(), FragmentColorError> {
        Ok(Target::resize(&mut *self.inner.lock(), size)?)
    }

    #[uniffi::method(name = "getImage")]
    #[lsp_doc("docs/api/targets/window_target/get_image.md")]
    pub async fn get_image_mobile(&self) -> Result<Vec<u8>, FragmentColorError> {
        Err(crate::targets::window_target::unreadable().into())
    }
}

impl crate::targets::FrameSource for MobileWindowTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, SurfaceError> {
        self.inner.lock().get_current_frame()
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        self.inner.lock().owner()
    }
}

// Expose the inner `Target` impl for use in `Renderer::render` dispatch.
impl Target for MobileWindowTarget {
    fn size(&self) -> Size {
        Target::size(&*self.inner.lock())
    }

    fn resize(&mut self, size: impl Into<Size>) -> Result<(), TextureError> {
        Target::resize(&mut *self.inner.lock(), size.into())
    }

    async fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        Err(crate::targets::window_target::unreadable())
    }
}

// ── TextureTarget wrapper ───────────────────────────────────────────────────

/// Mobile-facing wrapper around [`TextureTarget`].
///
/// Exposes `size()`, `resize(width, height)`, and `get_image()` to Swift and
/// Kotlin. `get_image()` returns a packed RGBA8 byte array of the offscreen
/// texture contents.
#[derive(Debug, uniffi::Object)]
#[uniffi(name = "TextureTarget")]
pub struct MobileTextureTarget {
    inner: RwLock<TextureTarget>,
}

impl MobileTextureTarget {
    /// Construct from an already-configured [`TextureTarget`].
    pub fn new(target: TextureTarget) -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(target),
        })
    }

    /// Return a clone of the underlying `TextureTarget`.
    ///
    /// Used by `Pass::set_target` and `Pass::set_depth_target` which need a
    /// typed `&TextureTarget` to dispatch through the `TryInto<ColorTarget>`
    /// / `TryInto<DepthTarget>` conversions.
    pub fn texture_target(&self) -> TextureTarget {
        self.inner.read().clone()
    }
}

#[uniffi::export(name = "TextureTarget")]
impl MobileTextureTarget {
    /// Returns the current size of the offscreen texture in pixels.
    #[uniffi::method(name = "size")]
    #[lsp_doc("docs/api/targets/texture_target/size.md")]
    pub fn size_mobile(&self) -> Size {
        Target::size(&*self.inner.read())
    }

    /// Resizes the offscreen texture to `size`. A new size starts with blank pixels,
    /// and the current size keeps them. Throws when the target can't take the size,
    /// such as a width or height of zero.
    ///
    /// Uniffi cannot express `&mut self` on an `Arc`-wrapped object, so this
    /// method acquires the inner write-lock.
    #[uniffi::method(name = "resize")]
    #[lsp_doc("docs/api/targets/texture_target/resize.md")]
    pub fn resize_mobile(&self, size: Size) -> Result<(), FragmentColorError> {
        Ok(Target::resize(&mut *self.inner.write(), size)?)
    }

    /// Returns the current offscreen texture contents as a packed RGBA8
    /// byte array (row-major, top-left origin). Uniffi exposes this as a
    /// Swift / Kotlin `suspend fun`; the underlying readback runs through
    /// `TextureTarget::get_image` async.
    #[uniffi::method(name = "getImage")]
    #[lsp_doc("docs/api/targets/texture_target/get_image.md")]
    pub async fn get_image_mobile(&self) -> Result<Vec<u8>, FragmentColorError> {
        // Inner is a TextureTarget; clone to release the read-lock before await.
        let inner = self.inner.read().clone();
        Ok(inner.get_image().await?)
    }

    /// Return a sampleable [`Texture`](crate::texture::Texture) handle that
    /// aliases this target's offscreen storage. Mirrors the inherent
    /// `TextureTarget::texture()` on every other binding so the docs example
    /// reads `target.texture()` identically across Python, Swift, Kotlin, JS.
    #[uniffi::method(name = "texture")]
    #[lsp_doc("docs/api/targets/texture_target/texture.md")]
    pub fn texture_mobile(&self) -> Arc<crate::texture::Texture> {
        Arc::new(self.inner.read().texture())
    }
}

impl crate::targets::FrameSource for MobileTextureTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, SurfaceError> {
        self.inner.read().get_current_frame()
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        self.inner.read().owner()
    }
}

// Expose the inner `Target` impl for use in `Renderer::render` dispatch.
impl Target for MobileTextureTarget {
    fn size(&self) -> Size {
        Target::size(&*self.inner.read())
    }

    fn resize(&mut self, size: impl Into<Size>) -> Result<(), TextureError> {
        Target::resize(&mut *self.inner.write(), size.into())
    }

    async fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        // Snapshot the inner TextureTarget so we can release the read-lock
        // before awaiting the GPU readback.
        let inner = self.inner.read().clone();
        inner.get_image().await
    }
}
