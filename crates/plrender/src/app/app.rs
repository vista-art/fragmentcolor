use crate::{
    app::{
        container::Container,
        event_loop::run_event_loop,
        window::{IsWindow, Window, WindowState, Windows},
        Event, EventLoop,
    },
    renderer::{RenderOptions, Renderer},
    scene::{Scene, SceneState, Scenes},
};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use winit::{
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::WindowId,
};

pub const ROOT: &'static str = env!("CARGO_MANIFEST_DIR");

type Error = Box<dyn std::error::Error>;
type RemovedWindow = Option<Arc<RwLock<WindowState>>>;
type RemovedScene = Option<Arc<RwLock<SceneState>>>;

/// The main App instance responsible for managing
/// the resources created by the user of this library.
type MainApp = Arc<RwLock<App>>;
pub static APP: OnceLock<MainApp> = OnceLock::new();

/// The main Renderer instance owned by the App.
type MainRenderer = Arc<RwLock<Renderer>>;
pub static RENDERER: OnceLock<MainRenderer> = OnceLock::new();

/// The backbone of this library.
///
/// It is responsible for initializing the main App instance
/// and providing a shared reference to it.
///  
/// Typically, the only function you need to call from this
/// struct is `PLRender::run()`.
///
/// If you need to configure the main
///
/// PLRender::config(options);
/// PLRender::run();
pub struct PLRender;

/// Alias for [`PLRender::run()`].
///
/// # Example
/// ```
/// use plrender::PLRender;
///
/// PLRender::run();
/// ```
pub async fn run() {
    PLRender::run().await;
}

impl PLRender {
    /// Configure the main App instance with startup options.
    ///
    /// Notice that **this function must be be called before any other
    /// function of this library** for it to be effective. If the App
    /// has already been initialized by another part of your program,
    /// this function will do nothing.
    ///
    /// # Example
    /// ```
    /// use plrender::PLRender;
    ///
    /// let options = plrender::AppOptions {
    ///     log_level: Some("info"),
    ///     renderer: Some(plrender::RenderOptions {
    ///         force_software_rendering: Some(true),
    ///         render_pass: Some("solid"),
    ///        ..Default::default()
    ///     }
    /// });
    ///
    /// PLRender::config(options);
    /// ```
    pub fn config(options: AppOptions) {
        APP.get_or_init(|| Arc::new(RwLock::new(App::new(options))));
    }

    /// Returns a mutex reference to the main App.
    ///
    /// If the App has not been initialized yet, it will be crated with default
    /// options. If you need to configure the main App, use `PLRender::config()`
    /// as the very first call of your program.
    ///
    /// Users do not need to call this function directly. Typically, internal
    /// objects of this library (mainly the Window or the Scene) will call it
    /// when they need to access the main Event Loop or the Renderer.
    pub fn app() -> &'static Arc<RwLock<App>> {
        APP.get_or_init(|| Arc::new(RwLock::new(App::new(AppOptions::default()))))
    }

    /// Returns a mutex reference to the main Renderer.
    ///
    /// If the Renderer has not been initialized yet, it will be crated with default
    /// options. If you need to configure the main Renderer, use `PLRender::config()`
    /// as the very first call of your program.
    ///
    /// Users do not need to call this function directly. Typically, internal
    /// objects of this library (mainly the Window or the Scene) will call it
    /// when they need to access the main Event Loop or the Renderer.
    pub fn renderer() -> &'static Arc<RwLock<Renderer>> {
        RENDERER.get_or_init(|| {
            let app = Self::app().read().expect("Could not get App Read lock");
            let renderer = pollster::block_on(app.state().init_offscreen_renderer())
                .expect("Failed to create Renderer");

            Arc::new(RwLock::new(renderer))
        })
    }

    /// Runs the main event loop. This function blocks the thread
    /// and never returns, until the user closes all windows.
    ///
    /// # Side effects
    /// Initializes the main App with default options if it hasn't
    /// been initialized yet.
    ///
    /// # Platform-specific
    /// On the Web, this function will return immediately and use
    /// the browser's event loop instead. It will hook the library's
    /// main loop into the `window.requestAnimationFrame()` callbacks.
    ///
    /// # Example
    /// ```
    /// use plrender::PLRender;
    ///
    /// PLRender::run();
    /// ```
    pub async fn run() {
        let mut app = Self::app().write().expect("Could not get App Write lock");

        app.run().await;
    }
}

/// The main App instance responsible for managing
/// the resources created by users of this library.
#[derive(Debug)]
pub struct App {
    event_loop: Arc<Mutex<EventLoop<Event>>>,
    event_dispatcher: Arc<Mutex<EventLoopProxy<Event>>>,
    state: Arc<RwLock<AppState>>,
}

