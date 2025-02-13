use fragmentcolor::{init_renderer, Frame, RenderPass, Renderer, Shader};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct State {
    window: Window,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    surface_size: winit::dpi::PhysicalSize<u32>,
    renderer: Option<Renderer>,
    window: Option<Window>,
}

#[derive(Default)]
struct App {
    shader: Option<Shader>,
    renderer: Option<Renderer>,
    // pass: RenderPass,
    // frame: Frame,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state = Some(State {
            window: event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
            surface: wgpu::Surface::create(&self.window),
            surface_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            surface_size: self.window.inner_size(),
            renderer: Some(pollster::block_on(init_renderer(self.window.as_ref())).unwrap()),
        });

        self.renderer = Some(pollster::block_on(init_renderer(self.window.as_ref())).unwrap());
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
    let shader = Shader::new(shader_source).unwrap();

    let mut app = App {
        shader: Some(shader),
        ..Default::default()
    };

    let _ = event_loop.run_app(&mut app);
}
