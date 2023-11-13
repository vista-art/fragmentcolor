use crate::{
    app::{
        container::Container,
        event_loop::run_event_loop,
        window::{IsWindow, Window, WindowOptions, Windows},
        Event, EventLoop,
    },
    renderer::{RenderOptions, Renderer},
    scene::Scenes,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use winit::{
    error::OsError,
    event_loop::{EventLoopClosed, EventLoopProxy},
};

type Error = Box<dyn std::error::Error>;

/// The main App instance responsible for managing
/// the resources created by the user of this library.
type MainApp = Arc<Mutex<App>>;
static APP: OnceLock<MainApp> = OnceLock::new();

/// The main Renderer instance owned by the App.
type MainRenderer = Arc<Mutex<Renderer>>;
static RENDERER: OnceLock<MainRenderer> = OnceLock::new();

/// The backbone of the library.
///
/// It is responsible for initializing the main App instance
/// and providing a shared reference to it.
///  
/// Users do not need to call this struct directly, as every
/// library object that needs an App reference will call it
/// internally.
pub struct PLRender;
impl PLRender {
    /// Returns a mutex reference to the main App.
    ///
    /// The App startup options are used only in the first call and
    /// ignored in subsequent calls. Calling this function multiple
    /// times is equivalent to calling `PLRender::app()`.
    pub fn new(options: AppOptions) -> MutexGuard<'static, App> {
        let app = APP.get_or_init(|| Arc::new(Mutex::new(App::new(options))));
        app.try_lock().expect("Could not get App mutex lock")
    }

    /// Returns a mutex reference to the main App.
    ///
    /// If the App has not been initialized yet,
    /// it will be crated with default options.
    pub fn app() -> MutexGuard<'static, App> {
        let app = APP.get_or_init(|| Arc::new(Mutex::new(App::new(AppOptions::default()))));
        app.try_lock().expect("Could not get App mutex lock")
    }

    /// Runs the main event loop.
    pub async fn run() {
        let mut app = PLRender::app();
        app.run().await;
    }
}

#[derive(Debug)]
pub struct App {
    event_loop: Arc<Mutex<EventLoop<Event>>>,
    event_dispatcher: EventLoopProxy<Event>,
    state: Arc<Mutex<AppState>>,
}

#[derive(Debug)]
pub struct AppState {
    pub windows: Arc<Mutex<Windows>>,
    pub scenes: Arc<Mutex<Scenes>>,
    pub options: AppOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppOptions {
    pub log_level: Option<&'static str>,
    pub force_software_rendering: Option<bool>,
    pub power_preference: Option<&'static str>,
    pub device_limits: Option<&'static str>,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            log_level: Some("info"),
            force_software_rendering: Some(false),
            power_preference: Some("default"),
            device_limits: Some("default"),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(AppOptions::default())
    }
}

impl App {
    fn new(options: AppOptions) -> Self {
        // @TODO init logger
        let _log_level = *options.log_level.as_ref().unwrap_or(&"info");

        let event_loop = EventLoop::new();
        let event_dispatcher = event_loop.create_dispatcher();

        Self {
            event_loop: Arc::new(Mutex::new(event_loop)),
            event_dispatcher,
            state: Arc::new(Mutex::new(AppState {
                windows: Arc::new(Mutex::new(Windows::new())),
                scenes: Arc::new(Mutex::new(Scenes::new())),
                options,
            })),
        }
    }

    /// Runs the main event loop.
    ///
    /// ## Side effects
    /// Initializes an offscreen Renderer if the app doesn't have any windows.
    pub async fn run(&mut self) {
        let _ = self.state().init_renderer::<Window>(vec![]).await;

        let runner = Box::new(run_event_loop);
        self.event_loop
            .try_lock()
            .unwrap()
            .run(runner, self.state.clone())
            .await;
    }

