use crate::{
    app::{events::Event, AppState, Container, EventListener, Window},
    renderer::{
        target::{RenderTarget, TargetId},
        RenderContext, RenderTargetCollection,
    },
    Quad,
};
use instant::{Duration, Instant};
use std::sync::{Arc, RwLock};
#[cfg(wasm)]
use winit::platform::web::EventLoopExtWebSys;
use winit::{
    event::{ElementState, Event as Winit, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{
        ControlFlow, EventLoop as WinitEventLoop, EventLoopBuilder, EventLoopProxy,
        EventLoopWindowTarget,
    },
};

// @TODO make this configurable
//       - we could have a target-specific frame rate
const TARGET_FRAME_TIME: f64 = 1.0 / 60.0;
const RUNNING: &str = "EventLoop not available: already running";

pub type EventLoopRunner = Box<dyn Runner>;
pub trait Runner: 'static + FnOnce(WinitEventLoop<Event>, Arc<RwLock<AppState>>) + Send {}
impl<F> Runner for F where F: 'static + FnOnce(WinitEventLoop<Event>, Arc<RwLock<AppState>>) + Send {}
impl std::fmt::Debug for dyn Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventLoopRunner")
    }
}

#[derive(Debug)]
pub struct EventLoop<T: 'static> {
    inner: Option<WinitEventLoop<T>>,
}

unsafe impl Send for EventLoop<Event> {}

impl EventLoop<Event> {
    pub fn new() -> Self {
        Self {
            inner: Some(EventLoopBuilder::<Event>::with_user_event().build()),
        }
    }

    pub fn window_target(&self) -> &EventLoopWindowTarget<Event> {
        self.inner.as_ref().expect(RUNNING)
    }

    pub fn create_dispatcher(&self) -> EventLoopProxy<Event> {
        let dispatcher = self.inner.as_ref().expect(RUNNING).create_proxy();
        dispatcher
    }

    pub async fn run(&mut self, runner: EventLoopRunner, app: Arc<RwLock<AppState>>) {
        let event_loop = self.inner.take().expect(RUNNING);
        runner(event_loop, app.clone())
    }
}

// Shorthand types for Winit's event handler arguments
type E<'a> = Winit<'a, Event>;
type W<'b> = &'b EventLoopWindowTarget<Event>;
type C<'c> = &'c mut ControlFlow;

