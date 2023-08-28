#[cfg(not(feature = "texture"))]
use crate::enrichments::gaze::GazeEvent;
use crate::events::VipEvent;
use crate::renderer::Renderer;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
};

pub struct EventHandler<T: 'static> {
    renderer: Option<Renderer>,
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

    pub fn attach_renderer(&mut self, renderer: Renderer) {
        self.renderer = Some(renderer);
    }

    pub fn get_event_loop(&self) -> &EventLoop<VipEvent> {
        &self.event_loop
    }

    /// This function will execute the main event loop to
    pub async fn run(mut self) {
        let renderer = self.renderer.take().expect("Renderer not set");
        (self.event_loop_runner)(self.event_loop, renderer);
    }
}

fn run_event_loop(event_loop: EventLoop<VipEvent>, mut renderer: Renderer) {
    let event_handler = Box::new(
        move |event: Event<'_, VipEvent>,
              _: &EventLoopWindowTarget<VipEvent>,
              control_flow: &mut ControlFlow| {
            match event {
                #[cfg(not(feature = "texture"))]
                Event::UserEvent(event) => match event {
                    VipEvent::Gaze(event) => match event {
                        GazeEvent::ChangePosition { x, y } => {
                            println!("from event_loop runner: x: {}, y: {}", &x, &y);
                            //renderer.update(x, y), // @TODO
                        }
                        GazeEvent::ChangeNormalizedPosition { x, y } => {
                            println!("from event_loop runner: x: {}, y: {}", &x, &y);
                        }
                    },
                },

                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == renderer.window().id() && !renderer.input(event) => match event {
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

                    _ => {}
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
        },
    );

    #[cfg(target_arch = "wasm32")]
    event_loop.spawn(event_handler);

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run(event_handler);
}
