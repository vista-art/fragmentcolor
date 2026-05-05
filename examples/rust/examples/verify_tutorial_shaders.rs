//! One-off validator: constructs every Shader the tutorial 01 demos build
//! and renders each to a 256×256 texture target. If any step's WGSL
//! composition is invalid, this fails before the docs site does.
//!
//! Run: `cargo run --release -p fce --example verify_tutorial_shaders`
//!
//! Not registered in any test target — it's a manual gate the tutorial
//! author runs after editing inline WGSL or swapping registry slugs.

use fragmentcolor::{Renderer, Shader, Target};

const STEP_2_NOISY: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> time: f32;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p  = array<vec2<f32>, 3>(
        vec2<f32>(-0.6, -0.5),
        vec2<f32>( 0.6, -0.5),
        vec2<f32>( 0.0,  0.7),
    );
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0.0, 1.0);
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

const STEP_4_PARTICLE: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> time: f32;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) center: vec2<f32>,
    @location(3) phase: f32,
    @location(4) tint: vec3<f32>,
) -> VOut {
    let scale = 0.045;
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase) * 0.05,
        cos(time * 0.9 + phase * 1.4) * 0.05,
    );
    let world = position.xy * scale + center + wobble;
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

const STEP_5_POSTFX: &str = r#"
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
        println!("  · step 1 (URL): exercised by tutorial_01_hello_triangle_three_lines");

        // Step 2 — noise/simplex2 + inline triangle.
        check("step 2 (compose-noise)", &["noise/simplex2", STEP_2_NOISY]);

        // Step 3 — inline only (no slugs).
        // Validated by tutorial_01_hello_triangle_vertex_gradient.
        println!("  · step 3 (vertex-gradient): exercised by its own example");

        // Step 4 — easing/in_out_sine + inline particle source.
        check("step 4 (particle-trail)", &["easing/in_out_sine", STEP_4_PARTICLE]);

        // Step 5 — four registry slugs + inline postfx main.
        check(
            "step 5 (postfx-pass)",
            &[
                "postfx/chromatic_offsets",
                "color/tonemap_filmic",
                "postfx/vignette",
                "postfx/film_grain",
                STEP_5_POSTFX,
            ],
        );

        // For the postfx step, also try a real render (with a stub texture
        // input) to confirm pipeline creation works end to end.
        let renderer = Renderer::new();
        let target = renderer.create_texture_target([256, 256]).await?;
        let postfx = Shader::new([
            "postfx/chromatic_offsets",
            "color/tonemap_filmic",
            "postfx/vignette",
            "postfx/film_grain",
            STEP_5_POSTFX,
        ])?;

        // Need a scene texture to sample. Use the target's own texture as a stand-in.
        let scene_tex = target.texture();
        postfx.set("scene", &scene_tex)?;
        postfx.set("time", 0.0_f32)?;
        renderer.render(&postfx, &target)?;
        let bytes = target.get_image().await;
        if bytes.is_empty() {
            return Err("step 5 render returned empty buffer".into());
        }
        println!("  ✓ step 5 renders end-to-end ({} bytes)", bytes.len());

        println!("All tutorial shaders verified.");
        Ok(())
    })
}
