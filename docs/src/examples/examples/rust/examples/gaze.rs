use fragmentcolor::{
    components::Circle, AppOptions, FragmentColor, RendererOptions, Scene, Sprite, Window,
};
use fragmentcolor::{CircleOptions, Color, Dimensions, Event, IsWindow, Shader};
use instant::Instant;
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    FragmentColor::config(options());
    let window = Window::default();
    let mut scene = Scene::new();

    let fake_video_element = Path::new(ROOT).join("assets/images/test.jpg");

    let mut video = Sprite::new(fake_video_element);
    scene.add(&mut video);

    let mut shader = Shader::new(
        "
        SHADER SOURCE GOES HERE.

        This Shader component works. It tells the renderer to use the custom
        shader section of the `toy.wgsl` file.

        The function is called `shadertoy_main` and it takes a `vec2` as input.

        For now, you can convert from ShaderToy using https://eliotbo.github.io/glsl2wgsl
        And paste it in `toy.wgsl` in the area reserved for shader composition.

        In the future, you can paste it directly here, or load from a database,
        and it will convert it automatically.
    ",
    );
    scene.add(&mut shader);

    let mut gaze = Circle::new(CircleOptions {
        radius: 50.0,
        color: Color(0xFF0000ff),
        border: 10.0,
    });
    scene.add(&mut gaze);

    let mut square = Square::new(50);
    square.set_color(Color(0x00ff0088));
    scene.add(&mut square);

    square.set_parent(&gaze);

    scene.target(&window); // equivalent to:
                           // --------------
                           // scene.target(&RenderTargetDescription {
                           //     target_id: TargetId::Window(window.id()),
                           //     camera_id: None,
                           //     target_size: window.size(),
                           //     clear_color: Color(0x00000000),
                           //     before_render: None,
                           //     after_render: None,
                           // });

    let now = Instant::now();

    let state = window.state();
    window.on("draw", move |event| match event {
        Event::Draw => {
            let elapsed = now.elapsed();
            let window = state.read().unwrap();

            let x = window.size().width() as f32 * (0.5 * (1.0 + f32::sin(elapsed.as_secs_f32())));
            let y = window.size().height() as f32 * (0.5 * (1.0 + f32::cos(elapsed.as_secs_f32())));
            gaze.set_position([x, y]);
            square.set_position([x, y]);
            video.set_position([x, y]);
            _ = scene.render();

            window.redraw();
        }
        _ => {}
    });

    FragmentColor::run();
}

pub fn options() -> AppOptions {
    AppOptions {
        renderer: RendererOptions {
            panic_on_error: true,
            render_pass: "toy".to_string(),
            ..Default::default()
        },
        log_level: "error".to_string(),
    }
}
