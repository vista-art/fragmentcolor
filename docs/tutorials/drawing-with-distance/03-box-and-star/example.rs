// Drawing with distance step 3: corners from a box and a star.

use std::sync::{Arc, Once};

use fragmentcolor::{App, AppError, Renderer, SetupResult, Shader, Target, call};
use winit::dpi::PhysicalSize;
use winit::window::Window;

const SHADER: &str = r#"
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

// #region: center
// Moves the origin to the middle of the canvas,
// turns y to point up, and scales the shorter side to one unit.
fn center(pixel: vec2<f32>) -> vec2<f32> {
    let p = (pixel - 0.5 * resolution) / min(resolution.x, resolution.y);
    return vec2<f32>(p.x, -p.y);
}
// #endregion: center

// #region: show
// Magenta inside, azure outside, and a stripe every 0.05 units of distance.
fn show(d: f32) -> vec3<f32> {
    let tint = select(vec3<f32>(0.0, 0.514, 1.0), vec3<f32>(1.0, 0.0, 0.514), d < 0.0);
    let stripe = step(0.5, fract(d * 20.0));
    return tint * (0.3 + 0.45 * stripe);
}
// #endregion: show

// #region: box
// Signed distance to a box centered on the origin.
// half_size holds half its width and half its height.
fn box(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let q = abs(p) - half_size;
    let outside = length(max(q, vec2<f32>(0.0)));
    let inside = min(max(q.x, q.y), 0.0);
    return outside + inside;
}
// #endregion: box

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);
    let d_box = box(p - vec2<f32>(-0.25, 0.0), vec2<f32>(0.2, 0.15));
    let d_star = star(p - vec2<f32>(0.25, 0.0), 0.2, 5u, 3.0);
    let d = min(d_box, d_star);
    return vec4<f32>(show(d), 1.0);
}
// #endregion: paint
"#;

fn setup() -> Result<Shader, AppError> {
    // #region: setup
    let shader = Shader::new(["sdf2d/star", SHADER])?;
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
    app.resize([size.width, size.height]);
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