/// App's internal state shared between threads.
#[derive(Debug)]
pub struct AppState {
    pub windows: Arc<RwLock<Windows>>,
    pub scenes: Arc<RwLock<Scenes>>,
    pub options: AppOptions,
}

/// App's startup options.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppOptions {
    pub log_level: String,
    pub renderer: RenderOptions,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            renderer: RenderOptions::default(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(AppOptions::default())
    }
}

impl App {
    /// Creates a new App instance.
    ///
    /// The App's constructor is private.
    /// Please use `PLRender::app()` to create it,
    /// or `PLRender::config()` to create it with options.
    ///
    /// # Panics
    /// - Panics if the App has already been initialized.
    fn new(options: AppOptions) -> Self {
        if APP.get().is_some() {
            panic!(
                "App already initialized.

                Please use PLRender::app() to get the main App instance."
            )
        }

        let log_level = options.log_level.clone();
        env_logger::builder()
            .filter_level(LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Info))
            .init();

        let event_loop = EventLoop::new();
        let event_dispatcher = event_loop.create_dispatcher();

        Self {
            event_loop: Arc::new(Mutex::new(event_loop)),
            event_dispatcher: Arc::new(Mutex::new(event_dispatcher)),
            state: Arc::new(RwLock::new(AppState {
                windows: Arc::new(RwLock::new(Windows::new())),
                scenes: Arc::new(RwLock::new(Scenes::new())),
                options,
            })),
        }
    }

    /// Runs the main event loop.
    ///
    /// # Side effects
    /// Initializes an offscreen Renderer if the app doesn't have any windows.
    ///
    /// # Panics
    /// - Panics if the current thread is dead
    ///   while acquiring the Event Loop mutex lock.
    pub async fn run(&mut self) {
        let _ = self.state().init_renderer::<Window>(vec![]).await;

        let runner = Box::new(run_event_loop);
        self.event_loop
            .try_lock()
            .expect("Could not get EventLoop mutex lock")
            .run(runner, self.state.clone())
            .await;
    }

    /// Locks the internal state and Returns the mutex guard to it.
    pub(crate) fn state(&self) -> RwLockReadGuard<'_, AppState> {
        self.state
            .read()
            .expect("Could not get AppState Read mutex lock")
    }

    /// Locks the main Event Loop and Returns the mutex guard to it.
    pub(crate) fn lock_event_loop(&self) -> MutexGuard<'_, EventLoop<Event>> {
        self.event_loop
            .try_lock()
            .expect("Could not get EventLoop mutex lock")
    }

    /// Dispatches an event to the main event loop.
    ///
    /// # Panics
    /// - Panics if the current thread is dead
    ///   while acquiring the Event Dispatcher mutex lock.
    pub fn dispatch_event(&self, event: Event) -> Result<(), EventLoopClosed<Event>> {
        Ok(self
            .event_dispatcher
            .try_lock()
            .expect("Could not get Event Dispatcher mutex lock")
            .send_event(event)?)
    }

    /// Returns a new Arc Mutex reference to the Windows collection.
    #[allow(dead_code)]
    pub(crate) fn windows(&self) -> Arc<RwLock<Windows>> {
        self.state().windows()
    }

    /// Adds a window to the Windows collection.
    ///
    /// ## Side effects
    /// Lazy-initializes the Renderer when we add the first Window
    pub(crate) async fn add_window<W: IsWindow>(&self, window: &mut W) {
        self.state().add_window(window).await;
    }

    /// Removes a window from the Windows collection.
    pub(crate) fn remove_window<W: IsWindow>(&self, window: &WindowId) -> RemovedWindow {
        self.state().remove_window::<W>(window)
    }

    /// Returns a new Arc Mutex reference to the Scenes collection.
    pub(crate) fn scenes(&self) -> Arc<RwLock<Scenes>> {
        self.state().scenes()
    }

    /// Adds a scene to the Scenes collection.
    pub(crate) fn add_scene(&self, scene: &mut Scene) {
        self.state().add_scene(scene);
    }

    /// Removes a scene from the Scenes collection.
    pub(crate) fn remove_scene(&self, scene: Scene) -> RemovedScene {
        self.state().remove_scene(scene)
    }
}

#[allow(dead_code)]
impl AppState {
    /// Returns a new Arc Mutex reference to the Windows collection.
    pub(crate) fn windows(&self) -> Arc<RwLock<Windows>> {
        self.windows.clone()
    }

