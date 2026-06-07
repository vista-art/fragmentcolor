//! Catalog gallery for the `dither/` registry category.
//!
//! Renders one 256x256 PNG per dither shader in
//! `docs/website/public/shaders/dither/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that applies the dither to a smooth
//! horizontal gradient (`color = vec3(uv.x)`), with a thin reference strip of
//! the source gradient on the top 1/4 of the frame.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_dither

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
            name: "bayer2x2",
            description: "2x2 ordered dither — chunky checkerboard banding on a smooth ramp.",
            slugs: &["dither/bayer2x2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = in.uv.x;
    if (in.uv.y < 0.22) {
        // Reference strip: smooth source gradient.
        return vec4<f32>(vec3<f32>(src), 1.0);
    }
    if (in.uv.y < 0.235) {
        // Separator line.
        return vec4<f32>(vec3<f32>(0.0), 1.0);
    }
    let pix = vec2<i32>(in.uv * f32(256));
    let t = bayer2x2(pix);
    let q = select(0.0, 1.0, src > t);
    return vec4<f32>(vec3<f32>(q), 1.0);
}
"#,
        },
        Composition {
            name: "bayer4x4",
            description: "4x4 ordered dither — the classic Bayer matrix, mid-coarse banding.",
            slugs: &["dither/bayer4x4"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = in.uv.x;
    if (in.uv.y < 0.22) {
        return vec4<f32>(vec3<f32>(src), 1.0);
    }
    if (in.uv.y < 0.235) {
        return vec4<f32>(vec3<f32>(0.0), 1.0);
    }
    let pix = vec2<i32>(in.uv * f32(256));
    let t = bayer4x4(pix);
    let q = select(0.0, 1.0, src > t);
    return vec4<f32>(vec3<f32>(q), 1.0);
}
"#,
        },
        Composition {
            name: "bayer8x8",
            description: "8x8 ordered dither — finest Bayer threshold, smoothest banding.",
            slugs: &["dither/bayer8x8"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = in.uv.x;
    if (in.uv.y < 0.22) {
        return vec4<f32>(vec3<f32>(src), 1.0);
    }
    if (in.uv.y < 0.235) {
        return vec4<f32>(vec3<f32>(0.0), 1.0);
    }
    let pix = vec2<i32>(in.uv * f32(256));
    let t = bayer8x8(pix);
    let q = select(0.0, 1.0, src > t);
    return vec4<f32>(vec3<f32>(q), 1.0);
}
"#,
        },
        Composition {
            name: "interleaved_gradient_noise",
            description: "Jorge Jimenez's IGN — blue-noise-like, broken-up speckle pattern.",
            slugs: &["dither/interleaved_gradient_noise"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = in.uv.x;
    if (in.uv.y < 0.22) {
        return vec4<f32>(vec3<f32>(src), 1.0);
    }
    if (in.uv.y < 0.235) {
        return vec4<f32>(vec3<f32>(0.0), 1.0);
    }
    let pix = in.uv * f32(256);
    let t = interleaved_gradient_noise(pix);
    let q = select(0.0, 1.0, src > t);
    return vec4<f32>(vec3<f32>(q), 1.0);
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
    println!("     {}", comp.description);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let out_dir = std::path::Path::new("out/gallery_dither");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering dither gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