pub fn run_event_loop(event_loop: WinitEventLoop<Event>, app: Arc<RwLock<AppState>>) {
    let mut last_update = Instant::now();

    let event_handler = Box::new(move |event: E, _elwt: W, control_flow: C| {
        let app = app.read().expect("Couldn't get AppState Read mutex lock");
        let renderer = app.renderer();

        match event {
            // Reserved for future use.
            //
            // Custom dispatched events as strings.
            Winit::UserEvent(event) => match event {
                _ => {}
            },

            // This variant represents anything that happens in a Window.
            Winit::WindowEvent {
                ref event,
                window_id,
            } => {
                let target_id = TargetId::Window(window_id);
                let windows = app.read_windows_collection::<Window>();
                let window = if let Some(window) = windows.get(&window_id) {
                    window
                } else {
                    return;
                };

                match event {
                    // The size of the window has changed.
                    // Contains the client area's new dimensions.
                    WindowEvent::Resized(physical_size) => {
                        let size = Quad::from_window_size(physical_size).to_wgpu_size();

                        if window.auto_resize {
                            let renderer = app.renderer();
                            let mut targets = renderer.write_targets();
                            let target = targets.get_mut(&target_id);
                            target
                                .is_some()
                                .then(|| target.unwrap().resize(&renderer, size));
                        }

                        window.call(
                            "resize",
                            Event::Resized {
                                width: physical_size.width,
                                height: physical_size.height,
                            },
                        )
                    }

                    // The window's scale factor has changed.
                    //
                    // The following user actions can cause DPI changes:
                    //
                    // * Changing the display's resolution.
                    // * Changing the display's scale factor (e.g. in Control Panel on Windows).
                    // * Moving the window to a display with a different scale factor.
                    //
                    // After this event callback has been processed, the window will be resized to whatever value
                    // is pointed to by the `new_inner_size` reference. By default, this will contain the size suggested
                    // by the OS, but it can be changed to any value.
                    //
                    // For more information about DPI in general, see the [`dpi`](crate::dpi) module.
                    WindowEvent::ScaleFactorChanged {
                        scale_factor: _,
                        new_inner_size,
                    } => {
                        let size = Quad::from_window_size(new_inner_size).to_wgpu_size();

                        if window.auto_resize {
                            let renderer = app.renderer();
                            let mut targets = renderer.write_targets();
                            let target = targets.get_mut(&target_id);
                            target
                                .is_some()
                                .then(|| target.unwrap().resize(&renderer, size));
                        }

                        window.call(
                            "rescale",
                            Event::Rescaled {
                                width: new_inner_size.width,
                                height: new_inner_size.height,
                            },
                        )
                    }

                    // The position of the window has changed.
                    // Contains the window's new position.
                    //
                    // Desktop only.
                    WindowEvent::Moved(new_position) => window.call(
                        "move",
                        Event::Moved {
                            x: new_position.x,
                            y: new_position.y,
                        },
                    ),

                    // The window has been requested to close.
                    WindowEvent::CloseRequested => {
                        let mut targets = renderer.write_targets();
                        targets.remove(&target_id);
                        drop(targets);

                        let removed = app.remove_window::<Window>(&window_id);
                        removed.map(|window| {
                            let window = window.write().unwrap();
                            window.instance.set_visible(false);
                            window.call("closed", Event::Closed);
                            if windows.len() == 0 {
                                window.call("exit", Event::Exit);
                            }
                            drop(window)
                        });

                        if windows.len() == 0 {
                            *control_flow = ControlFlow::Exit;
                        }
                    }

                    // The window has been destroyed.
                    WindowEvent::Destroyed => window.call("destroy", Event::Destroyed),

                    // A file has been dropped into the window.
                    //
                    // When the user drops multiple files at once,
                    // this event will be emitted for each file separately.
                    WindowEvent::DroppedFile(path) => {
                        if let Some(handle) = window.get_dropped_file_handle(path) {
                            window.call("file dropped", Event::FileDropped { handle })
                        }
                    }

                    // A file is being hovered over the window.
                    //
                    // When the user hovers multiple files at once, this event will be emitted for each file
                    // separately.
                    WindowEvent::HoveredFile(path) => {
                        let mut windows = app.write_to_windows_collection::<Window>();
                        let mut window = if let Some(window) = windows.get_mut(&window_id) {
                            window
                        } else {
                            return;
                        };

                        let handle = window.add_hovered_file(path);
                        window.call("file hovered", Event::FileHovered { handle })
                    }

                    // A file was hovered, but has exited the window.
                    //
                    // There will be a single `HoveredFileCancelled` event triggered even if multiple files were
                    // hovered.
                    WindowEvent::HoveredFileCancelled => {
                        window.call("hover cancelled", Event::FileHoverCancelled)
                    }

                    // The window received a unicode character.
                    WindowEvent::ReceivedCharacter(character) => window.call(
                        "character received",
                        Event::Character {
                            character: *character,
                        },
                    ),

                    // The window gained or lost focus.
                    // The parameter is true if the window has gained focus, and false if it has lost focus.
                    WindowEvent::Focused(focused) => {
                        window.call("focus", Event::Focus { focused: *focused })
                    }

                    // An event from the keyboard has been received.
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        input,
                        is_synthetic: _,
                    } => match input {
                        KeyboardInput {
                            state,
                            virtual_keycode,
                            scancode,
                            ..
                        } => {
                            let escape = match virtual_keycode {
                                Some(VirtualKeyCode::Escape) => true,
                                _ => false,
                            };
                            let released = match state {
                                ElementState::Pressed => false,
                                ElementState::Released => true,
                            };

                            if escape && released && window.close_on_esc {
                                let mut targets = renderer.write_targets();
                                targets.remove(&target_id);

                                let removed = app.remove_window::<Window>(&window_id);
                                removed.map(|window| {
                                    let window = window.write().unwrap();
                                    window.instance.set_visible(false);
                                    window.call("closed", Event::Closed);
                                    if windows.len() == 0 {
                                        window.call("exit", Event::Exit);
                                    }
                                    drop(window)
                                });

                                if windows.len() == 0 {
                                    *control_flow = ControlFlow::Exit;
                                }
                            }

                            match released {
                                true => window.call(
                                    "keyup",
                                    Event::KeyUp {
                                        key: *virtual_keycode,
                                        scancode: *scancode,
                                    },
                                ),
                                false => window.call(
                                    "keydown",
                                    Event::KeyDown {
                                        key: *virtual_keycode,
                                        scancode: *scancode,
                                    },
                                ),
                            }
                        }
                    },

                    // The keyboard modifiers have changed.
                    // Currently uninimplemented for Web.
                    WindowEvent::ModifiersChanged(_modifiers_state) => {}

                    // An event from an input method.
                    // **Note:** You have to explicitly enable this event using [`Window::set_ime_allowed`].
                    WindowEvent::Ime(_ime) => {}

                    // The cursor has moved on the window.
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position: _,
                        ..
                    } => {}

                    // The cursor has entered the window.
                    WindowEvent::CursorEntered { device_id: _ } => {
                        window.call("mouse enter", Event::CursorEntered)
                    }

                    // The cursor has left the window.
                    WindowEvent::CursorLeft { device_id: _ } => {
                        window.call("mouse leave", Event::CursorLeft)
                    }

                    // A mouse wheel movement or touchpad scroll occurred.
                    WindowEvent::MouseWheel {
                        device_id: _,
                        delta: _,
                        phase: _,
                        ..
                    } => {}

                    // An mouse button press has been received.
                    WindowEvent::MouseInput {
                        device_id: _,
                        state: _,
                        button: _,
                        ..
                    } => {}

                    // Touchpad magnification event with two-finger pinch gesture.
                    //
                    // Positive delta values indicate magnification (zooming in) and
                    // negative delta values indicate shrinking (zooming out).
                    //
                    // - Only available on **macOS**.
                    WindowEvent::TouchpadMagnify {
                        device_id: _,
                        delta: _,
                        phase: _,
                    } => {}

                    // Smart magnification event.
                    //
                    // On a Mac, smart magnification is triggered by a double tap with two fingers
                    // on the trackpad and is commonly used to zoom on a certain object
                    // (e.g. a paragraph of a PDF) or (sort of like a toggle) to reset any zoom.
                    // The gesture is also supported in Safari, Pages, etc.
                    //
                    // The event is general enough that its generating gesture is allowed to vary
                    // across platforms. It could also be generated by another device.
                    //
                    // Unfortunatly, neither [Windows](https://support.microsoft.com/en-us/windows/touch-gestures-for-windows-a9d28305-4818-a5df-4e2b-e5590f850741)
                    // nor [Wayland](https://wayland.freedesktop.org/libinput/doc/latest/gestures.html)
                    // support this gesture or any other gesture with the same effect.
                    //
                    // ## Platform-specific
                    //
                    // - Only available on **macOS 10.8** and later.
                    WindowEvent::SmartMagnify { device_id: _ } => {}

                    // Touchpad rotation event with two-finger rotation gesture.
                    //
                    // Positive delta values indicate rotation counterclockwise and
                    // negative delta values indicate rotation clockwise.
                    //
                    // ## Platform-specific
                    //
                    // - Only available on **macOS**.
                    WindowEvent::TouchpadRotate {
                        device_id: _,
                        delta: _,
                        phase: _,
                    } => {}

                    // Touchpad pressure event.
                    //
                    // At the moment, only supported on Apple forcetouch-capable macbooks.
                    // The parameters are: pressure level (value between 0 and 1 representing how hard the touchpad
                    // is being pressed) and stage (integer representing the click level).
                    WindowEvent::TouchpadPressure {
                        device_id: _,
                        pressure: _,
                        stage: _,
                    } => {}

                    // Motion on some analog axis. May report data redundant to other, more specific events.
                    WindowEvent::AxisMotion {
                        device_id: _,
                        axis: _,
                        value: _,
                    } => {}

                    // Touch event has been received
                    //
                    // ## Platform-specific
                    //
                    // - **macOS:** Unsupported.
                    WindowEvent::Touch(_touch) => {}

                    // The system window theme has changed.
                    //
                    // Applications might wish to react to this to change the theme of the content of the window
                    // when the system changes the window theme.
                    //
                    // ## Platform-specific
                    //
                    // - **iOS / Android / X11 / Wayland / Orbital:** Unsupported.
                    WindowEvent::ThemeChanged(_theme) => {}

                    // The window has been occluded (completely hidden from view).
                    //
                    // This is different to window visibility as it depends on whether the window is closed,
                    // minimised, set invisible, or fully occluded by another window.
                    //
                    // Platform-specific behavior:
                    // - **iOS / Android / Web / Wayland / Windows / Orbital:** Unsupported.
                    WindowEvent::Occluded(_bool) => {}
                }
            }

            // Emitted after [MainEventsCleared] when a window should be redrawn.
            //
            // This gets triggered in two scenarios:
            //   - The OS has performed an operation that's invalidated
            //     the window's contents (such as resizing the window).
            //   - The application has explicitly requested a redraw
            //     via Window::request_redraw.
            //
            // During each iteration of the event loop, Winit will aggregate
            // duplicate redraw requests into a single event, to help avoid
            // duplicating rendering work.
            //
            // Mainly of interest to applications with mostly-static graphics
            // that avoid redrawing unless something changes, like most non-game GUIs.
            Winit::RedrawRequested(window_id) => {
                let windows = app.read_windows_collection::<Window>();
                let window = if let Some(window) = windows.get(&window_id) {
                    window
                } else {
                    return;
                };

                // @TODO This should be a property of the target
                let target_frametime = Duration::from_secs_f64(TARGET_FRAME_TIME);
                // let target_id = TargetId::Window(window_id);

                let now = Instant::now();
                // This allows us to precisely control the frame rate
                *control_flow = match target_frametime.checked_sub(last_update.elapsed()) {
                    Some(wait_time) => ControlFlow::WaitUntil(now + wait_time),
                    None => {
                        window.call("draw", Event::Draw);
                        last_update = now;
                        ControlFlow::Poll
                    }
                };
            }

            // Emitted when all of the event loop's input events have been processed
            // and redraw processing is about to begin.
            //
            // This event is useful as a place to put your code that should be run
            // after all state-changing events have been handled and you want to do stuff
            // (updating state, performing calculations, etc) that happens as the "main body"
            // of your event_last_updateour program only draws graphics when something changes,
            // it's usually better to do it in response to Event::RedrawRequested, which gets
            // emitted immediately after this event. Programs that draw graphics continuously,
            // like most games, can render here unconditionally for simplicity.
            Winit::MainEventsCleared => {}

            Winit::NewEvents(_) => {}
            Winit::DeviceEvent {
                device_id: _,
                event: _,
            } => {}
            Winit::Suspended => {}
            Winit::Resumed => {}

            Winit::RedrawEventsCleared => {}

            Winit::LoopDestroyed => {}
        }
    });

    #[cfg(wasm)]
    event_loop.spawn(event_handler);

    #[cfg(not(wasm))]
    event_loop.run(event_handler);
}
