//! Mobile (Swift / Kotlin) uniffi bindings for `WindowTarget` and `TextureTarget`.
//!
//! Because uniffi `Object`s are always Arc-wrapped, methods cannot take
//! `&mut self` directly — all mutation must go through interior mutability.
//! This module follows the web `CanvasTarget` pattern: each concrete target
//! type is wrapped in a `parking_lot::Mutex` / `RwLock` so that Swift and
//! Kotlin callers can safely invoke `size()`, `resize()`, and `get_image()`
//! on a shared `Arc`.
//!
//! `WindowTarget` → `MobileWindowTarget`
//! `TextureTarget` → `MobileTextureTarget`
//!
//! The iOS and Android constructors return these wrapper types; `TargetHandle`
//! references them so that `Renderer::render_mobile` continues to work.

#![cfg(mobile)]

use std::sync::Arc;

use lsp_doc::lsp_doc;
use parking_lot::{Mutex, RwLock};

use crate::{Size, SurfaceError, Target, TargetFrame, TextureTarget, WindowTarget};

// ── WindowTarget wrapper ────────────────────────────────────────────────────

/// Mobile-facing wrapper around [`WindowTarget`].
///
/// Exposes `size()`, `resize(width, height)`, and `get_image()` to Swift and
/// Kotlin. `WindowTarget::get_image()` always returns an empty byte array —
/// window-backed surfaces are not readable; use a
/// [`MobileTextureTarget`] for pixel-readback.
#[derive(Debug, uniffi::Object)]
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

#[uniffi::export]
impl MobileWindowTarget {
    /// Returns the current size of the window surface in pixels.
    #[uniffi::method(name = "size")]
    #[lsp_doc("docs/api/targets/window_target/size.md")]
    pub fn size_mobile(&self) -> Size {
        Target::size(&*self.inner.lock())
    }

    /// Resizes the window surface to `width` × `height` pixels.
    ///
    /// Uniffi cannot express `&mut self` on an `Arc`-wrapped object, so this
    /// method acquires the inner mutex, applies the resize, and releases it.
    #[uniffi::method(name = "resize")]
    #[lsp_doc("docs/api/targets/window_target/resize.md")]
    pub fn resize_mobile(&self, width: u32, height: u32) {
        Target::resize(&mut *self.inner.lock(), [width, height]);
    }

    /// Returns an empty byte array.
    ///
    /// Window-backed surfaces are not readable across all GPU backends.
    /// Use a [`MobileTextureTarget`] and call `get_image()` there instead.
    #[uniffi::method(name = "getImage")]
    #[lsp_doc("docs/api/targets/window_target/get_image.md")]
    pub async fn get_image_mobile(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl crate::target::TargetInternal for MobileWindowTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, SurfaceError> {
        self.inner.lock().get_current_frame()
    }
}

// Expose the inner `Target` impl for use in `Renderer::render` dispatch.
impl Target for MobileWindowTarget {
    fn size(&self) -> Size {
        Target::size(&*self.inner.lock())
    }

    fn resize(&mut self, size: impl Into<Size>) {
        Target::resize(&mut *self.inner.lock(), size.into());
    }

    async fn get_image(&self) -> Vec<u8> {
        Vec::new()
    }
}

// ── TextureTarget wrapper ───────────────────────────────────────────────────

/// Mobile-facing wrapper around [`TextureTarget`].
///
/// Exposes `size()`, `resize(width, height)`, and `get_image()` to Swift and
/// Kotlin. `get_image()` returns a packed RGBA8 byte array of the offscreen
/// texture contents.
#[derive(Debug, uniffi::Object)]
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

#[uniffi::export]
impl MobileTextureTarget {
    /// Returns the current size of the offscreen texture in pixels.
    #[uniffi::method(name = "size")]
    #[lsp_doc("docs/api/targets/texture_target/size.md")]
    pub fn size_mobile(&self) -> Size {
        Target::size(&*self.inner.read())
    }

    /// Resizes the offscreen texture to `width` × `height` pixels.
    ///
    /// The underlying GPU texture is recreated; any previously-rendered
    /// pixel data is lost. Uniffi cannot express `&mut self` on an
    /// `Arc`-wrapped object, so this method acquires the inner write-lock.
    #[uniffi::method(name = "resize")]
    #[lsp_doc("docs/api/targets/texture_target/resize.md")]
    pub fn resize_mobile(&self, width: u32, height: u32) {
        Target::resize(&mut *self.inner.write(), [width, height]);
    }

    /// Returns the current offscreen texture contents as a packed RGBA8
    /// byte array (row-major, top-left origin). Uniffi exposes this as a
    /// Swift / Kotlin `suspend fun`; the underlying readback runs through
    /// `TextureTarget::get_image` async.
    #[uniffi::method(name = "getImage")]
    #[lsp_doc("docs/api/targets/texture_target/get_image.md")]
    pub async fn get_image_mobile(&self) -> Vec<u8> {
        // Inner is a TextureTarget; clone to release the read-lock before await.
        let inner = self.inner.read().clone();
        inner.get_image().await
    }
}

impl crate::target::TargetInternal for MobileTextureTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, SurfaceError> {
        self.inner.read().get_current_frame()
    }
}

// Expose the inner `Target` impl for use in `Renderer::render` dispatch.
impl Target for MobileTextureTarget {
    fn size(&self) -> Size {
        Target::size(&*self.inner.read())
    }

    fn resize(&mut self, size: impl Into<Size>) {
        Target::resize(&mut *self.inner.write(), size.into());
    }

    async fn get_image(&self) -> Vec<u8> {
        // Snapshot the inner TextureTarget so we can release the read-lock
        // before awaiting the GPU readback.
        let inner = self.inner.read().clone();
        inner.get_image().await
    }
}
