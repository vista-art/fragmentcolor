// Step 4 — postfx, the right way.
//
// Same particles from step 3, but instead of going straight to the canvas
// they pass through an intermediate texture and then a second shader that
// adds a vignette and a touch of film grain. This is where the last two
// objects in the core API show up:
//
//   - Pass    — the unit of "render this to that"; lets you chain effects.
//   - Texture — the bridge between two passes (or a sampleable input).
//
// And while we're at it, we use the shader composition system: the postfx
// shader is built from two helper functions pulled straight out of the
// public registry (`postfx/vignette` and `postfx/film_grain`) plus a small
// main shader that ties them together.

use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{App, Pass, Renderer, SetupResult, Shader, call};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const PARTICLE_COUNT: usize = 1500;
const SCENE_SIZE: u32 = 1024;

// #region: particle-shader
const PARTICLE_WGSL: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> time: f32;

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
    let world = position.xy * scale + center + wobble;
    // Same easing-driven pulse as step 4.
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
// #endregion: particle-shader

// #region: postfx-shader
const POSTFX_MAIN_WGSL: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    // Fullscreen triangle that covers the whole canvas.
    var p  = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0., 1.), vec2<f32>(2., 1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> time: f32;

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    // Sample the same texture three times at slightly offset UVs — one
    // sample per channel — for a subtle film-style chromatic split.
    let off = chromatic_offsets(in.uv, 0.006);
    let r = textureSample(scene, samp, off[0]).r;
    let g = textureSample(scene, samp, off[1]).g;
    let b = textureSample(scene, samp, off[2]).b;
    var color = vec3<f32>(r, g, b);

    // Hejl/Burgess-Dawson filmic curve to roll off highlights.
    color = tonemap_filmic(color);

    // Edge darkening + a touch of frame-correlated grain.
    let v_mask = vignette(in.uv, 0.55, 0.40);
    let grain  = film_grain(in.uv, time) * 0.05;

    return vec4<f32>(color * v_mask + grain, 1.0);
}
"#;
// #endregion: postfx-shader

// #region: setup
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    // Particle shader and mesh — same as step 4.
    let particle_shader = Shader::new(["easing/in_out_sine", PARTICLE_WGSL])?;
    particle_shader.set("time", 0.0_f32)?;

    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        Vertex::new([0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        Vertex::new([0.0, 0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);

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

    // Bind the mesh to the particle shader so the renderer knows the
    // vertex layout. Pass::add_mesh below registers the mesh with the
    // pass; this call registers it with the shader so the slug-composed
    // shader picks up the per-vertex/per-instance attribute layout.
    particle_shader.add_mesh(&mesh)?;

    // Intermediate texture target — the bridge between the two passes.
    // The postfx shader holds a Texture handle pointing at the same GPU
    // resource, and the renderer's texture pool retains an Arc to it as
    // soon as `intermediate.texture()` is called, so the underlying GPU
    // texture stays alive after this function returns.
    let intermediate = app
        .get_renderer()
        .create_texture_target([SCENE_SIZE, SCENE_SIZE])
        .await?;
    let scene_texture = intermediate.texture();

    // Postfx shader composed from four registry slugs and a main source.
    // Each slug is a pure WGSL helper function; Shader::new concatenates
    // them with our main shader and hands the merged source to naga for
    // validation.
    let postfx_shader = Shader::new([
        "postfx/chromatic_offsets",
        "color/tonemap_filmic",
        "postfx/vignette",
        "postfx/film_grain",
        POSTFX_MAIN_WGSL,
    ])?;
    postfx_shader.set("scene", &scene_texture)?;
    postfx_shader.set("time", 0.0_f32)?;

    // Pass 1: render the particles INTO the intermediate texture.
    let particle_pass = Pass::from_shader("particles", &particle_shader);
    particle_pass.add_mesh(&mesh)?;
    particle_pass.set_target(&intermediate)?;
    particle_pass.set_clear_color([0.04, 0.05, 0.08, 1.0]);

    // Pass 2: sample the intermediate, apply postfx, render to the canvas.
    let postfx_pass = Pass::from_shader("postfx", &postfx_shader);
    postfx_pass.require(&particle_pass)?;

    app.add("shader.particle", particle_shader);
    app.add("shader.postfx", postfx_shader);
    app.add("pass.particle", particle_pass);
    app.add("pass.postfx", postfx_pass);
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

    if let (Some(particle_shader), Some(postfx_shader), Some(particle_pass), Some(postfx_pass)) = (
        app.get::<Shader>("shader.particle"),
        app.get::<Shader>("shader.postfx"),
        app.get::<Pass>("pass.particle"),
        app.get::<Pass>("pass.postfx"),
    ) {
        let _ = particle_shader.set("time", time);
        let _ = postfx_shader.set("time", time);

        let passes = vec![(*particle_pass).clone(), (*postfx_pass).clone()];
        let id = app.primary_window_id();
        let renderer = app.get_renderer();
        let _ = app.with_target(id, |target| renderer.render(&passes, target));
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
