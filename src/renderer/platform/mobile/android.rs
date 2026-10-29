//! Android-specific FFI bridge.
//!
//! `Renderer.createTarget(surface: Surface)` takes two steps, because uniffi
//! cannot marshal the `JNIEnv*` that `ANativeWindow_fromSurface` needs:
//!
//! 1. **Raw JNI** (`native_window_from_surface`): turns the `Surface` into an
//!    acquired `ANativeWindow*`, returned as a `jlong`.
//! 2. **Uniffi method** (`Renderer.createTarget(nativeWindowPtr:)`): builds the
//!    `WindowTarget` on the calling `Renderer`, so the surface shares its device.
//!
//! The Kotlin extension in `RendererExtensions.kt` chains the two and then
//! drops its window reference with `release_native_window`.
//!
//! TODO(0.11.x): migrate `jni::JNIEnv` → `jni::EnvUnowned` (FFI-safe entry
//! points) and `jni::Env` (full API). The deprecation lives behind
//! `#[allow(deprecated)]` here until the rewrite. See jni 0.22 release notes.

#![allow(deprecated)]

use std::sync::Arc;

use jni::{
    JNIEnv,
    objects::JClass,
    sys::{jlong, jobject},
};
use jni_fn::jni_fn;
use lsp_doc::lsp_doc;
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};

use super::FragmentColorError;
use crate::MobileWindowTarget;
use crate::renderer::error::InitializationError;
use crate::renderer::platform::native_window;
use crate::{Renderer, WindowTarget};

/// HasWindowHandle + HasDisplayHandle wrapper over an `ANativeWindow`.
#[derive(Debug)]
struct AndroidNativeWindow {
    window: *mut ndk_sys::ANativeWindow,
}

unsafe impl Send for AndroidNativeWindow {}
unsafe impl Sync for AndroidNativeWindow {}

impl AndroidNativeWindow {
    /// Acquire the window behind `surface`. Null when the surface has none.
    fn acquire(env: *mut JNIEnv, surface: jobject) -> *mut ndk_sys::ANativeWindow {
        // Cast surface: jni 0.22 uses jni-sys 0.4's _jobject; ndk-sys 0.6 expects
        // jni-sys 0.3's _jobject. They're both pointer-sized opaque types, so
        // a raw cast through *mut _ is safe.
        unsafe { ndk_sys::ANativeWindow_fromSurface(env as *mut _, surface as *mut _) }
    }

    /// The window's width and height, each at least 1, or the error for a side Android
    /// reports below 0, which `ANativeWindow_getWidth` and `ANativeWindow_getHeight` return on failure.
    fn read_size(&self) -> Result<[u32; 2], InitializationError> {
        let width = unsafe { ndk_sys::ANativeWindow_getWidth(self.window) };
        let height = unsafe { ndk_sys::ANativeWindow_getHeight(self.window) };
        native_window::read_size(width, height)
    }
}

impl Drop for AndroidNativeWindow {
    fn drop(&mut self) {
        unsafe { ndk_sys::ANativeWindow_release(self.window) }
    }
}

impl HasWindowHandle for AndroidNativeWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let nn = std::ptr::NonNull::new(self.window as *mut _).ok_or(HandleError::Unavailable)?;
        let handle = AndroidNdkWindowHandle::new(nn);
        // SAFETY: NativeWindow is alive for the duration of `&self`.
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(handle)) })
    }
}

impl HasDisplayHandle for AndroidNativeWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // SAFETY: AndroidDisplayHandle carries no pointer; always valid.
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Android(AndroidDisplayHandle::new()))
        })
    }
}

/// Acquire the `ANativeWindow` behind an Android `Surface`.
///
/// Returns the pointer as a `jlong`, or 0 when the surface has no window.
/// The caller owns one reference and must pass it to `release_native_window`.
#[unsafe(no_mangle)]
#[jni_fn("org.fragmentcolor.SurfaceBridge")]
pub fn native_window_from_surface(env: *mut JNIEnv, _class: JClass, surface: jobject) -> jlong {
    AndroidNativeWindow::acquire(env, surface) as jlong
}

/// Release a window reference from `native_window_from_surface`. Ignores 0.
#[unsafe(no_mangle)]
#[jni_fn("org.fragmentcolor.SurfaceBridge")]
pub fn release_native_window(_env: *mut JNIEnv, _class: JClass, window: jlong) {
    let window = window as *mut ndk_sys::ANativeWindow;
    if !window.is_null() {
        unsafe { ndk_sys::ANativeWindow_release(window) }
    }
}

#[uniffi::export]
impl Renderer {
    /// Create a `WindowTarget` from a pre-acquired `ANativeWindow` pointer.
    ///
    /// The target takes its own reference to the window. The
    /// pointer only has to stay valid for the duration of this call.
    /// `Renderer.createTarget(surface: Surface)` in the Kotlin extensions
    /// acquires the pointer over JNI and calls this on the same `Renderer`.
    ///
    /// Exposed as synchronous because `ANativeWindow*` holds a raw pointer
    /// that is not `Send`, so the resulting future cannot satisfy uniffi's
    /// `Send` bound on async exports. Adapter/device creation is driven by
    /// pollster internally.
    #[uniffi::method(name = "createTarget")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_target_android.md")]
    pub fn create_target_android(
        self: Arc<Self>,
        native_window_ptr: u64,
    ) -> Result<Arc<MobileWindowTarget>, FragmentColorError> {
        let raw = native_window_ptr as *mut ndk_sys::ANativeWindow;
        if raw.is_null() {
            return Err(FragmentColorError::Init(
                "createTarget: the ANativeWindow pointer is null".into(),
            ));
        }
        // SAFETY: the caller guarantees the ANativeWindow is alive and valid
        // for the duration of this call. We immediately retain it via
        // ANativeWindow_acquire and release in AndroidNativeWindow::drop.
        unsafe { ndk_sys::ANativeWindow_acquire(raw) };
        let window = AndroidNativeWindow { window: raw };
        let [width, height] = window.read_size()?;
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        // The instance has no display handle, so wgpu takes it from the window.
        let (context, surface, config) = pollster::block_on(self.create_surface(window, size))
            .map_err(FragmentColorError::from)?;
        Ok(MobileWindowTarget::new(WindowTarget::new(
            context, surface, config,
        )))
    }
}
