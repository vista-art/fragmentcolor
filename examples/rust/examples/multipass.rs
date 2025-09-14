use fastrand::Rng;
use fragmentcolor::{App, Frame, Pass, Shader, Size, run};

const CIRCLE_SOURCE: &str = include_str!("circle.wgsl");
const TRIANGLE_SOURCE: &str = include_str!("hello_triangle.wgsl");

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

pub fn on_resize(app: &App, new_size: &winit::dpi::PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
    let id = app.window_id();
    let res = [new_size.width as f32, new_size.height as f32];
    let _ = app.set_uniform(id, "resolution", res);
}

pub fn on_draw(_app: &App) {}

fn main() {
    // Build passes
    let triangle = {
        let s = Shader::new(TRIANGLE_SOURCE).unwrap();
        s.set("color", [1.0, 0.2, 0.8, 1.0]).unwrap();
        s
    };
    let opaque_pass = {
        let p = Pass::new("Opaque Pass");
        p.add_shader(&triangle);
        p.set_clear_color([0.0, 0.0, 0.0, 1.0]);
        p
    };
    let transparent_pass = Pass::new("Transparent Pass");

    // Seed circles using a nominal size; example updates on resize
    let mut rng = Rng::new();
    let size = Size {
        width: 800,
        height: 600,
        depth: None,
    };
    for _ in 0..10 {
        let circle = random_circle(&mut rng, size, 1.0);
        opaque_pass.add_shader(&circle);
    }
    for _ in 0..20 {
        let circle = random_circle(&mut rng, size, 0.2);
        transparent_pass.add_shader(&circle);
    }

    let mut frame = Frame::new();
    frame.add_pass(&opaque_pass);
    frame.add_pass(&transparent_pass);

    let mut app = App::new();
    app.scene(frame)
        .on_resize(on_resize)
        .on_redraw_requested(on_draw);

    run(&mut app);
}
