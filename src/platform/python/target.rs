use crate::{FragmentColorError, Renderer, Target, TargetFrame, UniformData};
use pyo3::prelude::*;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
#[pyclass]
#[derive(Debug)]
pub struct RenderCanvasTarget {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

#[pyclass]
#[derive(Debug)]
pub struct RenderCanvasFrame {
    pub(crate) surface_texture: wgpu::SurfaceTexture,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) view: wgpu::TextureView,
}

#[pymethods]
impl RenderCanvasTarget {
    fn size(&self) -> [u32; 2] {
        let size = <Self as Target>::size(self);
        [size.width, size.height]
    }

    fn resize(&mut self, renderer: &Renderer, size: UniformData) {
        <Self as Target>::resize(self, renderer, size.into());
    }
}

impl Target for RenderCanvasTarget {
    fn size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        }
    }

    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&renderer.device, &self.config);
    }

    fn get_current_frame(&self) -> Result<Box<dyn crate::TargetFrame>, wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(Box::new(RenderCanvasFrame {
            surface_texture,
            format: self.config.format,
            view,
        }))
    }
}

impl TargetFrame for RenderCanvasFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn present(self: Box<Self>) {
        self.surface_texture.present();
    }

    /// Prevents the Renderer to call present() automatically
    /// to allow RenderCanvas to control the presentation
    fn auto_present(&self) -> bool {
        false
    }
}

pub(crate) struct PyWindowHandle<'window> {
    pub(crate) window_handle: WindowHandle<'window>,
    pub(crate) display_handle: DisplayHandle<'window>,
}

unsafe impl<'window> Send for PyWindowHandle<'window> {}
unsafe impl<'window> Sync for PyWindowHandle<'window> {}

impl<'window> HasWindowHandle for PyWindowHandle<'window> {
    fn window_handle(&self) -> Result<WindowHandle<'window>, HandleError> {
        Ok(self.window_handle)
    }
}

impl<'window> HasDisplayHandle for PyWindowHandle<'window> {
    fn display_handle(&self) -> Result<DisplayHandle<'window>, HandleError> {
        Ok(self.display_handle)
    }
}

pub(crate) fn create_raw_handles<'window>(
    platform: String,
    window: u64,
    display: Option<u64>,
) -> Result<(WindowHandle<'window>, DisplayHandle<'window>), PyErr> {
    match platform.as_str() {
        #[cfg(target_os = "linux")]
        "x11" => {
            use raw_window_handle::{
                RawDisplayHandle, RawWindowHandle, XlibDisplayHandle, XlibWindowHandle,
            };
            use std::ffi::{c_ulong, c_void};
            use std::ptr::NonNull;

            let display_ptr = {
                let ptr = display.ok_or(FragmentColorError::new_err(
                    "Display handle is missing for Xlib",
                ))? as *mut c_void;
                NonNull::new(ptr).ok_or(FragmentColorError::new_err(
                    "Could not convert u64 to c_void for Xlib display",
                ))?
            };

            let window: c_ulong = window.try_into().map_err(|_| {
                FragmentColorError::new_err(
                    "Window Id out of range: Could not convert u64 to u32 for Xlib",
                )
            })?;

            let xlib_window_handle = RawWindowHandle::Xlib(XlibWindowHandle::new(window));
            let xlib_display_handle =
                RawDisplayHandle::Xlib(XlibDisplayHandle::new(Some(display_ptr), 0));

            let window_handle = unsafe { WindowHandle::borrow_raw(xlib_window_handle) };
            let display_handle = unsafe { DisplayHandle::borrow_raw(xlib_display_handle) };

            Ok((window_handle, display_handle))
        }

        #[cfg(target_os = "linux")]
        "wayland" => {
            use raw_window_handle::{
                RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
            };
            use std::ffi::c_void;
            use std::ptr::NonNull;

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

            Ok((window_handle, display_handle))
        }

        #[cfg(target_os = "windows")]
        "windows" => {
            use raw_window_handle::{
                RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
            };
            use std::num::NonZeroIsize;

            let window_ptr = {
                NonZeroIsize::new(window as isize).ok_or(FragmentColorError::new_err(
                    "Could not convert u64 to isize for Win32 window",
                ))?
            };

            let win32_window_handle = RawWindowHandle::Win32(Win32WindowHandle::new(window_ptr));
            let win32_display_handle = RawDisplayHandle::Windows(WindowsDisplayHandle::new());

            let window_handle = unsafe { WindowHandle::borrow_raw(win32_window_handle) };
            let display_handle = unsafe { DisplayHandle::borrow_raw(win32_display_handle) };

            Ok((window_handle, display_handle))
        }

        #[cfg(target_os = "macos")]
        "cocoa" => {
            use objc2::msg_send;
            use objc2_app_kit::{NSView, NSWindow};
            use raw_window_handle::{
                AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
            };
            use std::ffi::c_void;
            use std::ptr::NonNull;

            let ns_window = window as *mut NSWindow;
            let ns_view_ptr: *mut NSView = unsafe { msg_send![ns_window, contentView] };
            let ns_view = NonNull::new(ns_view_ptr as *mut c_void).ok_or(
                FragmentColorError::new_err("Could not convert *mut NSView to c_void"),
            )?;

            let appkit_window_handle = RawWindowHandle::AppKit(AppKitWindowHandle::new(ns_view));
            let appkit_display_handle = RawDisplayHandle::AppKit(AppKitDisplayHandle::new());

            let window_handle = unsafe { WindowHandle::borrow_raw(appkit_window_handle) };
            let display_handle = unsafe { DisplayHandle::borrow_raw(appkit_display_handle) };

            Ok((window_handle, display_handle))
        }

        _ => Err(FragmentColorError::new_err(format!(
            "Unsupported platform: {:?} (window id: {:?}; display: {:?})",
            platform, window, display,
        ))),
    }
}