    /// Returns a mutex reference to the AppState.
    pub fn state(&self) -> MutexGuard<'_, AppState> {
        self.state
            .try_lock()
            .expect("Could not get AppState mutex lock")
    }

    /// Returns a reference to the main event loop.
    pub fn event_loop(&self) -> MutexGuard<'_, EventLoop<Event>> {
        self.event_loop
            .try_lock()
            .expect("Could not get EventLoop mutex lock")
    }

    /// Dispatches an event to the main event loop.
    pub fn dispatch_event(&self, event: Event) -> Result<(), EventLoopClosed<Event>> {
        Ok(self.event_dispatcher.send_event(event)?)
    }

    /// Creates a window, adds it to the Windows collection, and returns it.
    ///
    /// ## Side effects
    /// Lazy-initializes the Renderer when we create the first Window
    pub async fn new_window(&self) -> Result<Window, OsError> {
        let mut window = Window::new(WindowOptions::default())?;
        self.add_window(&mut window).await;
        Ok(window)
    }

    /// Adds a window to the Windows collection.
    ///
    /// ## Side effects
    /// Lazy-initializes the Renderer when we add the first Window
    pub async fn add_window<W: IsWindow>(&self, window: &mut W) {
        self.state().add_window(window).await;
    }

    /// Removes a window from the Windows collection.
    pub fn remove_window<W: IsWindow>(&self, window: W) {
        self.state().remove_window(window)
    }
}

impl AppState {
    /// Returns a mutex reference to the Windows collection.
    ///
    /// ## Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub fn windows<W: IsWindow>(&self) -> MutexGuard<'_, Windows> {
        self.windows
            .try_lock()
            .expect("Could not get Windows Collection mutex lock")
    }

    /// Adds a window to the Windows collection.
    ///
    /// ## Side effects
    /// Lazy-initializes the internal Renderer when we add the first Window
    pub async fn add_window<'w, W: IsWindow>(&mut self, window: &'w mut W) {
        self.get_or_init_renderer::<W>(vec![window]);

        let mut windows = self.windows::<W>();
        windows.insert(window.id(), window.state());
    }

    /// Removes a window from the Windows collection.
    pub fn remove_window<W: IsWindow>(&self, window: W) {
        let mut windows = self.windows::<W>();
        windows.remove(window.id());
    }

    /// Returns a mutex reference to the Windows collection.
    ///
    /// ## Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub fn scenes<W: IsWindow>(&self) -> MutexGuard<'_, Windows> {
        self.windows
            .try_lock()
            .expect("Could not get Windows Collection mutex lock")
    }

    // @TODO Add scene to the global collection
    //       and bind it to the window.
    //
    /// Adds a scene to the Scenes collection.
    // pub async fn add_scene<'s>(&mut self, scene: &'s mut Scene) {
    //     let mut scenes = self.scenes::<Scene>();
    //     scenes.insert(scene.id(), window.state());
    // }

    /// Removes a window from the Windows collection.
    pub fn remove_scene<W: IsWindow>(&self, window: W) {
        let mut scenes = self.scenes::<W>();
        scenes.remove(window.id());
    }

    /// Returns a mutex reference to the main Renderer.
    ///
    /// ## Side effects
    /// Lazy-initializes the Renderer with default options if it doesn't exist.
    /// Calling this function before creating a Window will create an offscreen
    /// global Renderer that will not check for compatibility with any Window.
    ///
    /// ## Panics
    /// - Panics if it fails to create the global Renderer.
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub fn renderer<W: IsWindow>(&self) -> MutexGuard<'_, Renderer> {
        let renderer = self.get_or_init_renderer::<W>(vec![]);

        renderer
            .try_lock()
            .expect("Could not get Renderer mutex lock")
    }

    /// Gets or initializes the global Renderer.
    ///
    /// ## Panics
    /// - Panics if it fails to create the global Renderer.
    fn get_or_init_renderer<'w, W: IsWindow>(&self, windows: Vec<&'w mut W>) -> &MainRenderer {
        RENDERER.get_or_init(|| {
            Arc::new(Mutex::new(
                pollster::block_on(self.init_renderer::<W>(windows))
                    .expect("Failed to create Renderer"),
            ))
        })
    }

    /// Initializes the Renderer.
    ///
    /// This function will ensure compatibility with the provided Window(s).
    /// If no Window is provided, it will initialize an offscreen Renderer
    /// that uses any GPU adapter, without checking for compatibility.
    async fn init_renderer<'w, W: IsWindow>(
        &self,
        windows: Vec<&'w mut W>,
    ) -> Result<Renderer, Error> {
        let targets = match windows.len() {
            0 => None,
            _ => Some(windows),
        };

        Ok(Renderer::new(RenderOptions {
            force_software_rendering: self.options.force_software_rendering,
            power_preference: self.options.power_preference,
            device_limits: self.options.device_limits,
            render_pass: None, // @TODO
            targets,
        })
        .await?)
    }
}
