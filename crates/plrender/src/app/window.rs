use crate::{
    app::{App, Event},
    renderer::target::HasSize,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use winit::window::WindowId;

const READ_LOCK_ERROR: &str = "Failed to acquire Read mutex lock";
const WRITE_LOCK_ERROR: &str = "Failed to acquire Write mutex lock";

// Waiting for https://github.com/gfx-rs/wgpu/pull/4202
// use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use serde::{Deserialize, Serialize};

// pub trait Callback<E>: Fn(E) + Send + Sync + 'static {}
// impl<E, F> Callback<E> for F where F: Fn(E) + Send + Sync + 'static {}

type Callback<E> = Arc<RwLock<dyn Fn(E) + Send + Sync + 'static>>;

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
                let callback = callback.read().expect(READ_LOCK_ERROR);
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
    // @TODO this should be private.
    //       It is temporarily public to make
    //       the renderer accessible in the Rust examples.
    //       Ideally, we should have a method to access the renderer
    pub app: Option<App>,
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
    fn len(&self) -> usize;
}

#[derive(Debug)]
pub struct Windows {
    container: HashMap<WindowId, Arc<RwLock<WindowState>>>,
}

impl WindowContainer<WindowState> for Windows {
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
    /// Create a singleton Window with default options and an internal event loop.
    ///
    /// # Panics!
    /// Because Winit must have only one event loop running in the main thread,
    /// this method will panic if you try to create a second window.
    ///
    /// Use it if you are sure your application will use only one window.
    /// Otherwise, use Window::new(&app, options) to inject an external
    /// App instance which holds the global event loop.
    ///
    /// This method is useful for quickly creating Rust example applications with
    /// less boilerplate. It is not supposed to be used in the public Js+Py API.
    fn default() -> Self {
        let app = App::default();
        let mut window = Self::new(
            &app,
            WindowOptions {
                ..Default::default()
            },
        )
        .expect("Failed to create default window");
        window.app = Some(app);
        window
    }
}

impl Window {
    pub fn new(app: &App, options: WindowOptions) -> Result<Self, winit::error::OsError> {
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
            })),
            app: None,
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
        if self.app.is_some() {
            self.app.as_mut().unwrap().run().await
        }
    }

    pub fn on(&mut self, event_name: &str, callback: Callback<Event>) {
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
