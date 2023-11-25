use plrender::{components::Circle, AppOptions, PLRender, RendererOptions, Scene, Sprite, Window};
use plrender::{CircleOptions, Color, Event, IsWindow};
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    PLRender::config(options());
    let window = Window::default();

    let i_am_a_video_element = Path::new(ROOT).join("assets/images/pikachu.png");

    let mut scene = Scene::new();
    let mut video = Sprite::new(i_am_a_video_element);
    scene.add(&mut video);

    let mut gaze = Circle::new(CircleOptions {
        radius: 15.0,
        color: Color(0xFF0000FF),
        border: 5.0,
    });
    scene.add(&mut gaze);

    scene.target(&window);

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

pub fn options() -> AppOptions {
    AppOptions {
        renderer: RendererOptions {
            panic_on_error: true,
            render_pass: "toy".to_string(),
            ..Default::default()
        },
        log_level: "debug".to_string(),
    }
}
