// Drawing with distance, step 5: coloring a distance field.

use std::sync::{Arc, Once};

use fragmentcolor::{App, AppError, Renderer, SetupResult, Shader, Target, call};
use winit::dpi::PhysicalSize;
use winit::window::Window;

const SHADER: &str = r#"
// The contour coloring follows Inigo Quilez:
// https://iquilezles.org/articles/distfunctions2d/

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

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);
    // #region: scene
    let d_star = star(p - vec2<f32>(-0.25, 0.0), 0.2, 5u, 3.0);
    let d_box = box(p - vec2<f32>(0.25, 0.0), vec2<f32>(0.13));
    let d_dot = circle(p, 0.08);
    let d = op_smooth_union(op_smooth_union(d_star, d_box, 0.12), d_dot, 0.12);
    // #endregion: scene

    // #region: color
    let inside = vec3<f32>(0.38, 0.62, 0.92);
    let outside = vec3<f32>(0.92, 0.6, 0.32);
    var color = select(outside, inside, d < 0.0);
    color = color * (1.0 - 0.75 * exp(-8.0 * abs(d)));
    color = color * (0.85 + 0.15 * cos(6.2831853 * 40.0 * d));
    let pixel_size = fwidth(d);
    color = mix(color, vec3<f32>(1.0), 1.0 - smoothstep(pixel_size, 2.0 * pixel_size, abs(d)));
    // #endregion: color
    return vec4<f32>(color, 1.0);
}
// #endregion: paint
"#;

fn setup() -> Result<Shader, AppError> {
    // #region: setup
    let shader = Shader::new([
        "sdf2d/circle",
        "sdf2d/box",
        "sdf2d/star",
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
        // A frame that fails fails again on every redraw, so only the first error prints.
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
