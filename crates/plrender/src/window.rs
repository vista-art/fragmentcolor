// Waiting for https://github.com/gfx-rs/wgpu/pull/4202
// use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use serde::{Deserialize, Serialize};
use winit::{
    event::{KeyboardInput, VirtualKeyCode},
    event_loop::EventLoop,
    window::Fullscreen,
};

type Error = Box<dyn std::error::Error>;

const TARGET_FRAME_TIME: f64 = 1.0 / 120.0;

#[derive(Debug)]
pub struct Window {
    event_loop: EventLoop<()>,
    instance: winit::window::Window,
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

type KeyCode = VirtualKeyCode;

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.instance.raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.instance.raw_display_handle()
    }
}

impl plr::IsWindow for Window {}
impl plr::HasSize for Window {
    fn size(&self) -> wgpu::Extent3d {
        let size = self.instance.inner_size();

        wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        }
    }

    fn aspect(&self) -> f32 {
        let size = self.size();
        size.width as f32 / size.height as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Digit(u8),
    Letter(char),
    Function(u8),
    Up,
    Down,
    Left,
    Right,
    Space,
    Escape,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Button {
    Left,
    Middle,
    Right,
    Other(u16),
}

pub enum Event {
    Resize { width: u32, height: u32 },
    Keyboard { key: Key, pressed: bool },
    Pointer { position: mint::Vector2<f32> },
    Scroll { delta: mint::Vector2<f32> },
    Click { button: Button, pressed: bool },
    Draw,
    Exit,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WindowOptions {
    pub decorations: Option<bool>,
    pub fullscreen: Option<bool>,
    pub resizable: Option<bool>,
    pub title: Option<String>,
    pub size: Option<(u32, u32)>,
}

impl Window {
    pub fn new(options: WindowOptions) -> Result<Self, winit::error::OsError> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title(options.title.as_ref().unwrap_or(&"PLRender".to_string()))
            .with_inner_size(winit::dpi::Size::Logical(
                options.size.unwrap_or((800, 600)).into(),
            ))
            .with_min_inner_size(winit::dpi::Size::Logical((64, 64).into()))
            .with_fullscreen(
                options
                    .fullscreen
                    .unwrap_or(false)
                    .then(|| Fullscreen::Borderless(None)),
            )
            .with_decorations(options.decorations.unwrap_or(true))
            .with_resizable(options.resizable.unwrap_or(true))
            .build(&event_loop)?;

        Ok(Window {
            event_loop,
            instance: window,
        })
    }

    // pub fn add_event_listener(callback: impl FnMut) {
    //     todo!()
    // }

    pub fn run(self, mut runner: impl 'static + FnMut(Event)) -> ! {
        use instant::{Duration, Instant};
        use winit::{
            event::{
                ElementState, Event as WinitEvent, MouseButton, MouseScrollDelta, WindowEvent,
            },
            event_loop::ControlFlow,
        };

        let mut last_update_inst = Instant::now();
        let Self {
            event_loop,
            instance: window,
        } = self;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = match event {
                WinitEvent::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    runner(Event::Resize {
                        width: size.width,
                        height: size.height,
                    });
                    ControlFlow::Poll
                }
                WinitEvent::WindowEvent {
                    window_id: _, // we'll use this later...
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode: Some(code),
                                    ..
                                },
                            ..
                        },
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
                WinitEvent::WindowEvent {
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
                WinitEvent::WindowEvent {
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
                WinitEvent::WindowEvent {
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
                WinitEvent::RedrawRequested(_) => {
                    runner(Event::Draw);
                    ControlFlow::Poll
                }
                WinitEvent::RedrawEventsCleared => {
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
                WinitEvent::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => ControlFlow::Exit,
                WinitEvent::LoopDestroyed => {
                    runner(Event::Exit);
                    ControlFlow::Exit
                }
                _ => ControlFlow::Poll,
            }
        })
    }
}
