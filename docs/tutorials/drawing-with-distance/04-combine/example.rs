// Drawing with distance step 4: four ways to combine shapes.

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

// #region: pair
// The two shapes of one pair, measured from the middle of its quarter.
fn disc(q: vec2<f32>) -> f32 {
    return circle(q - vec2<f32>(-0.07, 0.05), 0.1);
}

fn square(q: vec2<f32>) -> f32 {
    return box(q - vec2<f32>(0.04, -0.03), vec2<f32>(0.11));
}
// #endregion: pair

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);

    // Each quarter of the unit square holds one pair, combined its own way.
    let upper_left = p - vec2<f32>(-0.25, 0.25);
    let upper_right = p - vec2<f32>(0.25, 0.25);
    let lower_left = p - vec2<f32>(-0.25, -0.25);
    let lower_right = p - vec2<f32>(0.25, -0.25);

    let joined = op_union(disc(upper_left), square(upper_left));
    let overlap = op_intersect(disc(upper_right), square(upper_right));
    let carved = op_subtract(square(lower_left), disc(lower_left));
    let blended = op_smooth_union(disc(lower_right), square(lower_right), 0.08);

    let d = min(min(joined, overlap), min(carved, blended));
    return vec4<f32>(show(d), 1.0);
}
// #endregion: paint
"#;

fn setup() -> Result<Shader, AppError> {
    // #region: setup
    let shader = Shader::new([
        "sdf2d/circle",
        "sdf2d/box",
        "sdf/op_union",
        "sdf/op_intersect",
        "sdf/op_subtract",
        "sdf/op_smooth_union",
        SHADER,
    ])?;
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
