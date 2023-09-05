use crate::events::VipEvent;
use crate::renderer::Renderer;
use std::{sync::Arc, sync::RwLock};
#[cfg(wasm)]
use winit::platform::web::EventLoopExtWebSys;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
};

pub struct EventHandler<T: 'static> {
    renderer: Option<Arc<RwLock<Renderer>>>,
    event_loop: EventLoop<T>,
    event_loop_runner: Box<dyn FnOnce(EventLoop<T>, Arc<RwLock<Renderer>>) + Send>,
}

impl EventHandler<VipEvent> {
    pub fn new() -> Self {
        Self {
            renderer: None,
            event_loop: EventLoopBuilder::<VipEvent>::with_user_event().build(),
            event_loop_runner: Box::new(run_event_loop),
        }
    }

    pub fn attach_renderer(&mut self, renderer: Arc<RwLock<Renderer>>) {
        self.renderer = Some(renderer);
    }

    pub fn get_event_loop(&self) -> &EventLoop<VipEvent> {
        &self.event_loop
    }

    pub async fn run_event_loop(self) {
        let renderer = self.renderer.expect("Renderer not set");
        (self.event_loop_runner)(self.event_loop, renderer.clone())
    }
}

type E<'a> = Event<'a, VipEvent>;
type L<'b> = &'b EventLoopWindowTarget<VipEvent>;
type C<'c> = &'c mut ControlFlow;

fn run_event_loop(event_loop: EventLoop<VipEvent>, renderer: Arc<RwLock<Renderer>>) {
    let event_handler = Box::new(move |event: E, _event_loop: L, control_flow: C| {
        let mut renderer = renderer.write().expect("Couldn't get renderer write lock");

        match event {
            Event::UserEvent(event) => {
                let controller = renderer.get_controller(&event.controller);
                if controller.is_some() {
                    controller.unwrap().handle(event);
                }
            }

            Event::WindowEvent {
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
                    #[cfg(feature = "camera")]
                    renderer.window_input(&event);
                }
            },

            Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                renderer.update();

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

            Event::MainEventsCleared => {
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
