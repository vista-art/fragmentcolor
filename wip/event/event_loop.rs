use crate::renderer::Renderer;
use crate::Event;
use std::{sync::Arc, sync::RwLock};
#[cfg(wasm)]
use winit::platform::web::EventLoopExtWebSys;
use winit::{
    event::{ElementState, Event as Winit, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{
        ControlFlow, EventLoop as WinitEventLoop, EventLoopBuilder, EventLoopWindowTarget,
    },
};

pub struct EventLoop<T: 'static> {
    context: Arc<RwLock<Renderer>>,
    event_loop: WinitEventLoop<T>,
    event_loop_runner: Box<dyn FnOnce(EventLoop<T>, Arc<RwLock<Renderer>>) + Send>,
}

impl EventLoop<Event> {
    pub fn new() -> Self {
        Self {
            renderer,
            event_loop: EventLoopBuilder::<Event>::with_user_event().build(),
            event_loop_runner: Box::new(run_event_loop),
        }
    }

    pub fn get_event_loop(&self) -> &EventLoop<Event> {
        &self.event_loop.create_proxy()
    }

    pub async fn run_event_loop(self) {
        let renderer = self.renderer.expect("Renderer not set");
        (self.event_loop_runner)(self.event_loop, renderer.clone())
    }
}

// Shorthand types for Winit's event handler arguments
type E<'a> = Winit<'a, Event>;
type T<'b> = &'b EventLoopWindowTarget<Event>;
type C<'c> = &'c mut ControlFlow;

fn run_event_loop(event_loop: EventLoop<Event>, renderer: Arc<RwLock<Renderer>>) {
    let event_handler = Box::new(move |event: E, _window_target: T, control_flow: C| {
        let mut renderer = renderer.write().expect("Couldn't get renderer write lock");

        match event {
            Winit::UserEvent(event) => {
                let scene = renderer.get_scene(&event.scene);
                if scene.is_some() {
                    scene.unwrap().handle(event);
                }

                match event.event.as_str() {
                    "quit" => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }

            Winit::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window().id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,

                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    renderer.resize(**new_inner_size);
                }

                _ => {
                    renderer.window_input(&event);
                }
            },

            Winit::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                for scene in renderer.scenes() {
                    scene.write().unwrap().update();
                }

                match renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => renderer.recover(),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            Winit::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it.
                renderer.window().request_redraw();
            }

            _ => (),
        }
    });

    #[cfg(wasm)]
    event_loop.spawn(event_handler);

    #[cfg(not(wasm))]
    event_loop.run(event_handler);
}
