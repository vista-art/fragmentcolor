//! Visual smoke test for the embedded shader registry.
//!
//! Each entry below composes one or more registry slugs with a fullscreen
//! fragment that calls them, renders to a 256×256 texture target, reads the
//! pixels back, and writes a PNG to `out/registry-gallery/<name>.png`.
//!
//! Run with:
//!   cargo run --release -p fce --example registry_gallery
//!
//! Skipped silently on systems without a wgpu-compatible adapter; this is a
//! visual smoke test, not a CI gate.
//!
//! Add new entries by appending to `compositions()` — pure-function slugs
//! follow the same pattern: pull the helper into `Shader::new`, then call
//! it from the inline fullscreen fragment.

use fragmentcolor::{Renderer, Shader, Target};

const SIZE: u32 = 256;

/// Standard fullscreen vertex stage that emits UVs in [0, 1].
const FULLSCREEN_VS: &str = r#"
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var pts = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    let p = pts[i];
    let uv = vec2<f32>((p.x + 1.0) * 0.5, 1.0 - (p.y + 1.0) * 0.5);
    return VertexOutput(vec4<f32>(p, 0.0, 1.0), uv);
}
"#;

struct Composition {
    name: &'static str,
    description: &'static str,
    slugs: &'static [&'static str],
    fragment: &'static str,
}

fn compositions() -> Vec<Composition> {
    vec![
        Composition {
            name: "01-sdf2d-circle",
            description: "Solid disc. sdf2d/circle returns signed distance; we threshold at 0.",
            slugs: &["sdf2d/circle"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 2.0 - vec2<f32>(1.0);
    let d = circle(p, 0.7);
    let mask = 1.0 - smoothstep(-0.005, 0.005, d);
    return vec4<f32>(vec3<f32>(0.95, 0.55, 0.85) * mask, 1.0);
}
"#,
        },
        Composition {
            name: "02-sdf2d-heart",
            description: "Heart shape via sdf2d/heart. Positive distance = outside.",
            slugs: &["sdf2d/heart"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let d = heart(vec2<f32>(p.x, -p.y), 0.7);
    let mask = 1.0 - smoothstep(-0.01, 0.01, d);
    return vec4<f32>(vec3<f32>(1.0, 0.25, 0.35) * mask, 1.0);
}
"#,
        },
        Composition {
            name: "03-noise-simplex2",
            description: "Simplex noise field, single octave.",
            slugs: &["noise/simplex2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = simplex2(in.uv * 6.0) * 0.5 + 0.5;
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "04-pattern-dots",
            description: "Dot grid. cells per UV unit, radius in cell units, fw is feather width.",
            slugs: &["pattern/dots"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = dots(in.uv, 12.0, 0.25, 0.02);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(1.0, 0.85, 0.4);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "05-postfx-vignette",
            description: "Pure radial vignette mask multiplied against a flat colour.",
            slugs: &["postfx/vignette"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = vignette(in.uv, 0.4, 0.45);
    let base = vec3<f32>(0.95, 0.85, 0.7);
    return vec4<f32>(base * v, 1.0);
}
"#,
        },
        Composition {
            name: "06-circle-noise",
            description: "Composition: sdf2d/circle masked colour, modulated by noise/simplex2.",
            slugs: &["sdf2d/circle", "noise/simplex2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 2.0 - vec2<f32>(1.0);
    let d = circle(p, 0.7);
    let mask = 1.0 - smoothstep(-0.01, 0.01, d);
    let n = simplex2(in.uv * 12.0) * 0.5 + 0.5;
    let col = mix(vec3<f32>(0.2, 0.5, 0.9), vec3<f32>(0.95, 0.95, 1.0), n);
    return vec4<f32>(col * mask, 1.0);
}
"#,
        },
        Composition {
            name: "07-dots-vignette",
            description: "Composition: pattern/dots with postfx/vignette overlay.",
            slugs: &["pattern/dots", "postfx/vignette"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = dots(in.uv, 16.0, 0.25, 0.02);
    let v = vignette(in.uv, 0.45, 0.4);
    let bg = vec3<f32>(0.05);
    let fg = vec3<f32>(0.95, 0.4, 0.5);
    return vec4<f32>(mix(bg, fg, m) * v, 1.0);
}
"#,
        },
        Composition {
            name: "08-noise-stack",
            description: "Composition: noise/simplex2 as height + sdf2d/circle as gate.",
            slugs: &["noise/simplex2", "sdf2d/circle"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 2.0 - vec2<f32>(1.0);
    let d = circle(p, 0.85);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d);
    let n1 = simplex2(in.uv * 4.0) * 0.5 + 0.5;
    let n2 = simplex2(in.uv * 16.0) * 0.5 + 0.5;
    let n  = n1 * 0.7 + n2 * 0.3;
    let col = mix(vec3<f32>(0.2, 0.05, 0.4), vec3<f32>(1.0, 0.7, 0.2), n);
    return vec4<f32>(col * mask, 1.0);
}
"#,
        },
    ]
}

async fn render_one(
    renderer: &Renderer,
    out_dir: &std::path::Path,
    comp: &Composition,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = renderer.create_texture_target([SIZE, SIZE]).await?;

    let mut parts: Vec<&str> = comp.slugs.to_vec();
    let body = format!("{}{}", FULLSCREEN_VS, comp.fragment);
    parts.push(&body);
    let shader = Shader::new(parts.as_slice())?;

    renderer.render(&shader, &target)?;
    let bytes = target.get_image().await;
    if bytes.is_empty() {
        return Err(format!("readback returned empty buffer for {}", comp.name).into());
    }
    let img = image::RgbaImage::from_vec(SIZE, SIZE, bytes)
        .ok_or("failed to wrap pixel bytes as RgbaImage")?;
    let path = out_dir.join(format!("{}.png", comp.name));
    img.save(&path)?;
    println!("  ✓ {} → {}", comp.name, path.display());
    println!("    {}", comp.description);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let out_dir = std::path::Path::new("out/registry-gallery");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering registry gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  ✗ {} — {}", comp.name, e);
            }
        }
        Ok(())
    })
}
