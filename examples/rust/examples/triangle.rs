use fragmentcolor::{App, Frame, Pass, Renderer, Shader, run};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const TRIANGLE_SOURCE: &str = include_str!("shaders/hello_triangle.wgsl");

pub fn resize(app: &App, new_size: &winit::dpi::PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

pub fn draw(app: &App) {
    let id = app.primary_window_id();
    if let Some(frame) = app.get::<Frame>("frame.main") {
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*frame, t));
    }
}

fn setup(
    app: &App,
    windows: Vec<Arc<winit::window::Window>>,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + '_>> {
    Box::pin(async move {
        let shader = Shader::new(TRIANGLE_SOURCE)?;
        shader.set("color", [1.0, 0.2, 0.8, 1.0])?;
        let pass = Pass::from_shader("main", &shader);
        let mut frame = Frame::new();
        frame.add_pass(&pass);
        app.add("frame.main", frame);

        for win in windows {
            let target = app.get_renderer().create_target(win.clone()).await?;
            app.add_target(win.id(), target);
        }
        Ok(())
    })
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);
    app.on_start(setup)
        .on_resize(resize)
        .on_redraw_requested(draw);
    run(&mut app);
    Ok(())
}
