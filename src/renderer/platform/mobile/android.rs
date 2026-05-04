//! Android-specific FFI bridge.
//!
//! Two paths for creating a `WindowTarget` from an Android `Surface`:
//!
//! 1. **Raw JNI** (`create_window_target_from_surface`): accepts `JNIEnv*` +
//!    `jobject` directly; uniffi cannot marshal those, so the entry point is
//!    exposed as a `#[jni_fn]` function that returns an `Arc<WindowTarget>`
//!    pointer. The Kotlin side reconstructs the `WindowTarget` via
//!    `WindowTarget(Pointer(ptr))` (the JNA pointer-constructor uniffi emits).
//!
//! 2. **Uniffi method** (`Renderer.createTarget(nativeWindowPtr:)`): accepts a
//!    pre-acquired `ANativeWindow*` pointer cast to `u64`. The Kotlin extension
//!    obtains this pointer via `android.view.Surface.acquireNativeHandle()` or
//!    by calling into the NDK directly, then passes it to the uniffi method.
//!    This path satisfies the parity audit's `create_target.md:kotlin` gap and
//!    gives Kotlin callers a first-class uniffi-generated API.

use std::sync::Arc;

use jni::{JNIEnv, objects::JClass, sys::jobject};
use jni_fn::jni_fn;
use lsp_doc::lsp_doc;
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};

use super::FragmentColorError;
use crate::MobileWindowTarget;
use crate::{Renderer, WindowTarget};

/// HasWindowHandle + HasDisplayHandle wrapper over an `ANativeWindow`.
#[derive(Debug)]
struct AndroidNativeWindow {
    window: *mut ndk_sys::ANativeWindow,
}

unsafe impl Send for AndroidNativeWindow {}
unsafe impl Sync for AndroidNativeWindow {}

impl AndroidNativeWindow {
    fn from_surface(env: *mut JNIEnv, surface: jobject) -> Self {
        // ANativeWindow_fromSurface retains the surface; Drop releases it.
        // Cast surface: jni 0.22 uses jni-sys 0.4's _jobject; ndk-sys 0.6 expects
        // jni-sys 0.3's _jobject. They're both pointer-sized opaque types, so
        // a raw cast through *mut _ is safe.
        let window =
            unsafe { ndk_sys::ANativeWindow_fromSurface(env as *mut _, surface as *mut _) };
        Self { window }
    }

    fn width(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getWidth(self.window) as u32 }
    }

    fn height(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getHeight(self.window) as u32 }
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

async fn build_window_target(
    env: *mut JNIEnv<'_>,
    surface: jobject,
) -> Option<Arc<MobileWindowTarget>> {
    let window = AndroidNativeWindow::from_surface(env, surface);
    let size = wgpu::Extent3d {
        width: u32::max(window.width(), 1),
        height: u32::max(window.height(), 1),
        depth_or_array_layers: 1,
    };

    let renderer = Renderer::new();
    let handle: Box<dyn wgpu::WindowHandle> = Box::new(window);
    let (context, surface, config) = renderer
        .create_surface(wgpu::SurfaceTarget::Window(handle), size)
        .await
        .ok()?;
    Some(MobileWindowTarget::new(WindowTarget::new(
        context, surface, config,
    )))
}

/// Raw JNI entry point. Returns `Arc::into_raw(target)` as an opaque pointer
/// (cast to `*const MobileWindowTarget` for jlong compatibility). Caller owns
/// the resulting Arc — the uniffi-generated Kotlin bindings take ownership on
/// `MobileWindowTarget(Pointer(ptr))`.
///
/// Returns `std::ptr::null()` on failure.
#[unsafe(no_mangle)]
#[jni_fn("org.fragmentcolor.RendererJni")]
pub fn create_window_target_from_surface(
    env: *mut JNIEnv,
    _class: JClass,
    surface: jobject,
) -> *const MobileWindowTarget {
    match pollster::block_on(build_window_target(env, surface)) {
        Some(target) => Arc::into_raw(target),
        None => std::ptr::null(),
    }
}

#[uniffi::export]
impl Renderer {
    /// Create a `WindowTarget` from a pre-acquired `ANativeWindow` pointer.
    ///
    /// The Kotlin caller obtains the pointer via
    /// `android.view.Surface.acquireNativeHandle()` or the NDK's
    /// `ANativeWindow_fromSurface` helper, then casts it to `Long` before
    /// passing it here. A Kotlin extension file re-exports this as
    /// `Renderer.createTarget(surface: Surface)` so the public API reads
    /// the same as every other platform.
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
        // SAFETY: the caller guarantees the ANativeWindow is alive and valid
        // for the duration of this call. We immediately retain it via
        // ANativeWindow_acquire and release in AndroidNativeWindow::drop.
        unsafe { ndk_sys::ANativeWindow_acquire(raw) };
        let window = AndroidNativeWindow { window: raw };
        let size = wgpu::Extent3d {
            width: u32::max(window.width(), 1),
            height: u32::max(window.height(), 1),
            depth_or_array_layers: 1,
        };
        let handle: Box<dyn wgpu::WindowHandle> = Box::new(window);
        let (context, surface, config) =
            pollster::block_on(self.create_surface(wgpu::SurfaceTarget::Window(handle), size))
                .map_err(FragmentColorError::from)?;
        Ok(MobileWindowTarget::new(WindowTarget::new(
            context, surface, config,
        )))
    }
}
