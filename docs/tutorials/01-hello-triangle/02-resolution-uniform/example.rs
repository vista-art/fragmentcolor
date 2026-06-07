// Step 3 — a second uniform.
//
// The triangles in steps 1 and 2 squished as the window resized: NDC
// maps [-1, 1] across each axis independently, so positions hardcoded
// in clip space stretch with the canvas. The fix is information the
// shader doesn't have yet — the canvas aspect ratio. We add a second
// uniform, `resolution`, set it from the target's pixel size each
// frame, and let the vertex stage aspect-correct the positions before
// output. Same `Shader` API, one more `set("...", ...)` call per
// frame.

use fragmentcolor::{App, Renderer, SetupResult, Shader, Target, call};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// #region: shader
const RESOLUTION_TRIANGLE_WGSL: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    // Equilateral triangle in NDC, centroid at origin; aspect-corrected below.
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.4),
        vec2<f32>( 0.7, -0.4),
        vec2<f32>( 0.0,  0.8),
    );
    // Aspect-correct so the triangle keeps its shape on any canvas.
    // `max(.., 1.0)` keeps things sane if `resolution` hasn't been set
    // yet (uniforms default to zero in WGSL).
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    var pos = p[i];
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }

    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
"#;
// #endregion: shader

// #region: setup
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let shader = Shader::new(RESOLUTION_TRIANGLE_WGSL)?;
    shader.set("color", [0.95, 0.30, 0.42, 1.0])?;
    app.add("shader.main", shader);

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }
    Ok(())
}
// #endregion: setup

fn resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

// #region: frame
fn draw(app: &App) {
    static START: OnceLock<Instant> = OnceLock::new();
    let time = START.get_or_init(Instant::now).elapsed().as_secs_f32();

    if let Some(shader) = app.get::<Shader>("shader.main") {
        let r = 0.5 + 0.45 * (time * 0.7).sin();
        let g = 0.5 + 0.45 * (time * 0.5 + 1.7).cos();
        let b = 0.5 + 0.45 * (time * 0.9 + 3.1).sin();
        let _ = shader.set("color", [r, g, b, 1.0]);

        let id = app.primary_window_id();
        let renderer = app.get_renderer();
        let _ = app.with_target(id, |target| {
            // The new uniform is updated from the target's pixel size,
            // every frame, so resizing the window keeps the triangle
            // proportions correct.
            let s = target.size();
            let _ = shader.set("resolution", [s.width as f32, s.height as f32]);
            renderer.render(&*shader, target)
        });
    }
}
// #endregion: frame

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);
    app.on_start(call!(setup))
        .on_resize(resize)
        .on_redraw_requested(draw);
    app.run();
    Ok(())
}