    /// Returns a Read mutex reference to the Windows collection.
    ///
    /// # Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub(crate) fn read_windows_collection<W: IsWindow>(&self) -> RwLockReadGuard<'_, Windows> {
        self.windows
            .read()
            .expect("Could not get Windows Collection mutex lock")
    }

    /// Returns a Write mutex reference to the Windows collection.
    ///
    /// # Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub(crate) fn write_to_windows_collection<W: IsWindow>(&self) -> RwLockWriteGuard<'_, Windows> {
        self.windows
            .write()
            .expect("Could not get Windows Collection mutex lock")
    }

    /// Adds a window to the Windows collection.
    ///
    /// # Side effects
    /// Lazy-initializes the internal Renderer when we add the first Window
    pub(crate) async fn add_window<'w, W: IsWindow>(&self, window: &'w mut W) {
        self.get_or_init_renderer::<W>(vec![window]);

        let mut windows = self.write_to_windows_collection::<W>();
        windows.insert(&window.id(), window.state());
    }

    /// Removes a window from the Windows collection.
    pub(crate) fn remove_window<W: IsWindow>(&self, window_id: &WindowId) -> RemovedWindow {
        let mut windows = self.write_to_windows_collection::<W>();
        windows.remove(window_id)
    }

    /// Returns a new Arc Mutex reference to the Scenes collection.
    pub(crate) fn scenes(&self) -> Arc<RwLock<Scenes>> {
        self.scenes.clone()
    }

    /// Returns a Read mutex reference to the Scenes collection.
    ///
    /// # Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub(crate) fn read_scenes_collection(&self) -> RwLockReadGuard<'_, Scenes> {
        self.scenes
            .read()
            .expect("Could not get Scenes Collection mutex lock")
    }

    /// Returns a Write mutex reference to the Scenes collection.
    ///
    /// # Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub(crate) fn write_to_scenes_collection(&self) -> RwLockWriteGuard<'_, Scenes> {
        self.scenes
            .write()
            .expect("Could not get Scenes Collection mutex lock")
    }

    /// Adds a scene to the Scenes collection.
    pub(crate) fn add_scene<'s>(&self, scene: &'s mut Scene) {
        let mut scenes = self.write_to_scenes_collection();

        scenes.insert(&scene.id(), scene.state());
    }

    /// Removes a window from the Windows collection.
    pub(crate) fn remove_scene(&self, scene: Scene) -> RemovedScene {
        let mut scenes = self.write_to_scenes_collection();
        scenes.remove(&scene.id())
    }

    /// Returns a Read mutex reference to the main Renderer.
    ///
    /// # Side effects
    /// Lazy-initializes the Renderer with default options if it doesn't exist.
    /// Calling this function before creating a Window will create an offscreen
    /// global Renderer that will not check for compatibility with any Window.
    ///
    /// # Panics
    /// - Panics if it fails to create the global Renderer.
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub(crate) fn renderer(&self) -> RwLockReadGuard<'_, Renderer> {
        let renderer = self.get_or_init_offscreen_renderer();

        renderer
            .read()
            .expect("Could not get Renderer Read mutex lock")
    }

    /// Gets or initializes the global Renderer.
    ///
    /// # Panics
    /// - Panics if it fails to create the global Renderer.
    fn get_or_init_renderer<'w, W: IsWindow>(&self, windows: Vec<&'w mut W>) -> &MainRenderer {
        RENDERER.get_or_init(|| {
            Arc::new(RwLock::new(
                pollster::block_on(self.init_renderer::<W>(windows))
                    .expect("Failed to create Renderer"),
            ))
        })
    }

    /// Gets or initializes the global Renderer without testing for
    /// compatibility with a Window.
    ///
    /// # Panics
    /// - Panics if it fails to create the global Renderer.
    fn get_or_init_offscreen_renderer(&self) -> &MainRenderer {
        RENDERER.get_or_init(|| {
            Arc::new(RwLock::new(
                pollster::block_on(self.init_offscreen_renderer())
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
        Ok(Renderer::new(
            RenderOptions {
                force_software_rendering: self.options.renderer.force_software_rendering,
                power_preference: self.options.renderer.power_preference.clone(),
                device_limits: self.options.renderer.device_limits.clone(),
                render_pass: self.options.renderer.render_pass.clone(),
            },
            windows,
        )
        .await?)
    }

    /// Initializes an Offscreen Renderer.
    ///
    /// It uses any GPU adapter, without checking for compatibility.
    async fn init_offscreen_renderer(&self) -> Result<Renderer, Error> {
        Ok(Renderer::new_offscreen(RenderOptions {
            force_software_rendering: self.options.renderer.force_software_rendering,
            power_preference: self.options.renderer.power_preference.clone(),
            device_limits: self.options.renderer.device_limits.clone(),
            render_pass: self.options.renderer.render_pass.clone(),
        })
        .await?)
    }
}
