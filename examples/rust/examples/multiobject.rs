use fastrand::Rng;
use fragmentcolor::{App, Pass, Shader, Size, run};

const TRIANGLE_SOURCE: &str = include_str!("hello_triangle.wgsl");
const CIRCLE_SOURCE: &str = include_str!("circle.wgsl");

pub fn on_resize(app: &App, new_size: &winit::dpi::PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
    let id = app.window_id();
    let res = [new_size.width as f32, new_size.height as f32];
    let _ = app.set_uniform(id, "resolution", res);
}

pub fn on_draw(_app: &App) {}

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

fn main() {
    // Build the pass with many objects
    let triangle = {
        let s = Shader::new(TRIANGLE_SOURCE).unwrap();
        s.set("color", [1.0, 0.2, 0.8, 1.0]).unwrap();
        s
    };
    let pass = Pass::new("Multi Object Pass");
    pass.add_shader(&triangle);

    // seed circles with a nominal size; example will update on resize
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

    let mut app = App::new();
    app.scene(pass)
        .on_resize(on_resize)
        .on_redraw_requested(on_draw);

    run(&mut app);
}
