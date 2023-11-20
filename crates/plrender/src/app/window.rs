use crate::{
    app::{
        error::{READ_LOCK_ERROR, WRITE_LOCK_ERROR},
        events::{Callback, CallbackFn, Event},
        PLRender,
    },
    math::geometry::Quad,
    renderer::target::Dimensions,
};
use instant::Instant;
use instant::SystemTime;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
pub use winit::window::WindowId;

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
}

pub(crate) trait EventProcessor: EventListener {
    fn call(&self, name: &str, event: Event);
    fn call_later(&self, at: Instant, name: &str, event: Event);
    fn process_calls(&self);
}

type CallStack = Vec<(Callback<Event>, Event)>;

pub struct WindowState {
    pub auto_resize: bool,
    pub close_on_esc: bool,
    pub hovered_files: HashMap<u128, PathBuf>,
    pub target_frametime: Option<f64>,
    pub(crate) instance: winit::window::Window,
    reverse_lookup: HashMap<PathBuf, u128>,
    callbacks: HashMap<String, Vec<Callback<Event>>>,
    callstack: RwLock<CallStack>,
    scheduled: RwLock<BTreeMap<Instant, (String, Event)>>,
}

impl EventListener for WindowState {
    /// Registers a callback function for this window.
    ///
    /// The first argument is the name of the event that will trigger the function,
    /// for example "draw" or "keyup".
    ///
    /// The second argument is the callback function itself. It can take a closure
    /// or a function that implements the `FnMut(Event) + Send + Sync` trait.
    fn on(&mut self, name: &str, callback: Callback<Event>) {
        self.callbacks
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }
}

impl EventProcessor for WindowState {
    /// Adds an immediate callback to the callstack.
    ///
    /// The event will be processed at the current iteration of the Event Loop.
    ///
    /// # Errors
    /// - If the CallStack for this Window is already locked by other process,
    ///   the event will be dropped silently and the Window will log an Error.
    fn call(&self, name: &str, event: Event) {
        let callbacks = if let Some(callbacks) = self.callbacks.get(name) {
            callbacks
        } else if let Some(callbacks) = self.callbacks.get("any") {
            callbacks
        } else {
            return;
        };

        if let Ok(mut callstack) = self.callstack.try_write() {
            callbacks.iter().for_each(|callback| {
                callstack.push((callback.clone(), event.clone()));
            })
        } else {
            log::error!(
                "Failed to acquire the CallStack Mutex for for WindowState.call({}, {:?})!",
                name,
                event
            );
        };
    }

    /// Adds a callback to the callstack to be processed at a later time.
    ///
    /// The event will be processed at the given time.
    ///
    /// # Errors
    /// - If the Scheduled Events map is already locked by other process,
    ///   the event will be dropped silently and the Window will log an Error.
    fn call_later(&self, time: Instant, name: &str, event: Event) {
        if let Ok(mut scheduled) = self.scheduled.try_write() {
            scheduled.insert(time, (name.to_string(), event));
        } else {
            log::error!(
                "Failed to acquire the Scheduled Events Mutex for for WindowState.call_later({:?}, {}, {:?})!",
                time,
                name,
                event
            );
        };
    }

    //@TODO implement retrying
    /// Processes all the callstacks at the end of the Event Loop.
    ///
    /// # Errors
    /// - If this CallStack for this Window is already locked by other process,
    ///   no event will be processed and the Window will log an Error.
    /// - If the Window fails to acquite the Write mutex for the callback,
    ///   it won't be called and the Window will log an Error.
    fn process_calls(&self) {
        if let Ok(mut scheduled) = self.scheduled.try_write() {
            let mut future_events = scheduled.split_off(&Instant::now());

            while let Some((_, (name, event))) = scheduled.pop_first() {
                self.call(&name, event);
            }

            scheduled.append(&mut future_events);
        } else {
            log::error!("Failed to acquire Write Lock for Scheduled Events!");
        };

        if let Ok(mut callstack) = self.callstack.try_write() {
            while let Some((callback, event)) = callstack.pop() {
                if let Ok(mut callback) = callback.try_write() {
                    callback(event.clone());
                } else {
                    log::error!(
                        "Failed to acquire Write Lock for Callback '{:?}'!
                        The Event '{:?}' will be dropped.",
                        callback,
                        event
                    );
                }
            }
        } else {
            log::error!("Failed to acquire CallStack Write Lock for WindowState.process_calls()!");
        };
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

    pub fn get_hovered_file(&self, index: u128) -> Option<String> {
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
    pub keys: Vec<WindowId>,
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
    pub close_on_esc: bool,
    pub framerate: Option<u32>,
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
            close_on_esc: true,
            framerate: None,
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
                auto_resize: options.auto_resize,
                close_on_esc: options.close_on_esc,
                target_frametime: framerate_to_frametime(options.framerate),
                hovered_files: HashMap::new(),
                reverse_lookup: HashMap::new(),
                callbacks: HashMap::new(),
                callstack: RwLock::new(Vec::new()),
                scheduled: RwLock::new(BTreeMap::new()),
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

    pub fn set_close_on_esc(&mut self, close_on_esc: bool) -> &mut Self {
        self.write_state().close_on_esc = close_on_esc;
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

    pub fn set_framerate(&mut self, framerate: Option<u32>) -> &mut Self {
        self.write_state().target_frametime = framerate_to_frametime(framerate);
        self
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.write_state().instance.set_visible(visible);
        self
    }

    pub fn run(&mut self) {
        PLRender::run();
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

fn framerate_to_frametime(framerate: Option<u32>) -> Option<f64> {
    if let Some(framerate) = framerate {
        Some(1.0 / framerate as f64)
    } else {
        None
    }
}
