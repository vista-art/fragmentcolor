// Step 5 — one triangle, many instances, a tiny particle field.
//
// Same gradient triangle from step 2, but now we throw a few thousand
// copies of it onto the screen with per-instance offsets and let the GPU
// wobble them around using a single time uniform. This is the moment the
// 4-object model earns its keep: one mesh, one shader, one render call,
// thousands of triangles.

use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{App, Renderer, SetupResult, Shader, Target, call};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const PARTICLE_COUNT: usize = 1500;

// #region: shader
const PARTICLE_WGSL: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,    // per-vertex
    @location(1) center: vec2<f32>,       // per-instance
    @location(2) phase: f32,               // per-instance
    @location(3) tint: vec3<f32>,          // per-instance
    @location(4) color: vec3<f32>,         // per-vertex
) -> VOut {
    let scale = 0.045;
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase) * 0.05,
        cos(time * 0.9 + phase * 1.4) * 0.05,
    );
    var world = position.xy * scale + center + wobble;
    // Aspect-correct so each particle's triangle stays equilateral and
    // the field stays evenly distributed regardless of canvas shape.
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    if (aspect > 1.0) { world.x = world.x / aspect; }
    else              { world.y = world.y * aspect; }

    // Same oscillation as before, routed through `easing/in_out_sine`
    // (a registry slug we pulled in at Shader::new) to soften the
    // peaks and troughs into a slower-feeling pulse.
    let raw = 0.5 + 0.5 * sin(time * 2.0 + phase);
    let glow = 0.4 + 0.6 * in_out_sine(raw);

    var out: VOut;
    out.pos = vec4<f32>(world, 0.0, 1.0);
    out.color = color * tint * glow;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;
// #endregion: shader

// #region: setup
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    // Pull `easing/in_out_sine` from the catalog and link it with our
    // particle source. Same Shader::new constructor as step 2, just one
    // more slug.
    let shader = Shader::new(["easing/in_out_sine", PARTICLE_WGSL])?;
    shader.set("time", 0.0_f32)?;

    // The base triangle: same equilateral shape as step 4.
    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        Vertex::new([0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        Vertex::new([0.0, 0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);

    // Many instances of that same triangle, sprinkled across NDC space.
    let mut instances = Vec::with_capacity(PARTICLE_COUNT);
    for _ in 0..PARTICLE_COUNT {
        let cx = fastrand::f32() * 1.8 - 0.9;
        let cy = fastrand::f32() * 1.8 - 0.9;
        let phase = fastrand::f32() * std::f32::consts::TAU;
        let tint = [
            0.6 + fastrand::f32() * 0.4,
            0.6 + fastrand::f32() * 0.4,
            0.6 + fastrand::f32() * 0.4,
        ];
        // Use a Vertex template so instance properties get auto-incrementing
        // locations starting at 1 — clear of the vertex `position` slot at
        // @location(0).
        instances.push(
            Vertex::new([0.0, 0.0])
                .set("center", [cx, cy])
                .set("phase", phase)
                .set("tint", tint),
        );
    }
    mesh.add_instances(instances);

    shader.add_mesh(&mesh)?;
    app.add("shader.main", shader);
    app.add("mesh.main", mesh);

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
    app.run();
    Ok(())
}
