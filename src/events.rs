use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use {gloo_utils::document, wasm_bindgen::JsCast, winit::platform::web::WindowBuilderExtWebSys};

use crate::state::State;

pub enum VipEvent {
    MouseMove { x: f32, y: f32 },
}

pub struct EventManager {
    pub event_loop: Option<EventLoop<VipEvent>>,
    pub event_loop_proxy: EventLoopProxy<VipEvent>,
    pub state: Option<State>,
}

impl EventManager {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::<VipEvent>::with_user_event().build();
        let event_loop_proxy = event_loop.create_proxy();

        Self {
            event_loop: Some(event_loop),
            event_loop_proxy,
            state: None,
        }
    }

    // This has to be managed in a global context for WASM because it lacks threads..
    // @TODO figure out a way to unify the API across platforms
    //       Bevy has a good example of this.
    // #[cfg(target_arch = "wasm32")]
    // let window = init_window(event_loop, options.canvas_selector.as_ref().unwrap());
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn create_state(&mut self, options: crate::Options) {
        let event_loop = self.event_loop.as_ref().expect("Event loop not set");
        let window = self.init_window();
        let state = State::new(window, options.enrichments).await;

        self.state = Some(state);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn init_window(&self) -> Window {
        let event_loop = self.event_loop.as_ref().expect("Event loop not set");

        let window = WindowBuilder::new()
            .build(event_loop)
            .expect("Couldn't build window");

        window
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn run(&mut self) {
        let event_loop = self.event_loop.take().expect("Event loop not set");
        let state = self.state.take().expect("State not set");

        run_event_loop(event_loop, state);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn init_window(event_loop: &EventLoop<VipEvent>, canvas_selector: &str) -> Window {
    let canvas: Option<web_sys::HtmlCanvasElement> = document()
        .query_selector(canvas_selector)
        .unwrap()
        .expect("Couldn't get canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok();

    let size = canvas
        .as_ref()
        .expect("Couldn't get canvas size")
        .get_bounding_client_rect();

    let window = WindowBuilder::new()
        .with_canvas(canvas)
        .build(event_loop)
        .expect("Couldn't build canvas context");

    window.set_inner_size(winit::dpi::LogicalSize::new(
        size.width() as u32,
        size.height() as u32,
    ));

    window
}

pub async fn run_event_loop(event_loop: EventLoop<VipEvent>, mut state: State) {
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() && !state.input(event) => match event {
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
                state.resize(*physical_size);
            }

            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                state.resize(**new_inner_size);
            }

            _ => {}
        },

        #[cfg(not(feature = "texture"))]
        Event::UserEvent(event) => match event {
            VipEvent::MouseMove { x, y } => state.handle_mouse_move_vip_event(x, y),
        },

        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();

            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.recover(),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }

        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually request it.
            state.window().request_redraw();
        }

        _ => (),
    })
}
