use std::sync::Arc;

use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};

use jni::{objects::JClass, sys::jobject, JNIEnv};
use jni_fn::jni_fn;

use crate::Quad;

use crate::{ffi, Bitmap, Destination, Image, PixelFormat};

const BACKENDS: wgpu::Backends = wgpu::Backends::VULKAN;

/// An implementation of HasWindowHandle + HasDisplayHandle for Android.
#[derive(Debug)]
struct AndroidNativeWindow {
    android_window: *mut ndk_sys::ANativeWindow,
}

unsafe impl Send for AndroidNativeWindow {}
unsafe impl Sync for AndroidNativeWindow {}

impl AndroidNativeWindow {
    fn new(env: *mut JNIEnv, surface: jobject) -> Self {
        let android_window = unsafe {
            // Get the ANativeWindow associated with the Android Surface object
            // so that it can be used by Rust.
            //
            // This function will automatically increase its reference count by 1
            // when returning ANativeWindow to prevent the object from being
            // accidentally released on the Android side.
            ndk_sys::ANativeWindow_fromSurface(env as *mut _, surface)
        };

        Self { android_window }
    }

    fn width(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getWidth(self.android_window) as u32 }
    }

    fn height(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getHeight(self.android_window) as u32 }
    }
}

impl Drop for AndroidNativeWindow {
    fn drop(&mut self) {
        unsafe {
            ndk_sys::ANativeWindow_release(self.android_window);
        }
    }
}

impl HasWindowHandle for AndroidNativeWindow {
    fn window_handle(&self) -> Result<WindowHandle, HandleError> {
        unsafe {
            let handle = AndroidNdkWindowHandle::new(
                std::ptr::NonNull::new(self.android_window as *mut _).unwrap(),
            );
            Ok(WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(
                handle,
            )))
        }
    }
}

impl HasDisplayHandle for AndroidNativeWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe {
            Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Android(
                AndroidDisplayHandle::new(),
            )))
        }
    }
}
