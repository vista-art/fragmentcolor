// Drawing with distance, step 7: the pointer, and knobs.

use std::sync::{Arc, Once, OnceLock};
use std::time::Instant;

use fragmentcolor::{App, AppError, Renderer, SetupResult, Shader, Target, call};
use parking_lot::Mutex;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::DeviceId;
use winit::window::Window;

const SHADER: &str = r#"
// The star and box distances, the smooth union
// and the contour coloring follow Inigo Quilez: https://iquilezles.org/articles/distfunctions2d/

// #region: params
struct Params {
    resolution: vec2<f32>,
    pointer: vec2<f32>,
    time: f32,
    blend: f32, // 0..0.4
    bands: f32, // 5..120
    inside_color: vec3<f32>,
    outside_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Params;
// #endregion: params

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

// Moves the origin to the middle of the canvas,
// turns y to point up, and scales the shorter side to one unit.
fn center(pixel: vec2<f32>) -> vec2<f32> {
    let p = (pixel - 0.5 * u.resolution) / min(u.resolution.x, u.resolution.y);
    return vec2<f32>(p.x, -p.y);
}

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);
    let pointer = center(u.pointer);
    let t = u.time;

    let d_star = star(rotate2(p - vec2<f32>(-0.25, 0.0), t * 0.4), 0.2, 5u, 3.0);
    let d_box = box(p - vec2<f32>(0.25, 0.05 * sin(t)), vec2<f32>(0.13));
    let d_dot = circle(p - pointer, 0.08);
    let d = op_smooth_union(op_smooth_union(d_star, d_box, u.blend), d_dot, u.blend);

    var color = select(u.outside_color, u.inside_color, d < 0.0);
    color = color * (1.0 - 0.75 * exp(-8.0 * abs(d)));
    color = color * (0.85 + 0.15 * cos(6.2831853 * (u.bands * d - 0.5 * t)));
    let pixel_size = fwidth(d);
    color = mix(color, vec3<f32>(1.0), 1.0 - smoothstep(pixel_size, 2.0 * pixel_size, abs(d)));
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
        "map/rotate2",
        SHADER,
    ])?;
    shader.set("u.blend", 0.12)?;
    shader.set("u.bands", 40.0)?;
    shader.set("u.inside_color", [0.38, 0.62, 0.92])?;
    shader.set("u.outside_color", [0.92, 0.6, 0.32])?;
    // #endregion: setup
    Ok(shader)
}

fn frame(
    shader: &Shader,
    renderer: &Renderer,
    target: &impl Target,
    time: f32,
    size: [f32; 2],
    pointer: [f32; 2],
) -> Result<(), AppError> {
    // #region: frame
    shader.set("u.resolution", size)?;
    shader.set("u.time", time)?;
    shader.set("u.pointer", pointer)?;
    renderer.render(shader, target)?;
    // #endregion: frame
    Ok(())
}

/// The cursor in window pixels, y down, once it has moved over the window.
static POINTER: Mutex<Option<[f32; 2]>> = Mutex::new(None);

fn track(_app: &App, _device: DeviceId, position: &PhysicalPosition<f64>) {
    *POINTER.lock() = Some([position.x as f32, position.y as f32]);
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
    static START: OnceLock<Instant> = OnceLock::new();
    let time = START.get_or_init(Instant::now).elapsed().as_secs_f32();
    let Some(shader) = app.get::<Shader>("shader") else {
        return;
    };
    let renderer = app.get_renderer();
    app.with_target(app.primary_window_id(), |target| {
        let size = target.size();
        let size = [size.width as f32, size.height as f32];
        let pointer = POINTER.lock().unwrap_or([size[0] / 2.0, size[1] / 2.0]);
        // A failing frame fails again on every redraw, so only the first error prints.
        if let Err(error) = frame(&shader, renderer, target, time, size, pointer) {
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
        .on_cursor_moved(track)
        .on_redraw_requested(draw);
    app.run();
}
