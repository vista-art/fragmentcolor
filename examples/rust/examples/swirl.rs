use fragmentcolor::{App, Pass, Renderer, SetupResult, Shader, call, run};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

fn resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

fn draw(app: &App) {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();

    let id = app.primary_window_id();

    if let (Some(pass), Some(size)) = (app.get::<Pass>("pass.main"), app.window_size(id)) {
        if let Some(shader) = app.get::<Shader>("shader.main") {
            let res = [size.width as f32, size.height as f32];
            let _ = shader.set("resolution", res);
            let t = START.get_or_init(Instant::now).elapsed().as_secs_f32();
            let _ = shader.set("time", t);
        }
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*pass, t));
    }
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let src = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/shaders/swirl.wgsl");
    let shader = Shader::new(src)?;
    let pass = Pass::from_shader("main", &shader);

    app.add("shader.main", shader);
    app.add("pass.main", pass);

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);

    app.on_start(call!(setup))
        .on_resize(resize)
        .on_redraw_requested(draw);

    run(&mut app);
    Ok(())
}
