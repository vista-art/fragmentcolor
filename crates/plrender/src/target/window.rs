use crate::target::{events::Event, HasSize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use winit::window::WindowId;

// Waiting for https://github.com/gfx-rs/wgpu/pull/4202
// use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use serde::{Deserialize, Serialize};

const READ_LOCK_ERROR: &str = "Failed to acquire Read lock";
const WRITE_LOCK_ERROR: &str = "Failed to acquire Write lock";

// @TODO remove deprecated "raw"s, waiting for https://github.com/gfx-rs/wgpu/pull/4202
pub trait IsWindow:
    HasRawDisplayHandle + HasRawWindowHandle /* + HasDisplayHandle + HasWindowHandle*/ + HasSize
{
    fn id(&self) -> winit::window::WindowId;
    // workaround for casting it to the 
    // only concrete type we currently have.
    fn instance(&mut self) -> Arc<RwLock<winit::window::Window>>;
    fn request_redraw(&self);
}

#[derive(Debug)]
pub struct Window {
    instance: Arc<RwLock<winit::window::Window>>,
}

impl Debug for dyn IsWindow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Window ID: {:?}", self.id())
    }
}

pub trait WindowContainer<W> {
    fn new() -> Self;
    fn get(&self, id: WindowId) -> Option<RwLockReadGuard<'_, W>>;
    fn get_mut(&mut self, id: WindowId) -> Option<RwLockWriteGuard<'_, W>>;
    fn insert(&mut self, id: WindowId, window: Arc<RwLock<W>>);
    fn remove(&mut self, id: WindowId) -> Option<Arc<RwLock<W>>>;
}

#[derive(Debug)]
pub struct Windows {
    container: HashMap<winit::window::WindowId, Arc<RwLock<winit::window::Window>>>,
}

impl WindowContainer<winit::window::Window> for Windows {
    fn new() -> Self {
        Self {
            container: HashMap::new(),
        }
    }

    fn get(&self, id: WindowId) -> Option<RwLockReadGuard<'_, winit::window::Window>> {
        let window = self.container.get(&id)?;
        let window = window.read().expect(READ_LOCK_ERROR);
        Some(window)
    }

    fn get_mut(&mut self, id: WindowId) -> Option<RwLockWriteGuard<'_, winit::window::Window>> {
        let window = self.container.get_mut(&id)?;
        let window = window.write().expect(WRITE_LOCK_ERROR);
        Some(window)
    }

    fn insert(&mut self, id: WindowId, window: Arc<RwLock<winit::window::Window>>) {
        self.container.insert(id, window);
    }

    fn remove(&mut self, id: WindowId) -> Option<Arc<RwLock<winit::window::Window>>> {
        self.container.remove(&id)
    }
}

// Waiting for https://github.com/gfx-rs/wgpu/pull/4202
//
// impl HasWindowHandle for Window {
//     fn raw_window_handle(&self) -> WindowHandle {
//         self.instance.window_handle()
//     }
// }
//
// impl HasDisplayHandle for Window {
//     fn raw_display_handle(&self) -> DisplayHandle {
//         self.instance.display_handle()
//     }
// }

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.instance
            .read()
            .expect(READ_LOCK_ERROR)
            .raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.instance
            .read()
            .expect(READ_LOCK_ERROR)
            .raw_display_handle()
    }
}

impl IsWindow for Window {
    fn id(&self) -> winit::window::WindowId {
        self.instance.read().expect(READ_LOCK_ERROR).id()
    }
    fn instance(&mut self) -> Arc<RwLock<winit::window::Window>> {
        self.instance.clone()
    }
    fn request_redraw(&self) {
        self.instance
            .read()
            .expect(READ_LOCK_ERROR)
            .request_redraw()
    }
}

impl HasSize for Window {
    fn size(&self) -> wgpu::Extent3d {
        let size = self.instance.read().expect(READ_LOCK_ERROR).inner_size();

        wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        }
    }

    fn aspect(&self) -> f32 {
        let size = self.size();
        size.width as f32 / size.height as f32
    }
}

/// All properties are optional, so this can be serialized
/// to JSON. The user can inject a configuration object in
/// in Javascript. In Python, we convert the input to named
/// arguments or a single dict.
///
/// While this is a nice interface for Python and Javascript,
/// it's not idiomatic in Rust. For Rust users, we provide a
/// default() construcxtor and chainable setters.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowOptions {
    pub decorations: Option<bool>,
    pub fullscreen: Option<bool>,
    pub resizable: Option<bool>,
    pub title: Option<String>,
    pub size: Option<(u32, u32)>,
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            decorations: Some(true),
            fullscreen: Some(false),
            resizable: Some(true),
            title: Some("PLRender".to_string()),
            size: Some((800, 600)),
            min_size: None,
            max_size: None,
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new(WindowOptions::default()).unwrap()
    }
}

impl Window {
    pub fn new(options: WindowOptions) -> Result<Self, winit::error::OsError> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title(options.title.as_ref().unwrap_or(&"PLRender".to_string()))
            .with_inner_size(winit::dpi::Size::Logical(
                options.size.unwrap_or((800, 600)).into(),
            ))
            .with_fullscreen(
                options
                    .fullscreen
                    .unwrap_or(false)
                    .then(|| winit::window::Fullscreen::Borderless(None)),
            )
            .with_decorations(options.decorations.unwrap_or(true))
            .with_resizable(options.resizable.unwrap_or(true))
            .build(&event_loop)?;

        window.set_min_inner_size(
            options
                .min_size
                .map(|size| winit::dpi::Size::Logical(size.into())),
        );
        window.set_max_inner_size(
            options
                .max_size
                .map(|size| winit::dpi::Size::Logical(size.into())),
        );

        Ok(Window {
            instance: Arc::new(RwLock::new(window)),
        })
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.instance
            .write()
            .expect(WRITE_LOCK_ERROR)
            .set_title(title);
        self
    }

    pub fn set_size(&mut self, size: (u32, u32)) -> &mut Self {
        self.instance
            .write()
            .expect(WRITE_LOCK_ERROR)
            .set_inner_size(winit::dpi::Size::Logical(size.into()));
        self
    }

    pub fn set_min_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.instance
            .write()
            .expect(WRITE_LOCK_ERROR)
            .set_min_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }

    pub fn set_max_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.instance
            .write()
            .expect(WRITE_LOCK_ERROR)
            .set_max_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> &mut Self {
        self.instance
            .write()
            .expect(WRITE_LOCK_ERROR)
            .set_fullscreen(fullscreen.then(|| winit::window::Fullscreen::Borderless(None)));
        self
    }

    pub fn set_decorations(&mut self, decorations: bool) -> &mut Self {
        self.instance
            .write()
            .expect(WRITE_LOCK_ERROR)
            .set_decorations(decorations);
        self
    }

    pub fn set_resizable(&mut self, resizable: bool) -> &mut Self {
        self.instance
            .write()
            .expect(WRITE_LOCK_ERROR)
            .set_resizable(resizable);
        self
    }

    // @TODO implement this interface
    pub fn on(_event_name: &str, _callback: impl FnMut(Event)) {
        todo!()
    }
}
