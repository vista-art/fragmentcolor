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
use std::sync::{Arc, Mutex, MutexGuard};
use winit::event_loop::EventLoopProxy;

type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct App {
    event_loop: Arc<Mutex<EventLoop<Event>>>,
    event_dispatcher: EventLoopProxy<Event>,
    state: Arc<Mutex<AppState>>,
}

#[derive(Debug)]
pub struct AppState {
    pub renderer: Arc<Mutex<Option<Renderer>>>,
    pub windows: Arc<Mutex<Windows>>,
    pub scenes: Arc<Mutex<Scenes>>,
    pub options: AppOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppOptions {
    pub log_level: Option<String>,
    pub force_software_rendering: Option<bool>,
    pub power_preference: Option<&'static str>,
    pub device_limits: Option<&'static str>,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            log_level: Some("info".to_string()),
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
    pub fn new(options: AppOptions) -> Self {
        // @TODO init logger
        let _log_level = options
            .log_level
            .as_ref()
            .unwrap_or(&"info".to_string())
            .clone();

        let event_loop = EventLoop::new();
        let event_dispatcher = event_loop.create_dispatcher();

        Self {
            event_loop: Arc::new(Mutex::new(event_loop)),
            event_dispatcher,
            state: Arc::new(Mutex::new(AppState {
                renderer: Arc::new(Mutex::new(None)),
                windows: Arc::new(Mutex::new(Windows::new())),
                scenes: Arc::new(Mutex::new(Scenes::new())),
                options,
            })),
        }
    }

    /// Runs the main event loop.
    /// ## Side effects:
    /// Initializes the Renderer if it doesn't exist.
    pub async fn run(&mut self) {
        if self.state().renderer().is_none() {
            let _ = self.state().init_renderer::<Window>(vec![]).await;
        }

        let runner = Box::new(run_event_loop);
        self.event_loop
            .try_lock()
            .unwrap()
            .run(runner, self.state.clone())
            .await;
    }

    /// Returns a reference to the main event loop.
    pub fn event_loop(&self) -> MutexGuard<'_, EventLoop<Event>> {
        self.event_loop.try_lock().unwrap()
    }

    /// Dispatches an event to the main event loop.
    pub fn dispatch_event(&self, event: Event) {
        self.event_dispatcher.send_event(event).unwrap();
    }

    /// Creates a window, adds it to the Windows collection, and returns it.
    /// ## Side effects:
    /// Lazy-initializes the Renderer when we create the first Window
    pub async fn new_window(&self) -> Result<Window, Error> {
        let mut window = Window::new(self, WindowOptions::default())?;
        self.add_window(&mut window).await;
        Ok(window)
    }

    /// Adds a window to the Windows collection.
    /// ## Side effects:
    /// Lazy-initializes the Renderer when we add the first Window
    pub async fn add_window<W: IsWindow>(&self, window: &mut W) {
        self.state().add_window(window).await;
    }

    /// Removes a window from the Windows collection.
    pub fn remove_window<W: IsWindow>(&self, window: W) {
        self.state().remove_window(window)
    }

    /// Returns a mutex reference to the AppState.
    pub fn state(&self) -> MutexGuard<'_, AppState> {
        self.state
            .try_lock()
            .expect("Could not get AppState mutex lock")
    }
}

impl AppState {
    /// Returns a mutex reference to the Renderer.
    /// Panics if the current thread is poisoned.
    pub fn renderer(&self) -> MutexGuard<'_, Option<Renderer>> {
        self.renderer
            .try_lock()
            .expect("Could not get Renderer mutex lock")
    }

    /// Returns a mutex reference to the Windows collection.
    /// Panics if the current thread is poisoned.
    pub fn windows<W: IsWindow>(&self) -> MutexGuard<'_, Windows> {
        self.windows
            .try_lock()
            .expect("Could not get Windows Collection mutex lock")
    }

    /// Adds a window to the Windows collection.
    /// ## Side effects:
    /// Lazy-initializes the Renderer when we add the first Window
    pub async fn add_window<'w, W: IsWindow>(&mut self, window: &'w mut W) {
        if self.renderer().is_none() {
            let _ = self.init_renderer(vec![window]).await;
        }

        let mut windows = self.windows::<W>();
        windows.insert(window.id(), window.state());
    }

    /// Removes a window from the Windows collection.
    pub fn remove_window<W: IsWindow>(&self, window: W) {
        let mut windows = self.windows::<W>();
        windows.remove(window.id());
    }

    /// Initializes the Renderer.
    ///
    /// This function will ensure compatibility with the provided Window(s).
    /// If no Window is provided, it will initialize an offscreen Renderer
    /// that uses any GPU adapter, without checking for compatibility.
    async fn init_renderer<'w, W: IsWindow>(
        &mut self,
        windows: Vec<&'w mut W>,
    ) -> Result<(), Error> {
        let targets = match windows.len() {
            0 => None,
            _ => Some(windows),
        };

        if self.renderer().is_none() {
            let renderer = Renderer::new(RenderOptions {
                force_software_rendering: self.options.force_software_rendering,
                power_preference: self.options.power_preference,
                device_limits: self.options.device_limits,
                targets,
            })
            .await?;
            *self.renderer() = Some(renderer);
        }

        Ok(())
    }
}
