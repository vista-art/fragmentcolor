use crate::{
    app::{
        container::Container,
        event_loop::{run_event_loop, EventLoop},
        panics::Panic,
        window::{IsWindow, WindowState, Windows},
        Event,
    },
    renderer::{Renderer, RendererOptions},
    scene::{Scene, SceneState, Scenes},
};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
#[cfg(wasm)]
use wasm_bindgen::prelude::*;
use winit::{event_loop::EventLoopProxy, window::WindowId};

pub(crate) const ASSETS: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/images");

// Type abbbreviations
pub(crate) type Error = Box<dyn std::error::Error>;
pub(crate) type RemovedWindow = Option<Arc<RwLock<WindowState>>>;
pub(crate) type WindowsReadGuard<'w> = RwLockReadGuard<'w, Windows>;
pub(crate) type WindowsWriteGuard<'w> = RwLockWriteGuard<'w, Windows>;
pub(crate) type RemovedScene = Option<Arc<RwLock<SceneState>>>;
pub(crate) type ScenesReadGuard<'s> = RwLockReadGuard<'s, Scenes>;
pub(crate) type ScenesWriteGuard<'s> = RwLockWriteGuard<'s, Scenes>;

/// The main App instance responsible for managing shared global resources.
type MainApp = Arc<RwLock<App>>;
/// The main Renderer instance owned by the App.
type MainRenderer = Arc<RwLock<Renderer>>;

#[cfg(not(wasm))]
pub(crate) static APP: OnceLock<MainApp> = OnceLock::new();

#[cfg(not(wasm))]
pub(crate) static RENDERER: OnceLock<MainRenderer> = OnceLock::new();

#[cfg(wasm)]
thread_local! {
    pub(crate) static APP: OnceLock<MainApp> = OnceLock::new();
    pub(crate) static RENDERER: OnceLock<MainRenderer> = OnceLock::new();
}

#[cfg_attr(wasm, wasm_bindgen)]
/// The end user interface for configuring and running the App.
///  
/// Typically, the only functions you need to call from this
/// struct are `PLRender::config()` and `PLRender::run()`.
pub struct PLRender;

#[cfg_attr(wasm, wasm_bindgen)]
impl PLRender {
    /// Configure the global shared App instance with startup options.
    ///
    /// Notice that **this function must be be called before any other
    /// function of this library** for it to be effective. If the App
    /// has already been initialized by another part of your program,
    /// this function will do nothing.
    ///
    /// # Examples
    /// ```
    /// use plrender::{PLRender, RendererOptions, AppMetadata};
    ///
    /// let options = plrender::AppOptions {
    ///     log_level: "info".to_string(),
    ///     renderer: plrender::RendererOptions {
    ///         force_software_rendering: true,
    ///        ..Default::default()
    ///     }
    /// };
    ///
    /// PLRender::config(options);
    ///
    /// let app = PLRender::app().read().unwrap();
    ///
    /// assert_eq!(app.name(), "plrender");
    /// ```
    pub fn config(options: AppOptions) {
        #[cfg(not(wasm))]
        APP.get_or_init(|| Arc::new(RwLock::new(App::new(options))));
        #[cfg(wasm)]
        APP.with(|app| app.get_or_init(|| Arc::new(RwLock::new(App::new(options)))));
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
    pub(crate) fn app() -> &'static Arc<RwLock<App>> {
        #[cfg(not(wasm))]
        let app = APP.get_or_init(|| Arc::new(RwLock::new(App::new(AppOptions::default()))));
        #[cfg(wasm)]
        let app = APP
            .with(|app| app.get_or_init(|| Arc::new(RwLock::new(App::new(AppOptions::default())))));

        app
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
    pub(crate) fn renderer() -> &'static Arc<RwLock<Renderer>> {
        #[cfg(not(wasm))]
        let renderer = RENDERER.get_or_init(|| {
            let app = Self::app().read().expect("Could not Read App");
            let renderer = pollster::block_on(app.state().init_offscreen_renderer())
                .expect("Failed to create Renderer");

            Arc::new(RwLock::new(renderer))
        });
        #[cfg(wasm)]
        let renderer = RENDERER.with(|renderer| {
            renderer.get_or_init(|| {
                let app = Self::app().read().expect("Could not Read App");
                let renderer = pollster::block_on(app.state().init_offscreen_renderer())
                    .expect("Failed to create Renderer");

                Arc::new(RwLock::new(renderer))
            })
        });

        renderer
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
    /// # Examples
    /// ```
    /// use plrender::PLRender;
    ///
    /// PLRender::run();
    /// ```
    pub fn run() {
        let app = Self::app().read().expect("Could not Run App");

        pollster::block_on(app.run());
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
pub(crate) struct AppState {
    pub windows: Arc<RwLock<Windows>>,
    pub scenes: Arc<RwLock<Scenes>>,
    pub options: AppOptions,
}

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// App's startup options.
pub struct AppOptions {
    pub log_level: String,
    pub renderer: RendererOptions,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            renderer: RendererOptions::default(),
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
        #[cfg(not(wasm))]
        if APP.get().is_some() {
            Panic::trying_to_create_app_directly()
        }
        #[cfg(wasm)]
        if APP.with(|app| app.get()).is_some() {
            Panic::trying_to_create_app_directly()
        }

        let log_level = options.log_level.clone();
        let level_filter = LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Info);
        Self::init_logger(level_filter);

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

    #[cfg(not(wasm))]
    /// Initializes the logger with the given log level.
    fn init_logger(level_filter: log::LevelFilter) {
        env_logger::builder().filter_level(level_filter).init();
    }

    #[cfg(wasm)]
    /// Initializes the logger with the given log level.
    fn init_logger(level_filter: log::LevelFilter) {
        let level = level_filter.to_level().unwrap_or(log::Level::Info);
        //console_error_panic_hook::set_once();
        console_log::init_with_level(level).unwrap_or(println!("Failed to initialize logger"));
    }

    /// Runs the main event loop.
    ///
    /// # Side effects
    /// Initializes an offscreen Renderer if the app doesn't have any windows.
    ///
    /// # Panics
    /// - Panics if the current thread is dead
    ///   while acquiring the Event Loop mutex lock.
    pub(crate) async fn run(&self) {
        let _ = self.state().init_offscreen_renderer().await;

        let runner = Box::new(run_event_loop);
        self.lock_event_loop().run(runner, self.state.clone()).await;
    }

    /// Locks the internal state and Returns the mutex guard to it.
    pub(super) fn state(&self) -> RwLockReadGuard<'_, AppState> {
        self.state.read().expect("Could not Read App State")
    }

    /// Locks the main Event Loop and Returns the mutex guard to it.
    pub(crate) fn lock_event_loop(&self) -> MutexGuard<'_, EventLoop<Event>> {
        self.event_loop
            .try_lock()
            .expect("Could not Read EventLoop")
    }

