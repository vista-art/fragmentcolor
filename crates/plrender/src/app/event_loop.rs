use crate::{
    app::{AppState, Container, Event, EventListener, Window},
    renderer::{
        target::{RenderTarget, Target, TargetId},
        RenderContext, RenderTargetCollection,
    },
};
use instant::{Duration, Instant};
use std::sync::{Arc, Mutex};
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
pub trait Runner: 'static + FnOnce(WinitEventLoop<Event>, Arc<Mutex<AppState>>) + Send {}
impl<F> Runner for F where F: 'static + FnOnce(WinitEventLoop<Event>, Arc<Mutex<AppState>>) + Send {}
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

    pub async fn run(&mut self, runner: EventLoopRunner, app: Arc<Mutex<AppState>>) {
        let event_loop = self.inner.take().expect(RUNNING);
        runner(event_loop, app.clone())
    }
}

// Shorthand types for Winit's event handler arguments
type E<'a> = Winit<'a, Event>;
type W<'b> = &'b EventLoopWindowTarget<Event>;
type C<'c> = &'c mut ControlFlow;

pub fn run_event_loop(event_loop: WinitEventLoop<Event>, app: Arc<Mutex<AppState>>) {
    let event_handler = Box::new(move |event: E, _elwt: W, control_flow: C| {
        let app = app.try_lock().expect("Couldn't get AppState mutex lock");
        let renderer = app.renderer();
        let renderer = renderer.as_ref().expect("Renderer not initialized");
        let mut targets = renderer.targets();
        let mut windows = app.windows::<Window>();

        let mut last_update = Instant::now();

        match event {
            // Reserved for our custom dispatched events
            Winit::UserEvent(event) => match event {
                _ => {}
            },

            // This variant represents anything that happens in a Window.
            // @TODO we will expose callbacks for all of Window events
            //       so the user can handle them
            Winit::WindowEvent {
                ref event,
                window_id,
            } => {
                let target_id = TargetId::Window(window_id);
                match event {
                    // The size of the window has changed.
                    // Contains the client area's new dimensions.
                    WindowEvent::Resized(physical_size) => {
                        let size = wgpu::Extent3d {
                            width: physical_size.width,
                            height: physical_size.height,
                            depth_or_array_layers: 1,
                        };

                        let window = windows.get(window_id);
                        window.is_some().then(|| {
                            if window.as_ref().unwrap().auto_resize {
                                let target = targets.get_mut(&target_id);
                                target
                                    .is_some()
                                    .then(|| target.unwrap().resize(&renderer, size));
                            }
                            window.unwrap().call(
                                "resize",
                                Event::Resize {
                                    width: physical_size.width,
                                    height: physical_size.height,
                                },
                            )
                        });
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
                        let _size = wgpu::Extent3d {
                            width: new_inner_size.width,
                            height: new_inner_size.height,
                            depth_or_array_layers: 1,
                        };

                        // let target = targets.get_mut(&target_id);
                        // target
                        //     .is_some()
                        //     .then(|| target.unwrap().resize(&renderer, size));
                    }

                    // The position of the window has changed.
                    // Contains the window's new position.
                    // Desktop only.
                    WindowEvent::Moved(_new_position) => {}

                    // The window has been requested to close.
                    WindowEvent::CloseRequested => {
                        println!("Window {window_id:?} has received the signal to close");

                        let window = windows.remove(window_id);

                        window.map(|w| w.write().unwrap().instance.set_visible(false));
                        targets.remove(&target_id);

                        if windows.len() == 0 {
                            *control_flow = ControlFlow::Exit;
                        }
                    }

                    // The window has been destroyed.
                    WindowEvent::Destroyed => {}

                    // A file has been dropped into the window.
                    //
                    // When the user drops multiple files at once,
                    // this event will be emitted for each file separately.
                    WindowEvent::DroppedFile(_path) => {}

                    // A file is being hovered over the window.
                    //
                    // When the user hovers multiple files at once, this event will be emitted for each file
                    // separately.
                    WindowEvent::HoveredFile(_path) => {}

                    // A file was hovered, but has exited the window.
                    //
                    // There will be a single `HoveredFileCancelled` event triggered even if multiple files were
                    // hovered.
                    WindowEvent::HoveredFileCancelled => {}

                    // The window received a unicode character.
                    WindowEvent::ReceivedCharacter(_character) => {}

                    // The window gained or lost focus.
                    // The parameter is true if the window has gained focus, and false if it has lost focus.
                    WindowEvent::Focused(_bool) => {}

                    // An event from the keyboard has been received.
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        input,
                        is_synthetic: _,
                    } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            scancode: _,
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
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
                    WindowEvent::CursorEntered { device_id: _ } => {}

                    // The cursor has left the window.
                    WindowEvent::CursorLeft { device_id: _ } => {}

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
            Winit::RedrawRequested(_window_id) => {
                // @TODO this should be handled by the user of this library
                //       the responsibility of this variant is just to
                //       fire the appropriate user event and call the right callback

                // let target_id = TargetId::Window(window_id);

                // match renderer.render() {
                //     Ok(_) => {}

                //     // Legacy options, but we still need to handle them
                //     // when we acquire the next frame

                //     //// Reconfigure the surface if lost
                //     // Err(wgpu::SurfaceError::Lost) => {
                //     //     let target = targets.get(&target_id).expect("Couldn't get target");
                //     //     renderer.resize()
                //     // }

                //     //// The system is out of memory, we should probably quit
                //     //Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                //     // All other errors (Outdated, Timeout) should be resolved by the next frame
                //     Err(e) => eprintln!("{:?}", e),
                // }
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
            Winit::MainEventsCleared => {
                // @TODO each target could have a different frame rate

                // After getting window & target from the App
                for target in targets.all() {
                    let window_id = match target {
                        Target::Window(window) => window.id,
                        _ => continue,
                    };
                    let window = windows.get(window_id).expect("Couldn't get window");

                    // This should be a property of the target
                    let target_frametime = Duration::from_secs_f64(TARGET_FRAME_TIME);
                    let now = Instant::now();

                    // This allows us to precisely control the frame rate
                    *control_flow = match target_frametime.checked_sub(last_update.elapsed()) {
                        Some(wait_time) => ControlFlow::WaitUntil(now + wait_time),
                        None => {
                            window.instance.request_redraw();
                            last_update = now;
                            ControlFlow::Poll
                        }
                    }
                }
            }

            // @TODO document the variants below
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
