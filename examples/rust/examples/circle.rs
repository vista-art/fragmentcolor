use fragmentcolor::{App, Shader, run};
use winit::dpi::PhysicalSize;

const CIRCLE_SOURCE: &str = include_str!("circle.wgsl");

pub fn on_resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
    let id = app.window_id();
    let res = [new_size.width as f32, new_size.height as f32];
    let _ = app.set_uniform(id, "resolution", res);
}

pub fn on_draw(app: &App) {
    let id = app.window_id();

    if let Some(size) = app.size(id) {
        let resolution = [size.width as f32, size.height as f32];
        let _ = app.set_uniform(id, "resolution", resolution);
        let _ = app.set_uniform(id, "circle.position", [0.0f32, 0.0f32]);
    }
}

fn main() {
    let shader = Shader::new(CIRCLE_SOURCE).unwrap();
    shader.set("circle.radius", 300.0).unwrap();
    shader.set("circle.color", [0.2, 0.2, 0.8, 1.0]).unwrap();
    shader.set("circle.border", 100.0).unwrap();

    let mut app = App::new();
    app.scene(shader)
        .on_resize(on_resize)
        .on_redraw_requested(on_draw);

    run(&mut app);
}
