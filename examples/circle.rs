use std::sync::Arc;

use fragmentcolor::{
    Color, Frame, Pass, PassInput, RenderPass, Renderer, Shader, ShaderError, Target, WindowTarget,
};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct State {
    window: Arc<Window>,
    target: Arc<WindowTarget>,
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

        device.on_uncaptured_error(Box::new(|error| {
            println!("\n\n==== GPU error: ====\n\n{:#?}\n", error);
        }));

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let capabilities = surface.get_capabilities(&adapter);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilities.formats[0].remove_srgb_suffix(),
            width: u32::max(size.width, 1),
            height: u32::max(size.height, 1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_configuration);

        ////////////// public API // @TODO transform the boilerplate into a initializer

        let shader_source = include_str!("circle.wgsl");
        let shader = Shader::new(shader_source).unwrap();
        shader
            .set("resolution", [size.width as f32, size.height as f32])
            .unwrap();
        shader.set("circle.position", [0.0, 0.0]).unwrap();
        shader.set("circle.radius", 200.0).unwrap();
        shader.set("circle.color", [1.0, 0.2, 0.8, 1.0]).unwrap();
        shader.set("circle.border", 20.0).unwrap();

        let shader = Arc::new(shader);

        let mut frame = Frame::new();
        let mut pass = RenderPass::new(
            "Single Pass",
            PassInput::Clear(Color::from_rgba([0.5, 0.4, 0.2, 1.0])),
        );
        let target = Arc::new(WindowTarget {
            surface,
            config: surface_configuration,
        });
        pass.add_shader(shader.clone());
        pass.add_target(target.clone());

        frame.add_pass(Pass::Render(pass));

        /////////////

        State {
            window,
            target,
            renderer: Renderer::new(device, queue),
            frame,
        }
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

        // @FIXME this fails silently
        if let Some(target) = Arc::get_mut(&mut self.target) {
            target.resize(&self.renderer, size);
        }
    }

    fn render(&mut self) -> Result<(), ShaderError> {
        Ok(self.renderer.render(&self.frame, self.target.as_ref())?)
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
        //self.windows.insert(window.id(), window.clone());

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
