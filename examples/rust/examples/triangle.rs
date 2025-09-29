use fragmentcolor::{App, Frame, Pass, Renderer, SetupResult, Shader, run};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const TRIANGLE_SOURCE: &str = include_str!("shaders/hello_triangle.wgsl");

pub fn resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

pub fn draw(app: &App) {
    let id = app.primary_window_id();
    if let Some(frame) = app.get::<Frame>("frame.main") {
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*frame, t));
    }
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
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
