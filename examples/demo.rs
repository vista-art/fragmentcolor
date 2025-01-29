use fragmentcolor::{Renderer, Shader};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    shader: Shader,
    renderer: Option<Renderer>,
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        self.renderer = Some(pollster::block_on(Renderer::new(self.window.as_ref())).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let renderer = self.renderer.as_ref().unwrap();

                renderer.render(&self.shader).unwrap();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let shader_source = include_str!("hello_triangle.wgsl");
    let shader = Shader::new(shader_source);

    let mut app = App {
        shader,
        ..Default::default()
    };

    let _ = event_loop.run_app(&mut app);
}
