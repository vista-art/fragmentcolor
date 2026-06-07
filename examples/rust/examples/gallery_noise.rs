//! Catalog gallery for the `noise/` registry category.
//!
//! Renders one 256x256 PNG per noise shader in
//! `docs/website/public/shaders/noise/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that samples the field and visualizes it
//! as a grayscale (or RGB, for vector outputs) image.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_noise

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
            name: "curl2",
            description: "2D curl noise — divergence-free vector field. abs(curl) shown as RGB.",
            slugs: &["noise/curl2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = curl2(in.uv * 6.0);
    let c = saturate(abs(v) * 6.0);
    return vec4<f32>(c.x, c.y, 0.6 * (c.x + c.y) * 0.5, 1.0);
}
"#,
        },
        Composition {
            name: "fbm2",
            description: "2D fractal Brownian motion (5 octaves of value noise).",
            slugs: &["noise/fbm2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = saturate(fbm2(in.uv * 4.0, 5u));
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "fbm3",
            description: "3D fractal Brownian motion sampled on the z=0 slice (5 octaves).",
            slugs: &["noise/fbm3"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec3<f32>(in.uv * 4.0, 0.0);
    let n = saturate(fbm3(p, 5u));
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "gradient2",
            description: "2D gradient (Perlin-style) noise, single octave, remapped to [0, 1].",
            slugs: &["noise/gradient2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = gradient2(in.uv * 6.0) * 0.5 + 0.5;
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "ridged2",
            description: "2D ridged multifractal (5 octaves) — crisp mountain-ridge feel.",
            slugs: &["noise/ridged2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = saturate(ridged2(in.uv * 4.0, 5u));
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "simplex2",
            description: "2D simplex noise, single octave, remapped to [0, 1].",
            slugs: &["noise/simplex2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = simplex2(in.uv * 6.0) * 0.5 + 0.5;
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "turbulence2",
            description: "2D turbulence (5 octaves of |value noise|) — flame/smoke feel.",
            slugs: &["noise/turbulence2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = saturate(turbulence2(in.uv * 4.0, 5u));
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "value2",
            description: "2D value noise, single octave.",
            slugs: &["noise/value2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = value2(in.uv * 8.0);
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "value3",
            description: "3D value noise sampled on the z=0 slice.",
            slugs: &["noise/value3"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec3<f32>(in.uv * 8.0, 0.0);
    let n = value3(p);
    return vec4<f32>(n, n, n, 1.0);
}
"#,
        },
        Composition {
            name: "worley2",
            description: "2D Worley (cellular) noise — F1 distance shown as cells.",
            slugs: &["noise/worley2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let f = worley2(in.uv * 8.0);
    let n = saturate(f.x);
    return vec4<f32>(n, n, n, 1.0);
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
    println!("  ok {} -> {}", comp.name, path.display());
    println!("    {}", comp.description);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let out_dir = std::path::Path::new("out/gallery_noise");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering noise gallery into {}/", out_dir.display());

        let mut failures: Vec<String> = Vec::new();
        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} — {}", comp.name, e);
                failures.push(comp.name.to_string());
            }
        }
        if !failures.is_empty() {
            return Err(format!("failed: {}", failures.join(", ")).into());
        }
        Ok(())
    })
}
