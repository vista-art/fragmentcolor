use fragmentcolor::{App, Frame, Pass, Renderer, Shader, run};
use winit::dpi::PhysicalSize;

const CIRCLE_SOURCE: &str = include_str!("shaders/circle.wgsl");

fn main() {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);

    app.on_start(setup)
        .on_resize(resize)
        .on_redraw_requested(draw);

    run(&mut app);
}

fn setup(
    app: &App,
    windows: Vec<std::sync::Arc<winit::window::Window>>,
) -> std::pin::Pin<
    Box<
        dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + '_,
    >,
> {
    Box::pin(async move {
        let shader = Shader::new(CIRCLE_SOURCE)?;
        shader.set("circle.radius", 300.0)?;
        shader.set("circle.color", [0.2, 0.2, 0.8, 1.0])?;
        shader.set("circle.border", 100.0)?;

        let pass = Pass::from_shader("main", &shader);
        let mut frame = Frame::new();
        frame.add_pass(&pass);

        app.add("shader.main", shader);
        app.add("frame.main", frame);

        for win in windows {
            let target = app.get_renderer().create_target(win.clone()).await?;
            app.add_target(win.id(), target);
        }

        Ok(())
    })
}

fn resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

fn draw(app: &App) {
    let id = app.primary_window_id();

    if let (Some(frame), Some(size)) = (app.get::<Frame>("frame.main"), app.window_size(id)) {
        if let Some(shader) = app.get::<Shader>("shader.main") {
            let res = [size.width as f32, size.height as f32];
            let _ = shader.set("resolution", res);
            let _ = shader.set("circle.position", [0.0f32, 0.0f32]);
        }
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*frame, t));
    }
}
