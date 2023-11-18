use crate::{
    app::{
        error::{READ_LOCK_ERROR, WRITE_LOCK_ERROR},
        events::{Callback, CallbackFn, Event},
        PLRender,
    },
    math::geometry::Quad,
    renderer::target::Dimensions,
};
use instant::SystemTime;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use winit::window::WindowId;

// Waiting for https://github.com/gfx-rs/wgpu/pull/4202
// use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

// @TODO remove deprecated "raw"s, waiting for https://github.com/gfx-rs/wgpu/pull/4202
pub trait IsWindow:
    HasRawDisplayHandle + HasRawWindowHandle /* + HasDisplayHandle + HasWindowHandle*/ + Dimensions
{
    fn id(&self) -> winit::window::WindowId;
    fn state(&self) -> Arc<RwLock<WindowState>>;
    fn request_redraw(&self);
}

impl Debug for dyn IsWindow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Window ID: {:?}", self.id())
    }
}

pub trait EventListener {
    fn on(&mut self, name: &str, callback: Callback<Event>);
    fn call(&self, name: &str, event: Event);
}

pub struct WindowState {
    pub instance: winit::window::Window,
    pub callbacks: HashMap<String, Vec<Callback<Event>>>,
    pub auto_resize: bool,
    pub close_on_esc: bool,
    pub hovered_files: HashMap<u128, PathBuf>,
    reverse_lookup: HashMap<PathBuf, u128>,
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

impl Dimensions for WindowState {
    fn size(&self) -> Quad {
        let size = self.instance.inner_size();

        Quad::from_dimensions(size.width, size.height)
    }

    fn aspect(&self) -> f32 {
        self.size().aspect()
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

impl WindowState {
    pub fn redraw(&self) {
        self.instance.request_redraw();
    }

    pub fn get_hovered_file(&mut self, index: u128) -> Option<String> {
        if let Some(path) = self.hovered_files.get(&index) {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }

    pub fn get_dropped_file(&mut self, index: u128) -> Option<PathBuf> {
        if let Some(path) = self.hovered_files.remove(&index) {
            self.reverse_lookup.remove(&path);
            Some(path)
        } else {
            None
        }
    }

    pub(crate) fn add_hovered_file(&mut self, file: &PathBuf) -> u128 {
        let index = self.hovered_timestamp();
        self.hovered_files.insert(index, file.clone());
        self.reverse_lookup.insert(file.clone(), index);
        index
    }

    pub(crate) fn get_dropped_file_handle(&self, file: &PathBuf) -> Option<u128> {
        self.reverse_lookup.get(file).copied()
    }

    fn hovered_timestamp(&self) -> u128 {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        duration_since_epoch.as_nanos()
    }
}

#[derive(Debug, Default)]
pub struct Windows {
    container: HashMap<WindowId, Arc<RwLock<WindowState>>>,
}

crate::app::macros::implements_container!(Windows, <&WindowId, WindowState>);

#[derive(Debug)]
pub struct Window {
    state: Arc<RwLock<WindowState>>,
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
        self.read_state().instance.raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.read_state().instance.raw_display_handle()
    }
}

impl IsWindow for Window {
    fn id(&self) -> winit::window::WindowId {
        self.read_state().instance.id()
    }
    fn state(&self) -> Arc<RwLock<WindowState>> {
        self.state.clone()
    }
    fn request_redraw(&self) {
        self.read_state().instance.request_redraw()
    }
}

impl Dimensions for Window {
    fn size(&self) -> Quad {
        self.read_state().size()
    }

    fn aspect(&self) -> f32 {
        self.size().aspect()
    }
}

/// Clones the inner state reference count, creates
/// a new Window with the same state and returns it,
/// dropping the original Window.
impl Clone for Window {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

/// Options to create a Window.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowOptions {
    pub decorations: bool,
    pub fullscreen: bool,
    pub resizable: bool,
    pub title: String,
    pub size: (u32, u32),
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub auto_resize: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            decorations: true,
            fullscreen: false,
            resizable: true,
            title: "PLRender".to_string(),
            size: (800, 600),
            min_size: None,
            max_size: None,
            auto_resize: true,
        }
    }
}

impl Default for Window {
    /// Creates a Window with default options.
    ///
    /// # Panics!
    /// This method panics if the OS cannot create the window.
    ///
    /// If you would like to handle the error,
    /// use `Window::new(WindowOptions::default())`
    /// instead, which returns a `Result<Window, OsError>`.
    fn default() -> Self {
        let window = Self::create().expect("Failed to create default window");

        window
    }
}

impl Window {
    /// Creates a Window with default options.
    pub fn create() -> Result<Self, winit::error::OsError> {
        Self::new(WindowOptions::default())
    }

