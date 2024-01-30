use fragmentcolor::{
    app::window::Window, scene::Scene, AppOptions, Event, FragmentColor, IsWindow, RendererOptions,
    Sprite,
};
use std::path::Path;

pub const ROOT: &'static str = env!("CARGO_MANIFEST_DIR");

fn main() {
    FragmentColor::config(options());

    let mut window = Window::default();

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

    FragmentColor::run();
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
