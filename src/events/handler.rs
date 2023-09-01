use std::{cell::RefCell, sync::Arc};

use crate::events::VipEvent;
use crate::renderer::Renderer;
#[cfg(wasm)]
use winit::platform::web::EventLoopExtWebSys;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
};

pub struct EventHandler<T: 'static> {
    renderer: Option<Arc<RefCell<Renderer>>>,
    event_loop: EventLoop<T>,
    event_loop_runner: Box<dyn FnOnce(EventLoop<T>, Renderer) + Send>,
}

impl EventHandler<VipEvent> {
    pub fn new() -> Self {
        Self {
            renderer: None,
            event_loop: EventLoopBuilder::<VipEvent>::with_user_event().build(),
            event_loop_runner: Box::new(run_event_loop),
        }
    }

    pub fn attach_renderer(&mut self, renderer: Arc<RefCell<Renderer>>) {
        self.renderer = Some(renderer);
    }

    pub fn get_event_loop(&self) -> &EventLoop<VipEvent> {
        &self.event_loop
    }

    pub async fn run(mut self) {
        let renderer = self.renderer.take().expect("Renderer not set");
        let renderer = Arc::try_unwrap(renderer)
            .expect("Couldn't unwrap Arc<RefCell<Renderer>>")
            .into_inner();

        // The runner takes ownership of the event loop and renderer
        (self.event_loop_runner)(self.event_loop, renderer)
    }
}

fn run_event_loop(event_loop: EventLoop<VipEvent>, mut renderer: Renderer) {
    use log::info;
    type E<'a> = Event<'a, VipEvent>;
    type T<'b> = &'b EventLoopWindowTarget<VipEvent>;
    type C<'c> = &'c mut ControlFlow;

    let event_handler = Box::new(move |vip_event: E, _target: T, control_flow: C| {
        match vip_event {
            Event::UserEvent(vip_event) => match vip_event {
                VipEvent::Gaze(_) => {
                    let controller = renderer.get_controller("Gaze".to_string());

                    if controller.is_some() {
                        info!("Gaze event received");
                        controller.unwrap().handle(vip_event);
                    }
                }
            },

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
