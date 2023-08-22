use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

use crate::enrichments::gaze::GazeEvent;
use crate::events::VipEvent;
use crate::state::State;

pub fn run_event_loop(event_loop: EventLoop<VipEvent>, mut state: State) {
    event_loop.run(move |event, _, control_flow| match event {
        #[cfg(not(feature = "texture"))]
        Event::UserEvent(event) => match event {
            VipEvent::Gaze(event) => match event {
                GazeEvent::ChangePosition { x, y } => state.handle_gaze_change_position_event(x, y),
                GazeEvent::ChangeNormalizedPosition { x, y } => {
                    println!("from event_loop runner: x: {}, y: {}", &x, &y);
                    state.handle_gaze_change_position_event_normalized(x, y)
                }
            },
        },

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
