use crate::FragmentColorError;
use crate::renderer::WindowHandles;
use pyo3::prelude::*;
use wgpu::rwh::{DisplayHandle, WindowHandle};

pub(crate) fn create_raw_handles<'window>(
    platform: String,
    window: u64,
    display: Option<u64>,
) -> Result<WindowHandles<'window>, PyErr> {
    match platform.as_str() {
        #[cfg(target_os = "linux")]
        "x11" => {
            use std::ffi::{c_ulong, c_void};
            use std::ptr::NonNull;
            use wgpu::rwh::{
                RawDisplayHandle, RawWindowHandle, XlibDisplayHandle, XlibWindowHandle,
            };

            let display_ptr = {
                let ptr = display.ok_or(FragmentColorError::new_err(
                    "Display handle is missing for Xlib",
                ))? as *mut c_void;
                NonNull::new(ptr).ok_or(FragmentColorError::new_err(
                    "Could not convert u64 to c_void for Xlib display",
                ))?
            };

            let window = window as c_ulong;

            let xlib_window_handle = RawWindowHandle::Xlib(XlibWindowHandle::new(window));
            let xlib_display_handle =
                RawDisplayHandle::Xlib(XlibDisplayHandle::new(Some(display_ptr), 0));

            let window_handle = unsafe { WindowHandle::borrow_raw(xlib_window_handle) };
            let display_handle = unsafe { DisplayHandle::borrow_raw(xlib_display_handle) };

            Ok(WindowHandles {
                window_handle,
                display_handle,
            })
        }

        #[cfg(target_os = "linux")]
        "wayland" => {
            use std::ffi::c_void;
            use std::ptr::NonNull;
            use wgpu::rwh::{
                RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
            };

            let window_ptr = {
                let ptr = window as *mut c_void;
                NonNull::new(ptr).ok_or(FragmentColorError::new_err(
                    "Could not convert u64 to c_void for Wayland window",
                ))?
            };

            let display_ptr = {
                let ptr = display.ok_or(FragmentColorError::new_err(
                    "Display handle is missing for Wayland",
                ))? as *mut c_void;
                NonNull::new(ptr).ok_or(FragmentColorError::new_err(
                    "Could not convert u64 to c_void for Wayland display",
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

        #[cfg(target_os = "windows")]
        "windows" => {
            use std::num::NonZeroIsize;
            use wgpu::rwh::{
                RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
            };

            let window_ptr = {
                NonZeroIsize::new(window as isize).ok_or(FragmentColorError::new_err(
                    "Could not convert u64 to isize for Win32 window",
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
        }

        #[cfg(target_os = "macos")]
        "cocoa" => {
            use objc2::msg_send;
            use objc2_app_kit::{NSView, NSWindow};
            use std::ffi::c_void;
            use std::ptr::NonNull;
            use wgpu::rwh::{
                AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
            };

            let ns_window = window as *mut NSWindow;
            let ns_view_ptr: *mut NSView = unsafe { msg_send![ns_window, contentView] };
            let ns_view = NonNull::new(ns_view_ptr as *mut c_void).ok_or(
                FragmentColorError::new_err("Could not convert *mut NSView to c_void"),
            )?;

            let appkit_window_handle = RawWindowHandle::AppKit(AppKitWindowHandle::new(ns_view));
            let appkit_display_handle = RawDisplayHandle::AppKit(AppKitDisplayHandle::new());

            let window_handle = unsafe { WindowHandle::borrow_raw(appkit_window_handle) };
            let display_handle = unsafe { DisplayHandle::borrow_raw(appkit_display_handle) };

            Ok(WindowHandles {
                window_handle,
                display_handle,
            })
        }

        _ => Err(FragmentColorError::new_err(format!(
            "Unsupported platform: {:?} (window id: {:?}; display: {:?})",
            platform, window, display,
        ))),
    }
}
