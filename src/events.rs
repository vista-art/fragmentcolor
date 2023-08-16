use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

use crate::State;

fn handle_window_event(event: &WindowEvent<'_>, state: &mut State, control_flow: &mut ControlFlow) {
    match event {
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
    }
}

fn handle_event(event: Event<()>, state: &mut State, control_flow: &mut ControlFlow) {
    match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() && !state.input(event) => {
            handle_window_event(event, state, control_flow);
        }

        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            let scale_factor = state.window().scale_factor();

            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size.to_physical(scale_factor)),
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
    }
}

pub fn run_event_loop(event_loop: EventLoop<()>, mut state: State) -> ! {
    event_loop.run(move |event, _, control_flow| handle_event(event, &mut state, control_flow));
}
