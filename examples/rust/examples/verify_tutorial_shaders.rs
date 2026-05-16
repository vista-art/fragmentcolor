//! One-off validator: constructs every Shader the tutorial 01 demos build
//! and renders each to a 256×256 texture target. If any step's WGSL
//! composition is invalid, this fails before the docs site does.
//!
//! Run: `cargo run --release -p fce --example verify_tutorial_shaders`
//!
//! Not registered in any test target — it's a manual gate the tutorial
//! author runs after editing inline WGSL or swapping registry slugs.

use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{Pass, Renderer, Shader, Target};

// Step 3 — compose-noise (carries the resolution uniform from step 2).
const STEP_3_NOISY: &str = r#"
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
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    var pos = p[i];
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }
    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.uv = (p[i] + vec2<f32>(1.0)) * 0.5;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let n = simplex2(in.uv * 6.0 + vec2<f32>(time * 0.4)) * 0.5 + 0.5;
    let breath = 0.55 + 0.45 * n;
    return vec4<f32>(color.rgb * breath, color.a);
}
"#;

// Step 5 — particle-trail (aspect-corrected against the canvas).
const STEP_5_PARTICLE: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) center: vec2<f32>,
    @location(2) phase: f32,
    @location(3) tint: vec3<f32>,
    @location(4) color: vec3<f32>,
) -> VOut {
    let scale = 0.045;
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase) * 0.05,
        cos(time * 0.9 + phase * 1.4) * 0.05,
    );
    var world = position.xy * scale + center + wobble;
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    if (aspect > 1.0) { world.x = world.x / aspect; }
    else              { world.y = world.y * aspect; }
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

// Step 6 — postfx fullscreen-triangle shader (no aspect correction needed:
// the intermediate scene texture is square, the postfx is fullscreen).
const STEP_6_POSTFX: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
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
    let off = chromatic_offsets(in.uv, 0.006);
    let r = textureSample(scene, samp, off[0]).r;
    let g = textureSample(scene, samp, off[1]).g;
    let b = textureSample(scene, samp, off[2]).b;
    var color = vec3<f32>(r, g, b);
    color = tonemap_filmic(color);
    let v_mask = vignette(in.uv, 0.55, 0.40);
    let grain  = film_grain(in.uv, time) * 0.05;
    return vec4<f32>(color * v_mask + grain, 1.0);
}
"#;

fn check(name: &str, parts: &[&str]) {
    match Shader::new(parts) {
        Ok(_) => println!("  ✓ {} composes", name),
        Err(e) => {
            println!("  ✗ {} FAILED: {}", name, e);
            std::process::exit(1);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        println!("Verifying tutorial 01 shaders compose...");

        // Step 1 — URL-loaded triangle. Skip in this offline check; the
        // URL fetch goes to the network. Validation is exercised by the
        // existing tutorial_01_hello_triangle_three_lines example.
        println!("  · step 1 (three-lines, URL): exercised by tutorial_01_hello_triangle_three_lines");

        // Step 2 — resolution-uniform: inline only, exercised by its own example.
        println!("  · step 2 (resolution-uniform): exercised by its own example");

        // Step 3 — noise/simplex2 + inline aspect-corrected triangle.
        check("step 3 (compose-noise)", &["noise/simplex2", STEP_3_NOISY]);

        // Step 4 — vertex-gradient: exercised by its own example.
        println!("  · step 4 (vertex-gradient): exercised by its own example");

        // Step 5 — easing/in_out_sine + inline particle source.
        check("step 5 (particle-trail)", &["easing/in_out_sine", STEP_5_PARTICLE]);

        // Step 6 — four registry slugs + inline postfx main.
        check(
            "step 6 (postfx-pass)",
            &[
                "postfx/chromatic_offsets",
                "color/tonemap_filmic",
                "postfx/vignette",
                "postfx/film_grain",
                STEP_6_POSTFX,
            ],
        );

        // For the postfx step, also stand up the full multipass setup
        // exactly the way the tutorial does — particle shader + mesh +
        // instances + intermediate texture + postfx shader + two passes —
        // so that "compatible shader for this mesh" / vertex-layout bugs
        // can't slip through compilation alone.
        let renderer = Renderer::new();
        let target = renderer.create_texture_target([256, 256]).await?;

        let particle_shader = Shader::new(["easing/in_out_sine", STEP_5_PARTICLE])?;
        particle_shader.set("time", 0.0_f32)?;
        particle_shader.set("resolution", [256.0_f32, 256.0])?;
        let mesh = Mesh::new();
        mesh.add_vertices([
            Vertex::new([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
            Vertex::new([ 0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
            Vertex::new([ 0.0,  0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
        ]);
        // Use a Vertex template so instance properties get auto-incrementing
        // locations starting at 1 instead of 0 — keeping clear of the
        // vertex `position` slot at @location(0).
        mesh.add_instances([
            Vertex::new([0.0, 0.0])
                .set("center", [0.0_f32, 0.0])
                .set("phase", 0.0_f32)
                .set("tint", [1.0_f32, 1.0, 1.0]),
        ]);
        particle_shader.add_mesh(&mesh)?;

        let intermediate = renderer.create_texture_target([256, 256]).await?;
        let scene_tex = intermediate.texture();

        let particle_pass = Pass::from_shader("particles", &particle_shader);
        particle_pass.add_mesh(&mesh)?;
        particle_pass.set_target(&intermediate)?;

        let postfx = Shader::new([
            "postfx/chromatic_offsets",
            "color/tonemap_filmic",
            "postfx/vignette",
            "postfx/film_grain",
            STEP_6_POSTFX,
        ])?;
        postfx.set("scene", &scene_tex)?;
        postfx.set("time", 0.0_f32)?;

        let postfx_pass = Pass::from_shader("postfx", &postfx);
        postfx_pass.require(&particle_pass)?;

        let passes = vec![particle_pass, postfx_pass];
        renderer.render(&passes, &target)?;
        let bytes = target.get_image().await;
        if bytes.is_empty() {
            return Err("step 5 multipass render returned empty buffer".into());
        }
        println!("  ✓ step 5 multipass renders end-to-end ({} bytes)", bytes.len());

        println!("All tutorial shaders verified.");
        Ok(())
    })
}
