use std::sync::Arc;

use fragmentcolor::{
    Color, Frame, Pass, PassInput, RenderPass, Renderer, Shader, ShaderError, Target,
};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct State {
    window: Arc<Window>,
    renderer: Renderer,
    frame: Frame,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = fragmentcolor::platform::all::request_device(&adapter)
            .await
            .expect("Failed to request device");

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        ////////////// public API // @TODO transform the boilerplate into a initializer

        let shader_source = include_str!("circle.wgsl");
        let shader = Arc::new(Shader::new(shader_source).expect("Failed to create shader"));
        let mut frame = Frame::new();
        let mut pass = RenderPass::new("Single Pass", PassInput::Clear(Color::default()));
        let target = Target::from_surface(surface, size.width, size.height, surface_format);
        pass.add_shader(shader);
        frame.add_pass(Pass::Render(pass));
        frame.add_target(target);

        /////////////

        let mut state = State {
            window,
            renderer: Renderer::new(device, queue),
            frame,
        };

        // Configure surface for the first time
        state.frame.resize_targets(
            &state.renderer,
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let size = wgpu::Extent3d {
            width: new_size.width,
            height: new_size.height,
            depth_or_array_layers: 1,
        };

        // reconfigure the surface
        self.frame.resize_targets(&self.renderer, size);
    }

    fn render(&mut self) -> Result<(), ShaderError> {
        Ok(self.renderer.render(&self.frame)?)
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = state.render() {
                    log::error!("Failed to render: {:?}", err);
                }

                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    let _ = event_loop.run_app(&mut app);
}