    /// Dispatches an event to the main event loop.
    ///
    /// # Errors
    /// - Returns an Error if the EventLoop is closed.
    /// - Returns an Error if it fails to acquire the
    ///   Event Dispatcher mutex lock.
    #[allow(dead_code)]
    pub fn dispatch_event(&'static self, event: Event) -> Result<(), Error> {
        let dispatcher = self.event_dispatcher.try_lock()?;
        Ok(dispatcher.send_event(event)?)
    }

    /// Returns a new Arc RwLock reference to the Windows collection.
    // #[allow(dead_code)]
    pub fn windows(&self) -> Arc<RwLock<Windows>> {
        self.state().windows()
    }

    /// Adds a window to the Windows collection.
    ///
    /// ## Side effects
    /// Lazy-initializes the Renderer when we add the first Window
    pub(crate) fn add_window<W: IsWindow>(&self, window: &mut W) {
        self.state().add_window(window);
    }

    /// Returns a new Arc Mutex reference to the Scenes collection.
    pub fn scenes(&self) -> Arc<RwLock<Scenes>> {
        self.state().scenes()
    }

    /// Adds a scene to the Scenes collection.
    ///
    /// New Scene instances register themselves to the App,
    /// so users cannot call this function directly.
    pub(crate) fn add_scene(&self, scene: &mut Scene) {
        self.state().add_scene(scene);
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
    pub(crate) fn read_windows_collection<W: IsWindow>(&self) -> WindowsReadGuard<'_> {
        self.windows
            .read()
            .expect("Could not Read Windows Collection")
    }