    /// Creates a Window with the given options.
    pub fn new(options: WindowOptions) -> Result<Self, winit::error::OsError> {
        let app = PLRender::app().read().expect("Could not get App Read lock");

        let fullscreen = options
            .fullscreen
            .then(|| winit::window::Fullscreen::Borderless(None));

        let window = winit::window::WindowBuilder::new()
            .with_title(options.title)
            .with_inner_size(winit::dpi::Size::Logical(options.size.into()))
            .with_fullscreen(fullscreen)
            .with_decorations(options.decorations)
            .with_resizable(options.resizable)
            .build(app.lock_event_loop().window_target())?;

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
                auto_resize: options.auto_resize,
                close_on_esc: true,
                hovered_files: HashMap::new(),
                reverse_lookup: HashMap::new(),
            })),
        };

        pollster::block_on(app.add_window(&mut window));

        Ok(window)
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.write_state().instance.set_title(title);
        self
    }

    pub fn set_size(&mut self, size: (u32, u32)) -> &mut Self {
        self.write_state()
            .instance
            .set_inner_size(winit::dpi::Size::Logical(size.into()));
        self
    }

    pub fn set_min_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.write_state()
            .instance
            .set_min_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }

    pub fn set_max_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.write_state()
            .instance
            .set_max_inner_size(size.map(|size| winit::dpi::Size::Logical(size.into())));
        self
    }

    pub fn set_auto_resize(&mut self, auto_resize: bool) -> &mut Self {
        self.write_state().auto_resize = auto_resize;
        self
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> &mut Self {
        self.write_state()
            .instance
            .set_fullscreen(fullscreen.then(|| winit::window::Fullscreen::Borderless(None)));
        self
    }

    pub fn set_decorations(&mut self, decorations: bool) -> &mut Self {
        self.write_state().instance.set_decorations(decorations);
        self
    }

    pub fn set_resizable(&mut self, resizable: bool) -> &mut Self {
        self.write_state().instance.set_resizable(resizable);
        self
    }

    pub async fn run(&mut self) {
        PLRender::run().await;
    }

    pub fn redraw(&self) {
        self.read_state().redraw();
    }

    pub fn on(&self, event_name: &str, callback: impl CallbackFn<Event> + 'static) {
        let callback = Arc::new(RwLock::new(callback));
        self.state
            .write()
            .expect(WRITE_LOCK_ERROR)
            .on(event_name, callback)
    }

    pub fn call(&self, event_name: &str, event: Event) {
        self.read_state().call(event_name, event)
    }

    pub fn get_hovered_file(&mut self, index: u128) -> Option<String> {
        self.write_state().get_hovered_file(index)
    }

    pub fn get_dropped_file(&mut self, index: u128) -> Option<PathBuf> {
        self.write_state().get_dropped_file(index)
    }

    fn read_state(&self) -> RwLockReadGuard<'_, WindowState> {
        self.state.read().expect(READ_LOCK_ERROR)
    }

    fn write_state(&mut self) -> RwLockWriteGuard<'_, WindowState> {
        self.state.write().expect(WRITE_LOCK_ERROR)
    }
}
