use crate::target::{events::Event, HasSize};
use std::collections::HashMap;
use std::fmt::Debug;
use winit::window::WindowId;

// Waiting for https://github.com/gfx-rs/wgpu/pull/4202
// use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use serde::{Deserialize, Serialize};

// @TODO remove deprecated "raw"s, waiting for https://github.com/gfx-rs/wgpu/pull/4202
pub trait IsWindow:
    HasRawDisplayHandle + HasRawWindowHandle /* + HasDisplayHandle + HasWindowHandle*/ + HasSize
{
    fn id(&self) -> WindowId;
    // workaround for casting it to the 
    // only concrete type we currently have.
    fn instance(self) -> Window;
    fn request_redraw(&self);
}

pub trait WindowContainer {
    type Window: IsWindow;

    fn new() -> Self;
    fn get(&self, id: WindowId) -> Option<&Box<Self::Window>>;
    fn get_mut(&mut self, id: WindowId) -> Option<&mut Self::Window>;
    fn insert(&mut self, id: WindowId, window: Self::Window);
    fn remove(&mut self, id: WindowId) -> Option<Self::Window>;
}

pub trait From<W: IsWindow> {
    fn from(window: W) -> Self;
}

impl<W: IsWindow> From<W> for Window {
    fn from(w: W) -> Window {
        w.instance()
    }
}

#[derive(Debug)]
pub struct Windows {
    windows: HashMap<winit::window::WindowId, Box<self::Window>>,
}
impl Debug for dyn IsWindow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Window ID: {:?}", self.id())
    }
}

impl WindowContainer for Windows {
    type Window = Window;

    fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    fn get(&self, id: WindowId) -> Option<&Box<Self::Window>> {
        self.windows.get(&id)
    }

    fn get_mut(&mut self, id: WindowId) -> Option<&mut Self::Window> {
        self.windows.get_mut(&id).map(|boxed| Box::as_mut(boxed))
    }

    fn insert(&mut self, id: WindowId, window: Self::Window) {
        self.windows.insert(id, Box::new(window));
    }

    fn remove(&mut self, id: WindowId) -> Option<Self::Window> {
        self.windows.remove(&id).map(|boxed| *boxed)
    }
}

#[derive(Debug)]
pub struct Window {
    instance: winit::window::Window,
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
        self.instance.raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.instance.raw_display_handle()
    }
}

impl IsWindow for Window {
    fn id(&self) -> winit::window::WindowId {
        self.instance.id()
    }
    fn instance(self) -> Window {
        self
    }
    fn request_redraw(&self) {
        self.instance.request_redraw()
    }
}

impl HasSize for Window {
    fn size(&self) -> wgpu::Extent3d {
        let size = self.instance.inner_size();

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
            .with_min_inner_size(winit::dpi::Size::Logical(
                options.min_size.unwrap_or_default().into(),
            ))
            .with_max_inner_size(winit::dpi::Size::Logical(
                options.max_size.unwrap_or_default().into(),
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

        Ok(Window { instance: window })
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.instance.set_title(title);
        self
    }

    pub fn set_size(&mut self, size: (u32, u32)) -> &mut Self {
        self.instance
            .set_inner_size(winit::dpi::Size::Logical(size.into()));
        self
    }

    pub fn set_min_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.instance
            .set_min_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }

    pub fn set_max_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.instance
            .set_max_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> &mut Self {
        self.instance
            .set_fullscreen(fullscreen.then(|| winit::window::Fullscreen::Borderless(None)));
        self
    }

    pub fn set_decorations(&mut self, decorations: bool) -> &mut Self {
        self.instance.set_decorations(decorations);
        self
    }

    pub fn set_resizable(&mut self, resizable: bool) -> &mut Self {
        self.instance.set_resizable(resizable);
        self
    }

    // @TODO implement this interface
    pub fn on(_event_name: &str, _callback: impl FnMut(Event)) {
        todo!()
    }
}
