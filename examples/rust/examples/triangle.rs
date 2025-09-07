use fragmentcolor::{App, Shader, run};

const TRIANGLE_SOURCE: &str = include_str!("hello_triangle.wgsl");

pub fn on_resize(app: &App, new_size: &winit::dpi::PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

pub fn on_draw(_app: &App) {}

fn main() {
    let shader = {
        let s = Shader::new(TRIANGLE_SOURCE).unwrap();
        s.set("color", [1.0, 0.2, 0.8, 1.0]).unwrap();
        s
    };

    let mut app = App::new();
    app.scene(shader)
        .on_resize(on_resize)
        .on_redraw_requested(on_draw);

    run(&mut app);
}
