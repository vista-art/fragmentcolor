use fastrand::Rng;
use fragmentcolor::{App, Frame, Pass, Renderer, Shader, Size, run};

const TRIANGLE_SOURCE: &str = include_str!("shaders/hello_triangle.wgsl");
const CIRCLE_SOURCE: &str = include_str!("shaders/circle.wgsl");

fn on_resize(app: &App, new_size: &winit::dpi::PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

fn draw(app: &App) {
    let id = app.primary_window_id();
    if let Some(frame) = app.get::<Frame>("frame.main") {
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*frame, t));
    }
}

fn random_circle(rng: &mut Rng, size: Size, alpha: f32) -> Shader {
    let circle = Shader::new(CIRCLE_SOURCE).unwrap();
    circle
        .set("resolution", [size.width as f32, size.height as f32])
        .unwrap();

    let x = (rng.f32() * 2.0 - 1.0) * size.width as f32;
    let y = (rng.f32() * 2.0 - 1.0) * size.height as f32;
    circle.set("circle.position", [x, y]).unwrap();

    let r = rng.f32();
    let g = rng.f32();
    let b = rng.f32();
    circle.set("circle.color", [r, g, b, alpha]).unwrap();

    let radius = rng.f32() * (300.0 - 50.0) + 50.0;
    circle.set("circle.radius", radius).unwrap();

    let border = rng.f32() * (100.0 - 10.0) + 10.0;
    circle.set("circle.border", border).unwrap();

    circle
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
        // Build the pass with many objects
        let triangle = {
            let s = Shader::new(TRIANGLE_SOURCE).unwrap();
            s.set("color", [1.0, 0.2, 0.8, 1.0]).unwrap();
            s
        };
        let pass = Pass::new("Multi Object Pass");
        pass.add_shader(&triangle);

        // seed circles with a nominal size
        let mut rng = Rng::new();
        let size = Size {
            width: 800,
            height: 600,
            depth: None,
        };
        for _ in 0..50 {
            let circle = random_circle(&mut rng, size, 1.0);
            pass.add_shader(&circle);
        }

        // Compose into a frame (single pass)
        let mut frame = Frame::new();
        frame.add_pass(&pass);

        app.add("frame.main", frame);

        // Create targets for all windows
        for win in windows {
            let target = app.get_renderer().create_target(win.clone()).await?;
            app.add_target(win.id(), target);
        }

        Ok(())
    })
}

fn main() {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);

    app.on_start(setup)
        .on_resize(on_resize)
        .on_redraw_requested(draw);

    run(&mut app);
}
