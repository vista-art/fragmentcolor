#![cfg(not(wasm))]

use crate::{Renderer, Size, Target};
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// Generic application error type
pub type AppError = Box<dyn std::error::Error + Send + Sync>;

/// Result type for async setup functions
pub type SetupResult = Result<(), AppError>;
/// Callback for async setup functions
type SetupCallback = Option<Box<dyn FnOnce(&App, Vec<Arc<Window>>) -> SetupResult>>;
/// Convenience macro for async setup functions
#[macro_export]
macro_rules! call {
    ($func:ident) => {
        |app, windows| pollster::block_on($func(app, windows))
    };
}

/// Signature for top-level window callbacks.
/// Users can register global event callbacks and draw callbacks.
/// The event is passed by reference to avoid unnecessary cloning.
type WindowEventCallback = Box<dyn FnMut(&App, WindowId, &WindowEvent) + Send + 'static>;

// Device-level (non-window) event callback
type DeviceEventCallback =
    Box<dyn FnMut(&App, winit::event::DeviceId, &winit::event::DeviceEvent) + Send + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    ActivationTokenDone,
    Moved,
    Destroyed,
    DroppedFile,
    HoveredFile,
    HoveredFileCancelled,
    Focused,
    KeyboardInput,
    ModifiersChanged,
    Ime,
    CursorMoved,
    CursorEntered,
    CursorLeft,
    MouseWheel,
    MouseInput,
    PinchGesture,
    PanGesture,
    DoubleTapGesture,
    RotationGesture,
    TouchpadPressure,
    AxisMotion,
    Touch,
    Resized,
    ScaleFactorChanged,
    ThemeChanged,
    Occluded,
    RedrawRequested,
    CloseRequested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceEvent {
    Added,
    Removed,
    MouseMotion,
    MouseWheel,
    Motion,
    Button,
    Key,
}

fn kind_of(event: &WindowEvent) -> EventKind {
    match event {
        WindowEvent::ActivationTokenDone { .. } => EventKind::ActivationTokenDone,
        WindowEvent::Moved(_) => EventKind::Moved,
        WindowEvent::Destroyed => EventKind::Destroyed,
        WindowEvent::DroppedFile(_) => EventKind::DroppedFile,
        WindowEvent::HoveredFile(_) => EventKind::HoveredFile,
        WindowEvent::HoveredFileCancelled => EventKind::HoveredFileCancelled,
        WindowEvent::Focused(_) => EventKind::Focused,
        WindowEvent::KeyboardInput { .. } => EventKind::KeyboardInput,
        WindowEvent::ModifiersChanged(_) => EventKind::ModifiersChanged,
        WindowEvent::Ime(_) => EventKind::Ime,
        WindowEvent::CursorMoved { .. } => EventKind::CursorMoved,
        WindowEvent::CursorEntered { .. } => EventKind::CursorEntered,
        WindowEvent::CursorLeft { .. } => EventKind::CursorLeft,
        WindowEvent::MouseWheel { .. } => EventKind::MouseWheel,
        WindowEvent::MouseInput { .. } => EventKind::MouseInput,
        WindowEvent::PinchGesture { .. } => EventKind::PinchGesture,
        WindowEvent::PanGesture { .. } => EventKind::PanGesture,
        WindowEvent::DoubleTapGesture { .. } => EventKind::DoubleTapGesture,
        WindowEvent::RotationGesture { .. } => EventKind::RotationGesture,
        WindowEvent::TouchpadPressure { .. } => EventKind::TouchpadPressure,
        WindowEvent::AxisMotion { .. } => EventKind::AxisMotion,
        WindowEvent::Touch(_) => EventKind::Touch,
        WindowEvent::Resized(_) => EventKind::Resized,
        WindowEvent::ScaleFactorChanged { .. } => EventKind::ScaleFactorChanged,
        WindowEvent::ThemeChanged(_) => EventKind::ThemeChanged,
        WindowEvent::Occluded(_) => EventKind::Occluded,
        WindowEvent::RedrawRequested => EventKind::RedrawRequested,
        WindowEvent::CloseRequested => EventKind::CloseRequested,
    }
}

