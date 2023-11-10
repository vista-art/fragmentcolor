use crate::{
    app::{error::READ_LOCK_ERROR, error::WRITE_LOCK_ERROR, Container, Event, PLRender},
    renderer::target::HasSize,
};
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

pub trait CallbackFn<E>: FnMut(E) + Send + Sync {}
impl<E, F> CallbackFn<E> for F where F: FnMut(E) + Send + Sync {}

type Callback<E> = Arc<RwLock<dyn CallbackFn<E>>>;

// @TODO remove deprecated "raw"s, waiting for https://github.com/gfx-rs/wgpu/pull/4202
pub trait IsWindow:
    HasRawDisplayHandle + HasRawWindowHandle /* + HasDisplayHandle + HasWindowHandle*/ + HasSize
{
    fn id(&self) -> winit::window::WindowId;
    fn state(&mut self) -> Arc<RwLock<WindowState>>;
    fn request_redraw(&self);
}

pub trait EventListener {
    fn on(&mut self, name: &str, callback: Callback<Event>);
    fn call(&self, name: &str, event: Event);
}

pub struct WindowState {
    pub instance: winit::window::Window,
    pub callbacks: HashMap<String, Vec<Callback<Event>>>,
    pub auto_resize: bool,
}

impl EventListener for WindowState {
    fn on(&mut self, name: &str, callback: Callback<Event>) {
        self.callbacks
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }

    fn call(&self, name: &str, event: Event) {
        if let Some(callbacks) = self.callbacks.get(name) {
            callbacks.iter().for_each(|callback| {
                let mut callback = callback.write().expect(WRITE_LOCK_ERROR);
                callback(event.clone());
            });
        }
    }
}

impl Debug for WindowState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Instance: {:?}, Total Arc<RwLock<impl Fn(Event)>>s: {:?}",
            self.instance,
            self.callbacks.len()
        )
    }
}

#[derive(Debug)]
pub struct Window {
    state: Arc<RwLock<WindowState>>,
}

impl Debug for dyn IsWindow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Window ID: {:?}", self.id())
    }
}

#[derive(Debug)]
pub struct Windows {
    container: HashMap<WindowId, Arc<RwLock<WindowState>>>,
}

impl Container<WindowId, WindowState> for Windows {
    fn new() -> Self {
        Self {
            container: HashMap::new(),
        }
    }

    fn get(&self, id: WindowId) -> Option<RwLockReadGuard<'_, WindowState>> {
        let window = self.container.get(&id)?;
        let window = window.read().expect(READ_LOCK_ERROR);
        Some(window)
    }

    fn get_mut(&mut self, id: WindowId) -> Option<RwLockWriteGuard<'_, WindowState>> {
        let window = self.container.get_mut(&id)?;
        let window = window.write().expect(WRITE_LOCK_ERROR);
        Some(window)
    }

    fn insert(&mut self, id: WindowId, window: Arc<RwLock<WindowState>>) {
        self.container.insert(id, window);
    }

    fn remove(&mut self, id: WindowId) -> Option<Arc<RwLock<WindowState>>> {
        self.container.remove(&id)
    }

    fn len(&self) -> usize {
        self.container.len()
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
        self.state
            .read()
            .expect(READ_LOCK_ERROR)
            .instance
            .raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.state
            .read()
            .expect(READ_LOCK_ERROR)
            .instance
            .raw_display_handle()
    }
}

impl IsWindow for Window {
    fn id(&self) -> winit::window::WindowId {
        self.state.read().expect(READ_LOCK_ERROR).instance.id()
    }
    fn state(&mut self) -> Arc<RwLock<WindowState>> {
        self.state.clone()
    }
    fn request_redraw(&self) {
        self.state
            .read()
            .expect(READ_LOCK_ERROR)
            .instance
            .request_redraw()
    }
}

impl HasSize for Window {
    fn size(&self) -> wgpu::Extent3d {
        let size = self
            .state
            .read()
            .expect(READ_LOCK_ERROR)
            .instance
            .inner_size();

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
    pub auto_resize: Option<bool>,
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
            auto_resize: Some(true),
        }
    }
}

impl Default for Window {
    /// Creates a Window with default options.
    ///
    /// ## Panics!
    /// This method panics if the OS cannot create the window.
    /// If you would like to handle the error, use Window::new()
    /// instead and provide the options manually.
    fn default() -> Self {
        Self::new(WindowOptions {
            ..Default::default()
        })
        .expect("Failed to create default window")
    }
}

impl Window {
    pub fn new(options: WindowOptions) -> Result<Self, winit::error::OsError> {
        let app = PLRender::app();
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
            .build(app.event_loop().window_target())?;

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

        let mut window = Window {
            state: Arc::new(RwLock::new(WindowState {
                instance: window,
                callbacks: HashMap::new(),
                auto_resize: options.auto_resize.unwrap_or(true),
            })),
        };

        pollster::block_on(app.add_window(&mut window));

        Ok(window)
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .instance
            .set_title(title);
        self
    }

    pub fn set_size(&mut self, size: (u32, u32)) -> &mut Self {
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .instance
            .set_inner_size(winit::dpi::Size::Logical(size.into()));
        self
    }

    pub fn set_min_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .instance
            .set_min_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }

    pub fn set_max_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .instance
            .set_max_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }

    pub fn set_auto_resize(&mut self, auto_resize: bool) -> &mut Self {
        self.state.write().expect(WRITE_LOCK_ERROR).auto_resize = auto_resize;
        self
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> &mut Self {
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .instance
            .set_fullscreen(fullscreen.then(|| winit::window::Fullscreen::Borderless(None)));
        self
    }

    pub fn set_decorations(&mut self, decorations: bool) -> &mut Self {
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .instance
            .set_decorations(decorations);
        self
    }

    pub fn set_resizable(&mut self, resizable: bool) -> &mut Self {
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .instance
            .set_resizable(resizable);
        self
    }

    pub async fn run(&mut self) {
        PLRender::run().await;
    }

    pub fn on(&self, event_name: &str, callback: impl CallbackFn<Event> + 'static) {
        let callback = Arc::new(RwLock::new(callback));
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .on(event_name, callback)
    }

    pub fn call(&self, event_name: &str, event: Event) {
        self.state
            .read()
            .expect(READ_LOCK_ERROR)
            .call(event_name, event)
    }
}
