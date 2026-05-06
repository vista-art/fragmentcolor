// Step 2 — compose with the registry.
//
// Same `Shader::new` constructor as step 1, only this time we pass an
// array: a registry slug (`noise/simplex2`) followed by an inline source
// that calls into it. FragmentColor fetches the slug from the embedded
// registry, validates the merged source with naga, and links it all into
// one program. The triangle still uses the `color` uniform from step 1
// — we just multiply it by a soft simplex-noise breath so the surface
// shimmers.

use fragmentcolor::{App, Renderer, SetupResult, Shader, Target, call, run};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// #region: shader
const NOISY_TRIANGLE_WGSL: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@group(0) @binding(2) var<uniform> time: f32;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p  = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.4),
        vec2<f32>( 0.7, -0.4),
        vec2<f32>( 0.0,  0.8),
    );
    // Aspect-correct (same trick as step 2) so the triangle keeps shape.
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    var pos = p[i];
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }
    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    // Use the original (pre-correction) positions as the UV input for noise.
    out.uv = (p[i] + vec2<f32>(1.0)) * 0.5;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    // simplex2 was pulled in by slug; we just call it.
    let n = simplex2(in.uv * 6.0 + vec2<f32>(time * 0.4)) * 0.5 + 0.5;
    let breath = 0.55 + 0.45 * n;
    return vec4<f32>(color.rgb * breath, color.a);
}
"#;
// #endregion: shader

// #region: setup
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    // Same Shader::new — now with a registry slug followed by an inline
    // main source. The slug resolves to the embedded WGSL function
    // `simplex2`, which our main source calls on line 14 of the fragment.
    let shader = Shader::new(["noise/simplex2", NOISY_TRIANGLE_WGSL])?;
    shader.set("color", [0.95, 0.30, 0.42, 1.0])?;
    shader.set("time", 0.0_f32)?;
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
        // Same color animation as step 1; the noise breath is added by the shader.
        let r = 0.5 + 0.45 * (time * 0.7).sin();
        let g = 0.5 + 0.45 * (time * 0.5 + 1.7).cos();
        let b = 0.5 + 0.45 * (time * 0.9 + 3.1).sin();
        let _ = shader.set("color", [r, g, b, 1.0]);
        let _ = shader.set("time", time);

        let id = app.primary_window_id();
        let renderer = app.get_renderer();
        let _ = app.with_target(id, |target| {
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
    run(&mut app);
    Ok(())
}