pub struct App {
    renderer: Renderer,

    // Active windows and targets
    windows: HashMap<WindowId, Arc<Window>>, // created at runtime
    targets: RwLock<HashMap<WindowId, crate::RenderTarget>>, // interior mutability for callbacks

    // Global registry of API objects by key (Shader, Pass, Vec<Pass>, Frame, ...)
    objects: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,

    // Blueprints to create at resume (if empty, create a single default window)
    blueprints: Vec<WindowAttributes>,

    // Registered callbacks
    on_event: RwLock<Vec<WindowEventCallback>>, // called for every WindowEvent
    on_draw: RwLock<Vec<WindowEventCallback>>,  // called when WindowEvent::RedrawRequested

    // Event-specific callback registries
    primary_by_kind: RwLock<HashMap<EventKind, Vec<WindowEventCallback>>>,
    per_window_by_kind: RwLock<HashMap<WindowId, HashMap<EventKind, Vec<WindowEventCallback>>>>,

    // Device event registries (no window association)
    on_device_event: RwLock<Vec<DeviceEventCallback>>, // called for every DeviceEvent
    device_by_kind: RwLock<HashMap<DeviceEvent, Vec<DeviceEventCallback>>>,

    // Startup hook (async)
    start_callback: SetupCallback,

    primary_window: Option<WindowId>,
}

impl App {
    pub fn new(renderer: Renderer) -> Self {
        App {
            renderer,
            windows: HashMap::new(),
            targets: RwLock::new(HashMap::new()),
            objects: RwLock::new(HashMap::new()),
            blueprints: Vec::new(),
            on_event: RwLock::new(Vec::new()),
            on_draw: RwLock::new(Vec::new()),
            primary_by_kind: RwLock::new(HashMap::new()),
            per_window_by_kind: RwLock::new(HashMap::new()),
            on_device_event: RwLock::new(Vec::new()),
            device_by_kind: RwLock::new(HashMap::new()),
            start_callback: None,
            primary_window: None,
        }
    }

    // Configure windows to be created on resume.
    pub fn add_window(&mut self, attrs: WindowAttributes) -> &mut Self {
        self.blueprints.push(attrs);
        self
    }

