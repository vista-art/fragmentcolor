use fragmentcolor::{
    components::Circle, AppOptions, FragmentColor, RendererOptions, Scene, Sprite, Window,
};
use fragmentcolor::{CircleOptions, Color, Dimensions, Event, IsWindow, Quad, Shader, Square};
use instant::{Duration, Instant};
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    FragmentColor::config(options());
    let mut window = Window::default();
    window.set_title("Window 1");
    //let window2 = Window::default();

    let mut scene = Scene::new(); // API:
                                  // scene.add();     adds object to scene
                                  // scene.target();  sets window or sprite as render target
                                  // scene.render();  renders scene to target

    // We have Sprite, Shader, Circle, Square
    let fake_video = Path::new(ROOT).join("assets/images/test.jpg");
    let mut video = Sprite::new(fake_video);

    // new Sprite()   // plr.Sprite("")

    scene.add(&mut video);

    let mut shader = Shader::new(
        "
        SHADER SOURCE GOES HERE.

        This Shader component tells the renderer to use the custom
        shader section of the `toy.wgsl` file.

        The function is called `shadertoy_main` and it takes a `vec2` as input.

        In the future, you can paste ShaderToy code directly here, or load from a database,
        and it will convert automatically.

        For now, you can convert from ShaderToy using https://eliotbo.github.io/glsl2wgsl
        And paste it in `toy.wgsl` in the area reserved for shader composition.

        Only static shaders are supported at this point.
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
    square.set_border(15.0);

    square.set_parent(&gaze); // square is now a child of gaze

    scene.add(&mut square);

    //scene.target(&window2);
    scene.target(&window); //
                           // equivalent to:
                           //
                           // scene.target(&RenderTargetDescription {
                           //     target_id: TargetId::Window(window.id()),
                           //     camera_id: None,
                           //     target_size: window.size(),
                           //     clear_color: Color(0x0000ffff),
                           //     before_render: None,
                           //     after_render: None,
                           // });

    let now = Instant::now();

    let state = window.state();
    window.on("draw", move |event| match event {
        Event::Draw => {
            let window = state.read().unwrap();
            let elapsed = now.elapsed();

            let (x, y) = position_for_time(elapsed, window.size());

            gaze.set_position([x, y]);

            scene.render();

            window.redraw();
        }
        _ => {}
    });

    FragmentColor::run();
}

fn position_for_time(elapsed: Duration, quad: Quad) -> (f32, f32) {
    (
        quad.width_f32() * (0.5 * (1.0 + f32::sin(elapsed.as_secs_f32()))),
        quad.height_f32() * (0.5 * (1.0 + f32::cos(elapsed.as_secs_f32()))),
    )
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
