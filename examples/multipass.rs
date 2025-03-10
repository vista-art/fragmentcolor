use rand::prelude::*;
use std::sync::Arc;

use fragmentcolor::{
    FragmentColor, Frame, Pass, Renderer, Shader, ShaderError, Target, WindowTarget,
};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const CIRCLE_SOURCE: &str = include_str!("circle.wgsl");
const TRIANGLE_SOURCE: &str = include_str!("hello_triangle.wgsl");

struct State {
    window: Arc<Window>,
    target: WindowTarget,
    renderer: Renderer,
    frame: Frame,
    circles: Vec<Shader>,
}

fn random_circle(rng: &mut impl Rng, size: &wgpu::Extent3d, alpha: f32) -> Shader {
    let circle = Shader::new(CIRCLE_SOURCE).unwrap();
    circle
        .set("resolution", [size.width as f32, size.height as f32])
        .unwrap();

    let x = rng.random_range(-(size.width as f32)..size.width as f32);
    let y = rng.random_range(-(size.height as f32)..size.height as f32);
    circle.set("circle.position", [x, y]).unwrap();

    let r = rng.random_range(0.0..1.0);
    let g = rng.random_range(0.0..1.0);
    let b = rng.random_range(0.0..1.0);
    circle.set("circle.color", [r, g, b, alpha]).unwrap();

    let radius = rng.random_range(50.0..300.0);
    circle.set("circle.radius", radius).unwrap();

    let border = rng.random_range(10.0..100.0);
    circle.set("circle.border", border).unwrap();

    circle
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let (renderer, target) = FragmentColor::init(window.clone()).await.unwrap();
        let size = target.size();

        let triangle = Shader::new(TRIANGLE_SOURCE).unwrap();
        triangle.set("color", [1.0, 0.2, 0.8, 1.0]).unwrap();

        let opaque_pass = Pass::new("Opaque Pass");
        let transparent_pass = Pass::new("Transparent Pass");
        opaque_pass.add_shader(&triangle);
        opaque_pass.set_clear_color([0.0, 0.0, 0.0, 1.0]);

        let mut rng = rand::rng();

        let mut circles = Vec::new();
        for i in 0..10 {
            let circle = random_circle(&mut rng, &size, 1.0);
            circles.push(circle);
            opaque_pass.add_shader(&circles[i]);
        }

        for i in 0..20 {
            let circle = random_circle(&mut rng, &size, 0.2);
            circles.push(circle);
            transparent_pass.add_shader(&circles[i]);
        }

        let mut frame = Frame::new();
        frame.add_pass(&opaque_pass);
        frame.add_pass(&transparent_pass);

        State {
            window,
            target,
            renderer,
            frame,
            circles,
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

        for circle in &self.circles {
            circle
                .set("resolution", [size.width as f32, size.height as f32])
                .unwrap();
        }

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
