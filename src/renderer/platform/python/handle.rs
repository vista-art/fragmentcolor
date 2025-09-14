use crate::target::error::DisplayError;
use wgpu::rwh::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

#[derive(Clone, Copy)]
pub struct WindowHandles<'window> {
    pub window_handle: WindowHandle<'window>,
    pub display_handle: DisplayHandle<'window>,
}

unsafe impl<'window> Send for WindowHandles<'window> {}
unsafe impl<'window> Sync for WindowHandles<'window> {}

impl<'window> HasWindowHandle for WindowHandles<'window> {
    fn window_handle(&self) -> Result<WindowHandle<'window>, HandleError> {
        Ok(self.window_handle)
    }
}

impl<'window> HasDisplayHandle for WindowHandles<'window> {
    fn display_handle(&self) -> Result<DisplayHandle<'window>, HandleError> {
        Ok(self.display_handle)
    }
}

pub(crate) fn create_raw_handles<'window>(
    platform: &str,
    window: u64,
    display: Option<u64>,
) -> Result<WindowHandles<'window>, DisplayError> {
    #[cfg(target_os = "linux")]
    let result = match platform {
        "wayland" => {
            use std::ffi::c_void;
            use std::ptr::NonNull;
            use wgpu::rwh::{
                RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
            };

            let window_ptr = {
                let ptr = window as *mut c_void;
                NonNull::new(ptr).ok_or(DisplayError::WindowHandleError(
                    "Could not convert u64 to c_void for Wayland window".to_string(),
                ))?
            };

            let display_ptr = {
                let ptr = display.ok_or(DisplayError::DisplayHandleError(
                    "Display handle is missing for Wayland".to_string(),
                ))? as *mut c_void;
                NonNull::new(ptr).ok_or(DisplayError::DisplayHandleError(
                    "Could not convert u64 to c_void for Wayland display".to_string(),
                ))?
            };

            let wayland_window_handle =
                RawWindowHandle::Wayland(WaylandWindowHandle::new(window_ptr));
            let wayland_display_handle =
                RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display_ptr));

            let window_handle = unsafe { WindowHandle::borrow_raw(wayland_window_handle) };
            let display_handle = unsafe { DisplayHandle::borrow_raw(wayland_display_handle) };

            Ok(WindowHandles {
                window_handle,
                display_handle,
            })
        }
        // assumes X11 if not Wayland
        _ => {
            use std::ffi::{c_ulong, c_void};
            use std::ptr::NonNull;
            use wgpu::rwh::{
                RawDisplayHandle, RawWindowHandle, XlibDisplayHandle, XlibWindowHandle,
            };

            let display_ptr = if let Some(display) = display {
                let ptr = display as *mut c_void;
                Some(NonNull::new(ptr).ok_or(DisplayError::DisplayHandleError(
                    "Could not convert u64 to c_void for Xlib display".to_string(),
                ))?)
            } else {
                None
            };

            let window = window as c_ulong;

            let xlib_window_handle = RawWindowHandle::Xlib(XlibWindowHandle::new(window));
            let xlib_display_handle =
                RawDisplayHandle::Xlib(XlibDisplayHandle::new(display_ptr, 0));

            let window_handle = unsafe { WindowHandle::borrow_raw(xlib_window_handle) };
            let display_handle = unsafe { DisplayHandle::borrow_raw(xlib_display_handle) };

            Ok(WindowHandles {
                window_handle,
                display_handle,
            })
        }
    };

    #[cfg(target_os = "windows")]
    let result = {
        use std::num::NonZeroIsize;
        use wgpu::rwh::{
            RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
        };

        let _ = display;
        let window_ptr = {
            NonZeroIsize::new(window as isize).ok_or(DisplayError::WindowHandleError(
                "Could not convert u64 to isize for Win32 window".to_string(),
            ))?
        };

        let win32_window_handle = RawWindowHandle::Win32(Win32WindowHandle::new(window_ptr));
        let win32_display_handle = RawDisplayHandle::Windows(WindowsDisplayHandle::new());

        let window_handle = unsafe { WindowHandle::borrow_raw(win32_window_handle) };
        let display_handle = unsafe { DisplayHandle::borrow_raw(win32_display_handle) };

        Ok(WindowHandles {
            window_handle,
            display_handle,
        })
    };

    #[cfg(target_os = "macos")]
    let result = {
        use objc2::msg_send;
        use objc2_app_kit::{NSView, NSWindow};
        use std::ffi::c_void;
        use std::ptr::NonNull;
        use wgpu::rwh::{
            AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
        };

        let _ = display;
        let _ = platform;
        let ns_window = window as *mut NSWindow;
        let ns_view_ptr: *mut NSView = unsafe { msg_send![ns_window, contentView] };
        let ns_view = NonNull::new(ns_view_ptr as *mut c_void).ok_or(
            DisplayError::WindowHandleError("Could not convert *mut NSView to c_void".to_string()),
        )?;

        let appkit_window_handle = RawWindowHandle::AppKit(AppKitWindowHandle::new(ns_view));
        let appkit_display_handle = RawDisplayHandle::AppKit(AppKitDisplayHandle::new());

        let window_handle = unsafe { WindowHandle::borrow_raw(appkit_window_handle) };
        let display_handle = unsafe { DisplayHandle::borrow_raw(appkit_display_handle) };

        Ok(WindowHandles {
            window_handle,
            display_handle,
        })
    };

    #[cfg(wasm)]
    let result = {
        use wgpu::rwh::{RawDisplayHandle, RawWindowHandle, WebDisplayHandle, WebWindowHandle};

        let _ = display;
        let _ = platform;
        let web_window_handle = RawWindowHandle::Web(WebWindowHandle::new(window as u32));
        let web_display_handle = RawDisplayHandle::Web(WebDisplayHandle::new());

        let window_handle = unsafe { WindowHandle::borrow_raw(web_window_handle) };
        let display_handle = unsafe { DisplayHandle::borrow_raw(web_display_handle) };

        Ok(WindowHandles {
            window_handle,
            display_handle,
        })
    };

    #[cfg(ios)]
    let result = {
        Err(DisplayError::UnsupportedPlatform(
            "iOS platform is not yet supported".to_string(),
        ))
    };

    #[cfg(android)]
    let result = {
        Err(DisplayError::UnsupportedPlatform(
            "Android platform is not yet supported".to_string(),
        ))
    };

    result
}
