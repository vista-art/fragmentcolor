use plrender::{
    app::window::Window, scene::Scene, AppOptions, Event, IsWindow, PLRender, RendererOptions,
    Sprite,
};
use std::path::Path;

pub const ROOT: &'static str = env!("CARGO_MANIFEST_DIR");

fn main() {
    PLRender::config(options());

    let window = Window::default();

    let mut scene = Scene::new();

    let mut sprite = Sprite::new(Path::new(ROOT).join("assets/images/pikachu.png"));

    scene.add(&mut sprite);

    scene.target(&window);

    scene.print();

    let state = window.state();
    window.on("draw", move |event| match event {
        Event::Draw => {
            _ = scene.render();

            let window = state.read().unwrap();
            window.redraw();
        }
        _ => {}
    });

    PLRender::run();
}

fn options() -> AppOptions {
    AppOptions {
        renderer: RendererOptions {
            panic_on_error: true,
            render_pass: "toy".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}
