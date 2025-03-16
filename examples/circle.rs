use fragmentcolor::{FragmentColor, Renderer, Shader, ShaderError, Target, WindowTarget};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct State {
    window: Arc<Window>,
    target: WindowTarget,
    renderer: Renderer,
    shader: Shader,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let (renderer, target) = FragmentColor::init(window.clone()).await.unwrap();
        let size = target.size();

        let shader_source = include_str!("circle.wgsl");
        let shader = Shader::new(shader_source).unwrap();
        shader
            .set("resolution", [size.width as f32, size.height as f32])
            .unwrap();
        shader.set("circle.position", [0.0, 0.0]).unwrap();
        shader.set("circle.radius", 300.0).unwrap();
        shader.set("circle.color", [0.2, 0.2, 0.8, 1.0]).unwrap();
        shader.set("circle.border", 100.0).unwrap();

        State {
            window,
            target,
            renderer,
            shader,
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

        self.target.resize(&self.renderer, size);
        self.shader
            .set("resolution", [size.width as f32, size.height as f32])
            .unwrap();
    }

    fn render(&mut self) -> Result<(), ShaderError> {
        Ok(self.renderer.render(&self.shader, &self.target)?)
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
            // Render loop
            WindowEvent::RedrawRequested => {
                if let Err(err) = state.render() {
                    log::error!("Failed to render: {:?}", err);
                }

                state.get_window().request_redraw();
            }

            // resize
            WindowEvent::Resized(size) => {
                state.resize(size);
            }

            // quit
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping.");
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
