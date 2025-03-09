use std::sync::Arc;

use fragmentcolor::{Frame, Pass, Renderer, Shader, ShaderError, Target, WindowTarget};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct State {
    window: Arc<Window>,
    target: WindowTarget,
    renderer: Renderer,
    frame: Frame,
    circle: Shader,
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

        device.on_uncaptured_error(Box::new(|error| {
            println!("\n\n==== GPU error: ====\n\n{:#?}\n", error);
        }));

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let capabilities = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilities.formats[0].remove_srgb_suffix(),
            width: u32::max(size.width, 1),
            height: u32::max(size.height, 1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let target = WindowTarget { surface, config };
        let renderer = Renderer::new(device, queue);

        let triangle_source = include_str!("hello_triangle.wgsl");
        let triangle = Shader::new(triangle_source).unwrap();
        triangle.set("color", [1.0, 0.2, 0.8, 1.0]).unwrap();

        let circle_source = include_str!("circle.wgsl");
        let circle = Shader::new(circle_source).unwrap();
        circle
            .set("resolution", [size.width as f32, size.height as f32])
            .unwrap();
        circle.set("circle.position", [0.0, 0.0]).unwrap();
        circle.set("circle.radius", 200.0).unwrap();
        circle.set("circle.color", [0.2, 0.8, 0.3, 1.0]).unwrap();
        circle.set("circle.border", 20.0).unwrap();

        let mut pass = Pass::new("Multi Object Pass");
        pass.add_shader(&triangle);
        pass.add_shader(&circle);

        let mut frame = Frame::new();
        frame.add_pass(pass);

        State {
            window,
            target,
            renderer,
            frame,
            circle,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let size = wgpu::Extent3d {
            width: new_size.width,
            height: new_size.height,
            depth_or_array_layers: 1,
        };

        self.circle
            .set("resolution", [size.width as f32, size.height as f32])
            .unwrap();
        self.target.resize(&self.renderer, size);
    }

    fn render(&mut self) -> Result<(), ShaderError> {
        Ok(self.renderer.render(&self.frame, &self.target)?)
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
            // render loop
            WindowEvent::RedrawRequested => {
                if let Err(err) = state.render() {
                    log::error!("Failed to render: {:?}", err);
                }

                state.window().request_redraw();
            }

            // resize
            WindowEvent::Resized(size) => {
                state.resize(size);
            }

            // quit
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
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
