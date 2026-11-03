// Drawing with distance step 1: a coordinate for every pixel.

use std::sync::{Arc, Once};

use fragmentcolor::{App, AppError, Renderer, SetupResult, Shader, Target, call};
use winit::dpi::PhysicalSize;
use winit::window::Window;

const SHADER: &str = r#"
// #region: triangle
@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

// One triangle, big enough to cover the whole canvas.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(corners[i], 0.0, 1.0);
}
// #endregion: triangle

// #region: center
// Moves the origin to the middle of the canvas,
// turns y to point up, and scales the shorter side to one unit.
fn center(pixel: vec2<f32>) -> vec2<f32> {
    let p = (pixel - 0.5 * resolution) / min(resolution.x, resolution.y);
    return vec2<f32>(p.x, -p.y);
}
// #endregion: center

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);
    var color = vec3<f32>(0.5 + p.x, 0.5 + p.y, 0.5);
    if max(abs(p.x), abs(p.y)) > 0.5 {
        color = color * 0.3;
    }
    return vec4<f32>(color, 1.0);
}
// #endregion: paint
"#;

fn setup() -> Result<Shader, AppError> {
    // #region: setup
    let shader = Shader::new(SHADER)?;
    // #endregion: setup
    Ok(shader)
}

fn frame(
    shader: &Shader,
    renderer: &Renderer,
    target: &impl Target,
    size: [f32; 2],
) -> Result<(), AppError> {
    // #region: frame
    shader.set("resolution", size)?;
    renderer.render(shader, target)?;
    // #endregion: frame
    Ok(())
}

async fn start(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let shader = setup()?;
    for window in windows {
        let target = app.get_renderer().create_target(window.clone()).await?;
        app.add_target(window.id(), target);
    }
    app.add("shader", shader);
    Ok(())
}

fn resize(app: &App, size: &PhysicalSize<u32>) {
    if let Err(error) = app.resize([size.width, size.height]) {
        eprintln!("resize failed: {error}");
    }
}

fn draw(app: &App) {
    let Some(shader) = app.get::<Shader>("shader") else {
        return;
    };
    let renderer = app.get_renderer();
    app.with_target(app.primary_window_id(), |target| {
        let size = target.size();
        let size = [size.width as f32, size.height as f32];
        // A failing frame fails again on every redraw, so only the first error prints.
        if let Err(error) = frame(&shader, renderer, target, size) {
            static FAILED: Once = Once::new();
            FAILED.call_once(|| eprintln!("{error}"));
        }
    });
}

fn main() {
    env_logger::init();
    let mut app = App::new(Renderer::new());
    app.on_start(call!(start))
        .on_resize(resize)
        .on_redraw_requested(draw);
    app.run();
}
