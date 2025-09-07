use crate::{
    Renderer, Shader, Size, Target, WindowTarget, error::ShaderError, frame::Frame, pass::Pass,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

// Scene is a light wrapper to allow using Shader, Pass, or Frame as the renderable for a window.
pub enum Scene {
    Shader(Shader),
    Pass(Pass),
    Frame(Frame),
}

impl From<Shader> for Scene {
    fn from(s: Shader) -> Self {
        Scene::Shader(s)
    }
}
impl From<Pass> for Scene {
    fn from(p: Pass) -> Self {
        Scene::Pass(p)
    }
}
impl From<Frame> for Scene {
    fn from(f: Frame) -> Self {
        Scene::Frame(f)
    }
}

impl crate::renderer::Renderable for Scene {
    fn passes(&self) -> impl IntoIterator<Item = &crate::PassObject> {
        match self {
            Scene::Shader(s) => vec![s.pass.as_ref()],
            Scene::Pass(p) => vec![p.object.as_ref()],
            Scene::Frame(f) => f.passes.iter().map(|p| p.as_ref()).collect::<Vec<_>>(),
        }
    }
}

// Signature for top-level callbacks.
// Users can register global event callbacks and draw callbacks.
// The event is passed by reference to avoid unnecessary cloning.
type EventCb = Box<dyn FnMut(&App, WindowId, &WindowEvent) + Send + 'static>;

type SceneRef = Arc<Scene>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WinEvtKind {
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

fn kind_of(event: &WindowEvent) -> WinEvtKind {
    match event {
        WindowEvent::ActivationTokenDone { .. } => WinEvtKind::ActivationTokenDone,
        WindowEvent::Moved(_) => WinEvtKind::Moved,
        WindowEvent::Destroyed => WinEvtKind::Destroyed,
        WindowEvent::DroppedFile(_) => WinEvtKind::DroppedFile,
        WindowEvent::HoveredFile(_) => WinEvtKind::HoveredFile,
        WindowEvent::HoveredFileCancelled => WinEvtKind::HoveredFileCancelled,
        WindowEvent::Focused(_) => WinEvtKind::Focused,
        WindowEvent::KeyboardInput { .. } => WinEvtKind::KeyboardInput,
        WindowEvent::ModifiersChanged(_) => WinEvtKind::ModifiersChanged,
        WindowEvent::Ime(_) => WinEvtKind::Ime,
        WindowEvent::CursorMoved { .. } => WinEvtKind::CursorMoved,
        WindowEvent::CursorEntered { .. } => WinEvtKind::CursorEntered,
        WindowEvent::CursorLeft { .. } => WinEvtKind::CursorLeft,
        WindowEvent::MouseWheel { .. } => WinEvtKind::MouseWheel,
        WindowEvent::MouseInput { .. } => WinEvtKind::MouseInput,
        WindowEvent::PinchGesture { .. } => WinEvtKind::PinchGesture,
        WindowEvent::PanGesture { .. } => WinEvtKind::PanGesture,
        WindowEvent::DoubleTapGesture { .. } => WinEvtKind::DoubleTapGesture,
        WindowEvent::RotationGesture { .. } => WinEvtKind::RotationGesture,
        WindowEvent::TouchpadPressure { .. } => WinEvtKind::TouchpadPressure,
        WindowEvent::AxisMotion { .. } => WinEvtKind::AxisMotion,
        WindowEvent::Touch(_) => WinEvtKind::Touch,
        WindowEvent::Resized(_) => WinEvtKind::Resized,
        WindowEvent::ScaleFactorChanged { .. } => WinEvtKind::ScaleFactorChanged,
        WindowEvent::ThemeChanged(_) => WinEvtKind::ThemeChanged,
        WindowEvent::Occluded(_) => WinEvtKind::Occluded,
        WindowEvent::RedrawRequested => WinEvtKind::RedrawRequested,
        WindowEvent::CloseRequested => WinEvtKind::CloseRequested,
    }
}

pub struct App {
    renderer: Renderer,

    // Active windows and targets
    windows: HashMap<WindowId, Arc<Window>>, // created at runtime
    targets: RwLock<HashMap<WindowId, WindowTarget>>, // interior mutability for callbacks

    // Per-window scene to render
    scenes: RwLock<HashMap<WindowId, SceneRef>>,

    // Blueprints to create at resume (if empty, create a single default window)
    blueprints: Vec<WindowAttributes>,

    // Default scene to assign to newly created windows (shared across windows)
    default_scene: Option<SceneRef>,

    // Registered callbacks
    on_event: RwLock<Vec<EventCb>>, // called for every WindowEvent
    on_draw: RwLock<Vec<EventCb>>,  // called when WindowEvent::RedrawRequested

    // Event-specific callback registries
    primary_by_kind: RwLock<HashMap<WinEvtKind, Vec<EventCb>>>,
    per_window_by_kind: RwLock<HashMap<WindowId, HashMap<WinEvtKind, Vec<EventCb>>>>,

    primary_window: Option<WindowId>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            renderer: Renderer::new(),
            windows: HashMap::new(),
            targets: RwLock::new(HashMap::new()),
            scenes: RwLock::new(HashMap::new()),
            blueprints: Vec::new(),
            default_scene: None,
            on_event: RwLock::new(Vec::new()),
            on_draw: RwLock::new(Vec::new()),

            primary_by_kind: RwLock::new(HashMap::new()),
            per_window_by_kind: RwLock::new(HashMap::new()),
            primary_window: None,
        }
    }

    // Configure windows to be created on resume.
    pub fn add_window(&mut self, attrs: WindowAttributes) -> &mut Self {
        self.blueprints.push(attrs);
        self
    }

    // Assign a default scene to all windows (shared by Arc).
    pub fn scene<S: Into<Scene>>(&mut self, scene: S) -> &mut Self {
        self.default_scene = Some(Arc::new(scene.into()));
        self
    }

    // Replace or set a scene for a specific window.
    pub fn set_scene<S: Into<Scene>>(&self, id: WindowId, scene: S) {
        self.scenes.write().insert(id, Arc::new(scene.into()));
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

    // Generic registration for future coverage
    pub fn on_event_kind<F>(&mut self, kind: WinEvtKind, f: F) -> &mut Self
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

    pub fn on_window_event_kind<F>(&mut self, id: WindowId, kind: WinEvtKind, f: F) -> &mut Self
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
}

// Typed per-event convenience registrations -------------------------------------------------
macro_rules! define_typed_event_handlers {
    (
      $(
        ($name:ident, $perwin:ident, $kind:ident,
         match $pat:pat,
         primary($($p_ty:ty),*), call_primary($($p_arg:expr),*),
         perwin($($w_ty:ty),*), call_perwin($($w_arg:expr),*)
        )
      ),* $(,)?
    ) => {
        impl App {
            $(
                pub fn $name<F>(&mut self, mut f: F) -> &mut Self
                where F: FnMut(&App $(, $p_ty)*) + Send + 'static
                {
                    self.on_event_kind(WinEvtKind::$kind, move |app, _id, ev| {
                        if let $pat = ev {
                            f(app $(, $p_arg)*)
                        }
                    })
                }
                pub fn $perwin<F>(&mut self, id: WindowId, mut f: F) -> &mut Self
                where F: FnMut(&App $(, $w_ty)*) + Send + 'static
                {
                    self.on_window_event_kind(id, WinEvtKind::$kind, move |app, id, ev| {
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
        perwin(WindowId, &winit::dpi::PhysicalSize<u32>), call_perwin(size)
    ),
    (on_redraw_requested, on_window_redraw_requested, RedrawRequested,
        match WindowEvent::RedrawRequested,
        primary(), call_primary(),
        perwin(WindowId), call_perwin()
    ),
    (on_close_requested, on_window_close_requested, CloseRequested,
        match WindowEvent::CloseRequested,
        primary(), call_primary(),
        perwin(WindowId), call_perwin()
    ),
    (on_moved, on_window_moved, Moved,
        match WindowEvent::Moved(pos),
        primary(&winit::dpi::PhysicalPosition<i32>), call_primary(pos),
        perwin(WindowId, &winit::dpi::PhysicalPosition<i32>), call_perwin(pos)
    ),
    (on_destroyed, on_window_destroyed, Destroyed,
        match WindowEvent::Destroyed,
        primary(), call_primary(),
        perwin(WindowId), call_perwin()
    ),
    (on_focused, on_window_focused, Focused,
        match WindowEvent::Focused(v),
        primary(bool), call_primary(*v),
        perwin(WindowId, bool), call_perwin(*v)
    ),
    (on_cursor_moved, on_window_cursor_moved, CursorMoved,
        match WindowEvent::CursorMoved { device_id, position },
        primary(winit::event::DeviceId, &winit::dpi::PhysicalPosition<f64>), call_primary(*device_id, position),
        perwin(WindowId, winit::event::DeviceId, &winit::dpi::PhysicalPosition<f64>), call_perwin(*device_id, position)
    ),
    (on_cursor_entered, on_window_cursor_entered, CursorEntered,
        match WindowEvent::CursorEntered { device_id },
        primary(winit::event::DeviceId), call_primary(*device_id),
        perwin(WindowId, winit::event::DeviceId), call_perwin(*device_id)
    ),
    (on_cursor_left, on_window_cursor_left, CursorLeft,
        match WindowEvent::CursorLeft { device_id },
        primary(winit::event::DeviceId), call_primary(*device_id),
        perwin(WindowId, winit::event::DeviceId), call_perwin(*device_id)
    ),
    (on_mouse_wheel, on_window_mouse_wheel, MouseWheel,
        match WindowEvent::MouseWheel { device_id, delta, phase },
        primary(winit::event::DeviceId, &winit::event::MouseScrollDelta, winit::event::TouchPhase), call_primary(*device_id, delta, *phase),
        perwin(WindowId, winit::event::DeviceId, &winit::event::MouseScrollDelta, winit::event::TouchPhase), call_perwin(*device_id, delta, *phase)
    ),
    (on_mouse_input, on_window_mouse_input, MouseInput,
        match WindowEvent::MouseInput { device_id, state, button },
        primary(winit::event::DeviceId, winit::event::ElementState, winit::event::MouseButton), call_primary(*device_id, *state, *button),
        perwin(WindowId, winit::event::DeviceId, winit::event::ElementState, winit::event::MouseButton), call_perwin(*device_id, *state, *button)
    ),
    (on_keyboard_input, on_window_keyboard_input, KeyboardInput,
        match WindowEvent::KeyboardInput { device_id, event, is_synthetic },
        primary(winit::event::DeviceId, &winit::event::KeyEvent, bool), call_primary(*device_id, event, *is_synthetic),
        perwin(WindowId, winit::event::DeviceId, &winit::event::KeyEvent, bool), call_perwin(*device_id, event, *is_synthetic)
    ),
    (on_modifiers_changed, on_window_modifiers_changed, ModifiersChanged,
        match WindowEvent::ModifiersChanged(m),
        primary(winit::event::Modifiers), call_primary(*m),
        perwin(WindowId, winit::event::Modifiers), call_perwin(*m)
    ),
    (on_dropped_file, on_window_dropped_file, DroppedFile,
        match WindowEvent::DroppedFile(p),
        primary(&std::path::PathBuf), call_primary(p),
        perwin(WindowId, &std::path::PathBuf), call_perwin(p)
    ),
    (on_hovered_file, on_window_hovered_file, HoveredFile,
        match WindowEvent::HoveredFile(p),
        primary(&std::path::PathBuf), call_primary(p),
        perwin(WindowId, &std::path::PathBuf), call_perwin(p)
    ),
    (on_hovered_file_cancelled, on_window_hovered_file_cancelled, HoveredFileCancelled,
        match WindowEvent::HoveredFileCancelled,
        primary(), call_primary(),
        perwin(WindowId), call_perwin()
    ),
    (on_touch, on_window_touch, Touch,
        match WindowEvent::Touch(t),
        primary(&winit::event::Touch), call_primary(t),
        perwin(WindowId, &winit::event::Touch), call_perwin(t)
    ),
    (on_ime, on_window_ime, Ime,
        match WindowEvent::Ime(im),
        primary(&winit::event::Ime), call_primary(im),
        perwin(WindowId, &winit::event::Ime), call_perwin(im)
    ),
    (on_axis_motion, on_window_axis_motion, AxisMotion,
        match WindowEvent::AxisMotion { device_id, axis, value },
        primary(winit::event::DeviceId, winit::event::AxisId, f64), call_primary(*device_id, *axis, *value),
        perwin(WindowId, winit::event::DeviceId, winit::event::AxisId, f64), call_perwin(*device_id, *axis, *value)
    ),
    (on_theme_changed, on_window_theme_changed, ThemeChanged,
        match WindowEvent::ThemeChanged(theme),
        primary(winit::window::Theme), call_primary(*theme),
        perwin(WindowId, winit::window::Theme), call_perwin(*theme)
    ),
    (on_occluded, on_window_occluded, Occluded,
        match WindowEvent::Occluded(b),
        primary(bool), call_primary(*b),
        perwin(WindowId, bool), call_perwin(*b)
    ),
    (on_scale_factor_changed, on_window_scale_factor_changed, ScaleFactorChanged,
        match WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer },
        primary(f64, &winit::event::InnerSizeWriter), call_primary(*scale_factor, inner_size_writer),
        perwin(WindowId, f64, &winit::event::InnerSizeWriter), call_perwin(*scale_factor, inner_size_writer)
    ),
    // Additional events per full WindowEvent definition
    (on_activation_token_done, on_window_activation_token_done, ActivationTokenDone,
        match WindowEvent::ActivationTokenDone { serial, token },
primary(&winit::event_loop::AsyncRequestSerial, &winit::window::ActivationToken), call_primary(serial, token),
perwin(WindowId, &winit::event_loop::AsyncRequestSerial, &winit::window::ActivationToken), call_perwin(serial, token)
    ),
    (on_pinch_gesture, on_window_pinch_gesture, PinchGesture,
        match WindowEvent::PinchGesture { device_id, delta, phase },
        primary(winit::event::DeviceId, f64, winit::event::TouchPhase), call_primary(*device_id, *delta, *phase),
        perwin(WindowId, winit::event::DeviceId, f64, winit::event::TouchPhase), call_perwin(*device_id, *delta, *phase)
    ),
    (on_pan_gesture, on_window_pan_gesture, PanGesture,
        match WindowEvent::PanGesture { device_id, delta, phase },
        primary(winit::event::DeviceId, &winit::dpi::PhysicalPosition<f32>, winit::event::TouchPhase), call_primary(*device_id, delta, *phase),
        perwin(WindowId, winit::event::DeviceId, &winit::dpi::PhysicalPosition<f32>, winit::event::TouchPhase), call_perwin(*device_id, delta, *phase)
    ),
    (on_double_tap_gesture, on_window_double_tap_gesture, DoubleTapGesture,
        match WindowEvent::DoubleTapGesture { device_id },
        primary(winit::event::DeviceId), call_primary(*device_id),
        perwin(WindowId, winit::event::DeviceId), call_perwin(*device_id)
    ),
    (on_rotation_gesture, on_window_rotation_gesture, RotationGesture,
        match WindowEvent::RotationGesture { device_id, delta, phase },
        primary(winit::event::DeviceId, f32, winit::event::TouchPhase), call_primary(*device_id, *delta, *phase),
        perwin(WindowId, winit::event::DeviceId, f32, winit::event::TouchPhase), call_perwin(*device_id, *delta, *phase)
    ),
    (on_touchpad_pressure, on_window_touchpad_pressure, TouchpadPressure,
        match WindowEvent::TouchpadPressure { device_id, pressure, stage },
        primary(winit::event::DeviceId, f32, i64), call_primary(*device_id, *pressure, *stage),
        perwin(WindowId, winit::event::DeviceId, f32, i64), call_perwin(*device_id, *pressure, *stage)
    )
}

impl App {
    // Convenience getters for callbacks ------------------------------------------------------
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Returns the id of the first (primary) window created by the App.
    /// Panics if called before the window is created (resumed()).
    pub fn window_id(&self) -> WindowId {
        self.primary_window
            .expect("window_id() called before App resumed")
    }

    /// Convenience helper for single-window apps: resize the primary window target.
    pub fn resize(&self, size: impl Into<Size>) {
        let id = self.window_id();
        self.resize_target(id, size);
    }

    pub fn window(&self, id: WindowId) -> Option<Arc<Window>> {
        self.windows.get(&id).cloned()
    }

    pub fn size(&self, id: WindowId) -> Option<Size> {
        self.targets.read().get(&id).map(|t| t.size())
    }

    pub fn resize_target(&self, id: WindowId, size: impl Into<Size>) {
        if let Some(tgt) = self.targets.write().get_mut(&id) {
            tgt.resize(size);
        }
    }

    // Set a uniform across the window's scene (Shader, Pass, or Frame).
    pub fn set_uniform<T: Into<crate::shader::UniformData> + Clone>(
        &self,
        id: WindowId,
        key: &str,
        value: T,
    ) -> Result<(), ShaderError> {
        let scenes = self.scenes.read();
        let scene = match scenes.get(&id) {
            Some(s) => s.clone(),
            None => return Ok(()), // no scene bound; treat as no-op
        };

        match &*scene {
            Scene::Shader(s) => s.object.set(key, value)?,
            Scene::Pass(p) => {
                for so in p.object.shaders.read().iter() {
                    let _ = so.set(key, value.clone());
                }
            }
            Scene::Frame(f) => {
                for pass in f.passes.iter() {
                    for so in pass.shaders.read().iter() {
                        let _ = so.set(key, value.clone());
                    }
                }
            }
        }
        Ok(())
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

                    // Create WindowTarget (async)
                    match pollster::block_on(self.renderer.create_target(window.clone())) {
                        Ok(target) => {
                            self.targets.write().insert(id, target);

                            // Assign default scene if provided
                            if let Some(scene) = &self.default_scene {
                                self.scenes.write().insert(id, Arc::clone(scene));
                            }

                            // Kick off continuous redraw loop
                            window.request_redraw();
                        }
                        Err(e) => {
                            log::error!("Failed to create target for window {:?}: {}", id, e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create window: {}", e);
                }
            }
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
                // 2) Allow user to update uniforms/state each frame
                {
                    let mut cbs = self.on_draw.write();
                    for cb in cbs.iter_mut() {
                        cb(&*self, id, &event);
                    }
                }

                // 3) Render the bound scene for this window (if any)
                {
                    let targets = self.targets.read();
                    let scenes = self.scenes.read();
                    if let (Some(target), Some(scene)) = (targets.get(&id), scenes.get(&id))
                        && let Err(err) = self.renderer.render(&**scene, target)
                    {
                        log::error!("Failed to render window {:?}: {}", id, err);
                    }
                }

                // 4) Keep the loop going continuously for animations
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