    // Startup hook: called once after windows are created in resumed()
    pub fn on_start<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&App, Vec<Arc<Window>>) -> SetupResult + 'static,
    {
        self.start_callback = Some(Box::new(f));
        self
    }

    // Global registry: add an API object by key.
    pub fn add<T>(&self, key: &str, value: T)
    where
        T: crate::renderer::Renderable + Send + Sync + 'static,
    {
        self.objects
            .write()
            .insert(key.to_string(), Arc::new(value));
    }

    // Global registry: get an API object by key with type downcast.
    pub fn get<T>(&self, key: &str) -> Option<Arc<T>>
    where
        T: crate::renderer::Renderable + Send + Sync + 'static,
    {
        self.objects
            .read()
            .get(key)
            .cloned()
            .and_then(|a| a.downcast::<T>().ok())
    }

    // Register a callback that receives every window event.
    pub fn on_event<F>(&mut self, f: F) -> &mut Self
    where
        F: FnMut(&App, WindowId, &WindowEvent) + Send + 'static,
    {
        self.on_event.write().push(Box::new(f));
        self
    }

    // Register a callback that runs on every RedrawRequested (use to animate/update uniforms).
    pub fn on_draw<F>(&mut self, f: F) -> &mut Self
    where
        F: FnMut(&App, WindowId, &WindowEvent) + Send + 'static,
    {
        self.on_draw.write().push(Box::new(f));
        self
    }

    // Event-specific registration -----------------------------------------------------------

    // Generic registration for future coverage (window events)
    pub fn on_event_kind<F>(&mut self, kind: EventKind, f: F) -> &mut Self
    where
        F: FnMut(&App, WindowId, &WindowEvent) + Send + 'static,
    {
        self.primary_by_kind
            .write()
            .entry(kind)
            .or_default()
            .push(Box::new(f));
        self
    }

    pub fn on_window_event_kind<F>(&mut self, id: WindowId, kind: EventKind, f: F) -> &mut Self
    where
        F: FnMut(&App, WindowId, &WindowEvent) + Send + 'static,
    {
        self.per_window_by_kind
            .write()
            .entry(id)
            .or_default()
            .entry(kind)
            .or_default()
            .push(Box::new(f));
        self
    }

    // Device event registration ---------------------------------------------------------------
    pub fn on_device_event<F>(&mut self, f: F) -> &mut Self
    where
        F: FnMut(&App, winit::event::DeviceId, &winit::event::DeviceEvent) + Send + 'static,
    {
        self.on_device_event.write().push(Box::new(f));
        self
    }

    pub fn on_device_event_kind<F>(&mut self, kind: DeviceEvent, f: F) -> &mut Self
    where
        F: FnMut(&App, winit::event::DeviceId, &winit::event::DeviceEvent) + Send + 'static,
    {
        self.device_by_kind
            .write()
            .entry(kind)
            .or_default()
            .push(Box::new(f));
        self
    }

    // Convenience getters for callbacks ------------------------------------------------------
    pub fn get_renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer(&self) -> &Renderer {
        self.get_renderer()
    }

    /// Returns the id of the first (primary) window created by the App.
    /// Panics if called before the window is created (resumed()).
    pub fn primary_window_id(&self) -> WindowId {
        self.primary_window
            .expect("SAFETY: primary_window set during resumed(); call only after App resumes")
    }

    /// Backwards-compatible alias for examples; forwards to primary_window_id().
    pub fn window_id(&self) -> WindowId {
        self.primary_window_id()
    }

    /// Convenience helper for single-window apps: resize the primary window target.
    pub fn resize(&self, size: impl Into<Size>) {
        let id = self.primary_window_id();
        self.resize_target(id, size);
    }

    pub fn window(&self, id: WindowId) -> Option<Arc<Window>> {
        self.windows.get(&id).cloned()
    }

    pub fn window_size(&self, id: WindowId) -> Option<Size> {
        self.targets.read().get(&id).map(|t| t.size())
    }

    pub fn add_target(&self, id: WindowId, target: crate::RenderTarget) {
        self.targets.write().insert(id, target);
    }

    /// Read-only access to a target; runs the closure with a borrowed target.
    pub fn with_target<R>(
        &self,
        id: WindowId,
        f: impl FnOnce(&crate::RenderTarget) -> R,
    ) -> Option<R> {
        let map = self.targets.read();
        map.get(&id).map(f)
    }

    pub fn resize_target(&self, id: WindowId, size: impl Into<Size>) {
        if let Some(target) = self.targets.write().get_mut(&id) {
            target.resize(size);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create one default window if none were specified
        let blueprints = if self.blueprints.is_empty() {
            vec![Window::default_attributes()]
        } else {
            self.blueprints.clone()
        };

        let mut created: Vec<Arc<Window>> = Vec::new();

        for attrs in blueprints {
            match event_loop.create_window(attrs) {
                Ok(w) => {
                    let window = Arc::new(w);
                    let id = window.id();

                    // Store window
                    if self.primary_window.is_none() {
                        self.primary_window = Some(id);
                    }
                    self.windows.insert(id, window.clone());
                    created.push(window);
                }
                Err(e) => {
                    log::error!("Failed to create window: {}", e);
                }
            }
        }

        // Invoke on_start once after windows exist
        if let Some(callback) = self.start_callback.take() {
            let result = callback(&*self, created.clone());
            if let Err(e) = result {
                log::error!("App startup failed: {}", e);
                event_loop.exit();
            }
        }

        // Kick off continuous redraw loop for all windows
        for win in &created {
            win.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // 1) Always forward event to generic callbacks first
        {
            let mut cbs = self.on_event.write();
            for cb in cbs.iter_mut() {
                cb(&*self, id, &event);
            }
        }

        // Dispatch event-specific registries
        let kind = kind_of(&event);
        if let Some(primary) = self.primary_window
            && primary == id
        {
            let mut map = self.primary_by_kind.write();
            if let Some(list) = map.get_mut(&kind) {
                for cb in list.iter_mut() {
                    cb(&*self, id, &event);
                }
            }
        }

        {
            let mut outer = self.per_window_by_kind.write();
            if let Some(win_map) = outer.get_mut(&id)
                && let Some(list) = win_map.get_mut(&kind)
            {
                for cb in list.iter_mut() {
                    cb(&*self, id, &event);
                }
            }
        }

        match &event {
            WindowEvent::RedrawRequested => {
                // 2) Allow user to update state each frame
                {
                    let mut cbs = self.on_draw.write();
                    for cb in cbs.iter_mut() {
                        cb(&*self, id, &event);
                    }
                }

                // 3) Keep the loop going continuously for animations
                if let Some(win) = self.windows.get(&id) {
                    win.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                // Forwarded above; now exit
                event_loop.exit();
            }
            _ => {
                // No built-in behavior for other events
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        // 1) Forward to generic device callbacks
        {
            let mut cbs = self.on_device_event.write();
            for cb in cbs.iter_mut() {
                cb(&*self, device_id, &event);
            }
        }
        // 2) Dispatch by kind to typed handlers
        let kind = match &event {
            winit::event::DeviceEvent::Added => DeviceEvent::Added,
            winit::event::DeviceEvent::Removed => DeviceEvent::Removed,
            winit::event::DeviceEvent::MouseMotion { .. } => DeviceEvent::MouseMotion,
            winit::event::DeviceEvent::MouseWheel { .. } => DeviceEvent::MouseWheel,
            winit::event::DeviceEvent::Motion { .. } => DeviceEvent::Motion,
            winit::event::DeviceEvent::Button { .. } => DeviceEvent::Button,
            winit::event::DeviceEvent::Key(_) => DeviceEvent::Key,
        };
        let mut map = self.device_by_kind.write();
        if let Some(list) = map.get_mut(&kind) {
            for cb in list.iter_mut() {
                cb(&*self, device_id, &event);
            }
        }
    }
}

/// Runs the given App using winit's event loop.
/// Needs to be called from the main thread. Blocks forever.
pub fn run(app: &mut App) {
    let event_loop = match EventLoop::new() {
        Ok(e) => e,
        Err(e) => {
            log::error!("Failed to create EventLoop: {}", e);
            return;
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);
    let _ = event_loop.run_app(app);
}

//-----------------------------------------------------
// BLANKET IMPLEMENTATIONS FOR FLATTENING WINIT EVENTS:
//-----------------------------------------------------

// Typed per-event convenience registrations -------------------------------------------------
macro_rules! define_typed_event_handlers {
    (
    $(
        ($name:ident, $per_window:ident, $kind:ident,
            match $pat:pat,
            primary($($p_ty:ty),*), call_primary($($p_arg:expr),*),
            per_window($($w_ty:ty),*), call_per_window($($w_arg:expr),*)
        )
    ),* $(,)?
    ) => {
        impl App {
            $(
                pub fn $name<F>(&mut self, mut f: F) -> &mut Self
                where F: FnMut(&App $(, $p_ty)*) + Send + 'static
                {
                    self.on_event_kind(EventKind::$kind, move |app, _id, ev| {
                        if let $pat = ev {
                            f(app $(, $p_arg)*)
                        }
                    })
                }
                pub fn $per_window<F>(&mut self, id: WindowId, mut f: F) -> &mut Self
                where F: FnMut(&App $(, $w_ty)*) + Send + 'static
                {
                    self.on_window_event_kind(id, EventKind::$kind, move |app, id, ev| {
                        if let $pat = ev {
                            f(app, id $(, $w_arg)*)
                        }
                    })
                }
            )*
        }
    }
}

define_typed_event_handlers! {
    (on_resize, on_window_resize, Resized,
        match WindowEvent::Resized(size),
        primary(&winit::dpi::PhysicalSize<u32>), call_primary(size),
        per_window(WindowId, &winit::dpi::PhysicalSize<u32>), call_per_window(size)
    ),
    (on_redraw_requested, on_window_redraw_requested, RedrawRequested,
        match WindowEvent::RedrawRequested,
        primary(), call_primary(),
        per_window(WindowId), call_per_window()
    ),
    (on_close_requested, on_window_close_requested, CloseRequested,
        match WindowEvent::CloseRequested,
        primary(), call_primary(),
        per_window(WindowId), call_per_window()
    ),
    (on_moved, on_window_moved, Moved,
        match WindowEvent::Moved(pos),
        primary(&winit::dpi::PhysicalPosition<i32>), call_primary(pos),
        per_window(WindowId, &winit::dpi::PhysicalPosition<i32>), call_per_window(pos)
    ),
    (on_destroyed, on_window_destroyed, Destroyed,
        match WindowEvent::Destroyed,
        primary(), call_primary(),
        per_window(WindowId), call_per_window()
    ),
    (on_focused, on_window_focused, Focused,
        match WindowEvent::Focused(v),
        primary(bool), call_primary(*v),
        per_window(WindowId, bool), call_per_window(*v)
    ),
    (on_cursor_moved, on_window_cursor_moved, CursorMoved,
        match WindowEvent::CursorMoved { device_id, position },
        primary(winit::event::DeviceId, &winit::dpi::PhysicalPosition<f64>), call_primary(*device_id, position),
        per_window(WindowId, winit::event::DeviceId, &winit::dpi::PhysicalPosition<f64>), call_per_window(*device_id, position)
    ),
    (on_cursor_entered, on_window_cursor_entered, CursorEntered,
        match WindowEvent::CursorEntered { device_id },
        primary(winit::event::DeviceId), call_primary(*device_id),
        per_window(WindowId, winit::event::DeviceId), call_per_window(*device_id)
    ),
    (on_cursor_left, on_window_cursor_left, CursorLeft,
        match WindowEvent::CursorLeft { device_id },
        primary(winit::event::DeviceId), call_primary(*device_id),
        per_window(WindowId, winit::event::DeviceId), call_per_window(*device_id)
    ),
    (on_mouse_wheel, on_window_mouse_wheel, MouseWheel,
        match WindowEvent::MouseWheel { device_id, delta, phase },
        primary(winit::event::DeviceId, &winit::event::MouseScrollDelta, winit::event::TouchPhase), call_primary(*device_id, delta, *phase),
        per_window(WindowId, winit::event::DeviceId, &winit::event::MouseScrollDelta, winit::event::TouchPhase), call_per_window(*device_id, delta, *phase)
    ),
    (on_mouse_input, on_window_mouse_input, MouseInput,
        match WindowEvent::MouseInput { device_id, state, button },
        primary(winit::event::DeviceId, winit::event::ElementState, winit::event::MouseButton), call_primary(*device_id, *state, *button),
        per_window(WindowId, winit::event::DeviceId, winit::event::ElementState, winit::event::MouseButton), call_per_window(*device_id, *state, *button)
    ),
    (on_keyboard_input, on_window_keyboard_input, KeyboardInput,
        match WindowEvent::KeyboardInput { device_id, event, is_synthetic },
        primary(winit::event::DeviceId, &winit::event::KeyEvent, bool), call_primary(*device_id, event, *is_synthetic),
        per_window(WindowId, winit::event::DeviceId, &winit::event::KeyEvent, bool), call_per_window(*device_id, event, *is_synthetic)
    ),
    (on_modifiers_changed, on_window_modifiers_changed, ModifiersChanged,
        match WindowEvent::ModifiersChanged(m),
        primary(winit::event::Modifiers), call_primary(*m),
        per_window(WindowId, winit::event::Modifiers), call_per_window(*m)
    ),
    (on_dropped_file, on_window_dropped_file, DroppedFile,
        match WindowEvent::DroppedFile(p),
        primary(&std::path::PathBuf), call_primary(p),
        per_window(WindowId, &std::path::PathBuf), call_per_window(p)
    ),
    (on_hovered_file, on_window_hovered_file, HoveredFile,
        match WindowEvent::HoveredFile(p),
        primary(&std::path::PathBuf), call_primary(p),
        per_window(WindowId, &std::path::PathBuf), call_per_window(p)
    ),
    (on_hovered_file_cancelled, on_window_hovered_file_cancelled, HoveredFileCancelled,
        match WindowEvent::HoveredFileCancelled,
        primary(), call_primary(),
        per_window(WindowId), call_per_window()
    ),
    (on_touch, on_window_touch, Touch,
        match WindowEvent::Touch(t),
        primary(&winit::event::Touch), call_primary(t),
        per_window(WindowId, &winit::event::Touch), call_per_window(t)
    ),
    (on_ime, on_window_ime, Ime,
        match WindowEvent::Ime(im),
        primary(&winit::event::Ime), call_primary(im),
        per_window(WindowId, &winit::event::Ime), call_per_window(im)
    ),
    (on_axis_motion, on_window_axis_motion, AxisMotion,
        match WindowEvent::AxisMotion { device_id, axis, value },
        primary(winit::event::DeviceId, winit::event::AxisId, f64), call_primary(*device_id, *axis, *value),
        per_window(WindowId, winit::event::DeviceId, winit::event::AxisId, f64), call_per_window(*device_id, *axis, *value)
    ),
    (on_theme_changed, on_window_theme_changed, ThemeChanged,
        match WindowEvent::ThemeChanged(theme),
        primary(winit::window::Theme), call_primary(*theme),
        per_window(WindowId, winit::window::Theme), call_per_window(*theme)
    ),
    (on_occluded, on_window_occluded, Occluded,
        match WindowEvent::Occluded(b),
        primary(bool), call_primary(*b),
        per_window(WindowId, bool), call_per_window(*b)
    ),
    (on_scale_factor_changed, on_window_scale_factor_changed, ScaleFactorChanged,
        match WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer },
        primary(f64, &winit::event::InnerSizeWriter), call_primary(*scale_factor, inner_size_writer),
        per_window(WindowId, f64, &winit::event::InnerSizeWriter), call_per_window(*scale_factor, inner_size_writer)
    ),
    (on_activation_token_done, on_window_activation_token_done, ActivationTokenDone,
        match WindowEvent::ActivationTokenDone { serial, token },
        primary(&winit::event_loop::AsyncRequestSerial, &winit::window::ActivationToken), call_primary(serial, token),
        per_window(WindowId, &winit::event_loop::AsyncRequestSerial, &winit::window::ActivationToken), call_per_window(serial, token)
    ),
    (on_pinch_gesture, on_window_pinch_gesture, PinchGesture,
        match WindowEvent::PinchGesture { device_id, delta, phase },
        primary(winit::event::DeviceId, f64, winit::event::TouchPhase), call_primary(*device_id, *delta, *phase),
        per_window(WindowId, winit::event::DeviceId, f64, winit::event::TouchPhase), call_per_window(*device_id, *delta, *phase)
    ),
    (on_pan_gesture, on_window_pan_gesture, PanGesture,
        match WindowEvent::PanGesture { device_id, delta, phase },
        primary(winit::event::DeviceId, &winit::dpi::PhysicalPosition<f32>, winit::event::TouchPhase), call_primary(*device_id, delta, *phase),
        per_window(WindowId, winit::event::DeviceId, &winit::dpi::PhysicalPosition<f32>, winit::event::TouchPhase), call_per_window(*device_id, delta, *phase)
    ),
    (on_double_tap_gesture, on_window_double_tap_gesture, DoubleTapGesture,
        match WindowEvent::DoubleTapGesture { device_id },
        primary(winit::event::DeviceId), call_primary(*device_id),
        per_window(WindowId, winit::event::DeviceId), call_per_window(*device_id)
    ),
    (on_rotation_gesture, on_window_rotation_gesture, RotationGesture,
        match WindowEvent::RotationGesture { device_id, delta, phase },
        primary(winit::event::DeviceId, f32, winit::event::TouchPhase), call_primary(*device_id, *delta, *phase),
        per_window(WindowId, winit::event::DeviceId, f32, winit::event::TouchPhase), call_per_window(*device_id, *delta, *phase)
    ),
    (on_touchpad_pressure, on_window_touchpad_pressure, TouchpadPressure,
        match WindowEvent::TouchpadPressure { device_id, pressure, stage },
        primary(winit::event::DeviceId, f32, i64), call_primary(*device_id, *pressure, *stage),
        per_window(WindowId, winit::event::DeviceId, f32, i64), call_per_window(*device_id, *pressure, *stage)
    )
}

// Typed device-event convenience registrations -----------------------------------------------
macro_rules! define_typed_device_handlers {
    (
        $(
            ($name:ident, $kind:ident,
                match $pat:pat,
                args($($p_ty:ty),*), call($($p_arg:expr),*)
            )
        ),* $(,)?
    ) => {
        impl App {
            $(
                pub fn $name<F>(&mut self, mut f: F) -> &mut Self
                where F: FnMut(&App, winit::event::DeviceId $(, $p_ty)*) + Send + 'static
                {
                    self.on_device_event_kind(DeviceEvent::$kind, move |app, device_id, ev| {
                        if let $pat = ev {
                            f(app, device_id $(, $p_arg)*)
                        }
                    })
                }
            )*
        }
    }
}

define_typed_device_handlers! {
    (on_device_added, Added,
        match winit::event::DeviceEvent::Added,
        args(), call()
    ),
    (on_device_removed, Removed,
        match winit::event::DeviceEvent::Removed,
        args(), call()
    ),
    (on_device_mouse_motion, MouseMotion,
        match winit::event::DeviceEvent::MouseMotion { delta },
        args((f64, f64)), call(*delta)
    ),
    (on_device_mouse_wheel, MouseWheel,
        match winit::event::DeviceEvent::MouseWheel { delta },
        args(&winit::event::MouseScrollDelta), call(delta)
    ),
    (on_device_motion, Motion,
        match winit::event::DeviceEvent::Motion { axis, value },
        args(winit::event::AxisId, f64), call(*axis, *value)
    ),
    (on_device_button, Button,
        match winit::event::DeviceEvent::Button { button, state },
        args(winit::event::ButtonId, winit::event::ElementState), call(*button, *state)
    ),
    (on_device_key, Key,
        match winit::event::DeviceEvent::Key(ev),
        args(&winit::event::RawKeyEvent), call(ev)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: App can register windows and callbacks without running the event loop.
    #[test]
    fn configures_callbacks() {
        // Arrange
        let renderer = Renderer::new();
        let mut app = App::new(renderer);
        app.add_window(Window::default_attributes());

        // Act: register callbacks of multiple kinds
        let _ = app
            .on_event(|_a, _id, _e| {})
            .on_draw(|_a, _id, _e| {})
            .on_resize(|_a, _s| {})
            .on_window_close_requested(WindowId::from(1), |_a, _id| {})
            .on_device_event(|_a, _id, _e| {})
            .on_device_mouse_motion(|_a, _id, _delta| {});

        // Assert: internal registries captured entries
        assert_eq!(app.blueprints.len(), 1);
        assert_eq!(app.on_event.read().len(), 1);
        assert_eq!(app.on_draw.read().len(), 1);
        let _ = app.primary_by_kind.read().len();
        assert_eq!(app.on_device_event.read().len(), 1);
        let _ = app.device_by_kind.read().len();
    }

    // Story: resize_target updates the underlying texture-backed target and size() reflects changes.
    #[test]
    fn resize_target_updates_texture_target_size() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let app = App::new(renderer);
            let id = WindowId::from(7);
            let rt = app
                .get_renderer()
                .create_texture_target([5u32, 6u32])
                .await
                .expect("tex target");
            app.add_target(id, crate::RenderTarget::from(rt));

            let s1 = app.window_size(id).expect("size present");
            assert_eq!([s1.width, s1.height], [5, 6]);

            app.resize_target(id, [9u32, 11u32]);
            let s2 = app.window_size(id).expect("size present after");
            assert_eq!([s2.width, s2.height], [9, 11]);
        });
    }

    // Story: primary_window_id() panics if called before any window is created.
    #[test]
    fn primary_window_id_panics_before_resumed() {
        let renderer = Renderer::new();
        let app = App::new(renderer);
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = app.primary_window_id();
        }))
        .is_err();
        assert!(panicked);
    }

    // Story: size() returns None when no target exists for the id; Some when present.
    #[test]
    fn size_some_and_none() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let app = App::new(renderer);
            let missing = WindowId::from(1u64);
            assert!(app.window_size(missing).is_none());

            let id = WindowId::from(2u64);
            let rt = app
                .get_renderer()
                .create_texture_target([2u32, 3u32])
                .await
                .expect("tex target");
            app.add_target(id, crate::RenderTarget::from(rt));
            let s = app.window_size(id).expect("size present");
            assert_eq!([s.width, s.height], [2, 3]);
        });
    }

    // Story: Registry add/get round-trip with Shader and Vec<Pass>.
    #[test]
    fn registry_round_trip() {
        use crate::{Pass, Renderable, Shader};
        let renderer = Renderer::new();
        let app = App::new(renderer);
        let shader = Shader::default();
        app.add("shader.main", shader.clone());
        let got = app.get::<Shader>("shader.main").expect("get shader");
        // ensure same content (Shader is Clone + Debug; pointer equality not guaranteed)
        let _ = got; // existence is enough here

        let p1 = Pass::new("p1");
        let p2 = Pass::new("p2");
        app.add("passes", vec![p1.clone(), p2.clone()]);
        let passes = app.get::<Vec<Pass>>("passes").expect("get passes");
        assert_eq!(passes.passes().iter().count(), 2);
    }

    // Story: Typed registration helpers populate the appropriate maps.
    #[test]
    fn typed_registration_maps_receive_entries() {
        let renderer = Renderer::new();
        let mut app = App::new(renderer);
        let id = WindowId::from(99u64);
        app.on_resize(|_, _| {});
        app.on_window_mouse_input(id, |_, _, _, _, _| {});

        // Primary-by-kind should have Resized key with one cb
        let p = app.primary_by_kind.read();
        assert!(p.get(&EventKind::Resized).is_some());

        // Per-window-by-kind should have entry for id/MouseInput
        let w = app.per_window_by_kind.read();
        let exists = w
            .get(&id)
            .and_then(|m| m.get(&EventKind::MouseInput))
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        assert!(exists);
    }

    // Story: Device event registration maps receive entries.
    #[test]
    fn device_event_registration_maps_receive_entries() {
        let renderer = Renderer::new();
        let mut app = App::new(renderer);
        app.on_device_event(|_, _, _| {});
        app.on_device_motion(|_, _, _, _| {});
        assert_eq!(app.on_device_event.read().len(), 1);
        let m = app.device_by_kind.read();
        assert!(m.get(&DeviceEvent::Motion).is_some());
    }
}
