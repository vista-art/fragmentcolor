struct OldWindow;

impl OldWindow {
    // @TODO The user should not care about injecting the runner here.
    //       this method should be just run() or open(), without arguments.
    //
    // old signature: (delete this comment after transition is complete)
    pub fn run(self, mut runner: impl 'static + FnMut(Event)) -> ! {
        use instant::{Duration, Instant};
        use winit::{
            event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
            event_loop::ControlFlow,
        };

        let mut last_update_inst = Instant::now();
        let Self {
            _,
            event_loop,
            instance: window,
        } = self;

        // NOTE: the original author uses this "runner" function as a callback.
        //       this works for Rust because match() expressions, but it is clucky
        //       for other languages.
        //
        // @TODO instead of a global "runner" callback function:
        //       For each Winit event, we should create a on_<event> method which accepts
        //       a callback function. This is more flexible and ergonomic for other languages.
        event_loop.run(move |event, _, control_flow| {
            *control_flow = match event {
                Winit::WindowEvent { window_id, event } => match event {
                    WindowEvent::Resized(size) => {
                        runner(Event::Resize {
                            width: size.width,
                            height: size.height,
                        });
                        ControlFlow::Poll
                    },
                    WindowEvent::CloseRequested => {
                        // @TODO close current window
                        // do NOT exit unless we closed them all
                        ControlFlow::Exit
                    },
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(code),
                                ..
                            },
                        ..
                    } => {

                    }
                },

                Winit::WindowEvent {
                    window_id: _, // we'll use this later...
                    event:
                        ,
                } => {
                    runner(Event::Keyboard {
                        key: if code >= KeyCode::Key1 && code <= KeyCode::Key0 {
                            Key::Digit(code as u8 - KeyCode::Key1 as u8)
                        } else if code >= KeyCode::A && code <= KeyCode::Z {
                            Key::Letter((code as u8 - KeyCode::A as u8) as char)
                        } else if code >= KeyCode::F1 && code <= KeyCode::F12 {
                            Key::Function(code as u8 - KeyCode::F1 as u8)
                        } else {
                            match code {
                                KeyCode::Left => Key::Left,
                                KeyCode::Right => Key::Right,
                                KeyCode::Up => Key::Up,
                                KeyCode::Down => Key::Down,
                                KeyCode::Space => Key::Space,
                                KeyCode::Escape => Key::Escape,
                                _ => {
                                    log::debug!("Unrecognized key {:?}", code);
                                    Key::Other
                                }
                            }
                        },
                        pressed: state == ElementState::Pressed,
                    });
                    ControlFlow::Poll
                }
                Winit::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    runner(Event::Pointer {
                        position: mint::Vector2 {
                            x: position.x as f32,
                            y: position.y as f32,
                        },
                    });
                    ControlFlow::Poll
                }
                Winit::WindowEvent {
                    event: WindowEvent::MouseInput { button, state, .. },
                    ..
                } => {
                    runner(Event::Click {
                        button: match button {
                            MouseButton::Left => Button::Left,
                            MouseButton::Middle => Button::Middle,
                            MouseButton::Right => Button::Right,
                            MouseButton::Other(code) => Button::Other(code),
                        },
                        pressed: state == ElementState::Pressed,
                    });
                    ControlFlow::Poll
                }
                Winit::WindowEvent {
                    event: WindowEvent::MouseWheel { delta, .. },
                    ..
                } => {
                    match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            runner(Event::Scroll {
                                delta: mint::Vector2 { x, y },
                            });
                        }
                        MouseScrollDelta::PixelDelta(position) => {
                            runner(Event::Scroll {
                                delta: mint::Vector2 {
                                    x: position.x as f32,
                                    y: position.y as f32,
                                },
                            });
                        }
                    }
                    ControlFlow::Poll
                }
                Winit::RedrawRequested(_) => {
                    runner(Event::Draw);
                    ControlFlow::Poll
                }
                Winit::RedrawEventsCleared => {
                    let target_frametime = Duration::from_secs_f64(TARGET_FRAME_TIME);
                    let now = Instant::now();
                    match target_frametime.checked_sub(last_update_inst.elapsed()) {
                        Some(wait_time) => ControlFlow::WaitUntil(now + wait_time),
                        None => {
                            window.request_redraw();
                            last_update_inst = now;
                            ControlFlow::Poll
                        }
                    }
                }
                Winit::LoopDestroyed => {
                    runner(Event::Exit);
                    ControlFlow::Exit
                }
                _ => ControlFlow::Poll,
            }
        })
    }
}