    /// Returns a Write mutex reference to the Windows collection.
    ///
    /// # Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub(crate) fn write_to_windows_collection<W: IsWindow>(&self) -> WindowsWriteGuard<'_> {
        self.windows
            .write()
            .expect("Could not Write to Windows Collection")
    }

    /// Adds a window to the Windows collection.
    ///
    /// # Side effects
    /// Lazy-initializes the internal Renderer when we add the first Window
    pub(crate) fn add_window<'w, W: IsWindow>(&self, window: &'w mut W) {
        self.get_or_init_renderer::<W>(window);

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
    pub(crate) fn read_scenes_collection(&self) -> ScenesReadGuard<'_> {
        self.scenes
            .read()
            .expect("Could not get Scenes Collection Read lock")
    }

    /// Returns a Write mutex reference to the Scenes collection.
    ///
    /// # Panics
    /// - Panics if the current thread is dead while acquiring the mutex lock.
    pub(crate) fn write_to_scenes_collection(&self) -> ScenesWriteGuard<'_> {
        self.scenes
            .write()
            .expect("Could not get Scenes Collection Write lock")
    }

    /// Adds a scene to the Scenes collection.
    pub(crate) fn add_scene<'s>(&self, scene: &'s mut Scene) {
        let mut scenes = self.write_to_scenes_collection();

        scenes.insert(&scene.id(), scene.state())
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

        renderer.read().expect("Could not Read Renderer State")
    }

    #[cfg(not(wasm))]
    /// Gets or initializes the global Renderer.
    ///
    /// # Panics
    /// - Panics if it fails to create the global Renderer.
    fn get_or_init_renderer<'w, W: IsWindow>(&self, window: &'w mut W) -> &MainRenderer {
        RENDERER.get_or_init(|| {
            Arc::new(RwLock::new(
                pollster::block_on(self.init_renderer::<W>(window))
                    .expect("Failed to create Renderer"),
            ))
        })
    }

    #[cfg(wasm)]
    /// Gets or initializes the global Renderer.
    ///
    /// # Panics
    /// - Panics if it fails to create the global Renderer.
    fn get_or_init_renderer<'w, W: IsWindow>(&self, window: &'w mut W) -> &MainRenderer {
        RENDERER.with(|renderer| {
            renderer.get_or_init(|| {
                Arc::new(RwLock::new(
                    pollster::block_on(self.init_renderer::<W>(window))
                        .expect("Failed to create Renderer"),
                ))
            })
        })

        // if let Some(renderer) = self.renderer().clone() {
        //     &renderer
        // } else {
        //     self.renderer = Some({
        //         Arc::new(RwLock::new(
        //             pollster::block_on(self.init_renderer::<W>(window))
        //                 .expect("Failed to create Renderer"),
        //         ))
        //     });
        //     &self.renderer.as_ref().unwrap().clone()
        // }
    }

    #[cfg(not(wasm))]
    /// Gets or initializes the global Renderer without testing for
    /// compatibility with a Window.
    ///
    /// # Panics
    /// - Panics if it fails to create the global Renderer.
    fn get_or_init_offscreen_renderer(&self) -> &MainRenderer {
        #[cfg(not(wasm))]
        RENDERER.get_or_init(|| {
            Arc::new(RwLock::new(
                pollster::block_on(self.init_offscreen_renderer())
                    .expect("Failed to create Renderer"),
            ))
        })
    }

    #[cfg(wasm)]
    fn get_or_init_offscreen_renderer(&self) -> &MainRenderer {
        RENDERER.with(|renderer| {
            renderer.get_or_init(|| {
                Arc::new(RwLock::new(
                    pollster::block_on(self.init_offscreen_renderer())
                        .expect("Failed to create Renderer"),
                ))
            })
        })
    }

    /// Initializes the Renderer.
    ///
    /// This function will ensure compatibility with the provided Window(s).
    /// If no Window is provided, it will initialize an offscreen Renderer
    /// that uses any GPU adapter, without checking for compatibility.
    async fn init_renderer<'w, W: IsWindow>(&self, window: &'w mut W) -> Result<Renderer, Error> {
        Ok(Renderer::new(
            RendererOptions {
                force_software_rendering: self.options.renderer.force_software_rendering,
                power_preference: self.options.renderer.power_preference.clone(),
                panic_on_error: self.options.renderer.panic_on_error.clone(),
                device_limits: self.options.renderer.device_limits.clone(),
                render_pass: self.options.renderer.render_pass.clone(),
            },
            Some(window),
        )
        .await?)
    }

    /// Initializes an Offscreen Renderer.
    ///
    /// It uses any GPU adapter, without checking for compatibility.
    async fn init_offscreen_renderer(&self) -> Result<Renderer, Error> {
        Ok(Renderer::new_offscreen(RendererOptions {
            force_software_rendering: self.options.renderer.force_software_rendering,
            power_preference: self.options.renderer.power_preference.clone(),
            panic_on_error: self.options.renderer.panic_on_error.clone(),
            device_limits: self.options.renderer.device_limits.clone(),
            render_pass: self.options.renderer.render_pass.clone(),
        })
        .await?)
    }
}
