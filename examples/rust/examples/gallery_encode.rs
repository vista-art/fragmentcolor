//! Catalog gallery for the `encode/` registry category.
//!
//! Renders one 256x256 PNG per encode/decode helper in
//! `docs/website/public/shaders/encode/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that exercises the helper on a synthetic
//! test signal and visualises the result.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_encode

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
            name: "f32_to_half_bits",
            description: "Convert a varying f32 value (UV-driven smooth field) to half-precision bits and visualise the low/high bytes as red/green channels.",
            slugs: &["encode/f32_to_half_bits"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Source signal: smooth 2D field in [-2, 2] so we exercise sign and exponent bits.
    let x = (in.uv.x * 4.0 - 2.0) + sin(in.uv.y * 6.2831) * 0.5;
    let bits = f32_to_half_bits(x);
    let lo = f32(bits & 0xFFu) / 255.0;
    let hi = f32((bits >> 8u) & 0xFFu) / 255.0;
    return vec4<f32>(lo, hi, 0.25, 1.0);
}
"#,
        },
        Composition {
            name: "morton2d",
            description: "Morton (Z-order) interleave on a 256x256 grid; the resulting u32 is normalised and shown as a hue/value pair so adjacent indices stay close on screen.",
            slugs: &["encode/morton2d"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let g = 64u;
    let x = u32(in.uv.x * f32(g));
    let y = u32(in.uv.y * f32(g));
    let m = morton2d(x, y);
    let denom = f32(g * g);
    let n = f32(m) / denom;
    // Split the index across red/green for a banded Z-curve look.
    let lo = fract(n * 16.0);
    let hi = n;
    return vec4<f32>(hi, lo, 0.5 - hi * 0.4, 1.0);
}
"#,
        },
        Composition {
            name: "pack_normal_octahedron",
            description: "Hemisphere normal n = (uv.x, uv.y, sqrt(1 - x^2 - y^2)) packed via octahedron mapping; the encoded vec2 is mapped to the RG channels.",
            slugs: &["encode/pack_normal_octahedron"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 2.0 - vec2<f32>(1.0);
    let z = sqrt(max(0.0, 1.0 - dot(p, p)));
    let n = normalize(vec3<f32>(p.x, p.y, z));
    let e = pack_normal_octahedron(n);
    // e is in [-1, 1]^2; remap to [0, 1] for display.
    let rg = e * 0.5 + vec2<f32>(0.5);
    return vec4<f32>(rg.x, rg.y, 0.2, 1.0);
}
"#,
        },
        Composition {
            name: "pack_rgb8",
            description: "Round-trip: build a vec3 from UV (R = uv.x, G = uv.y, B = a smooth fold), pack to u32, unpack, and display. Visualises the lossy 8-bit quantisation banding.",
            slugs: &["encode/pack_rgb8", "encode/unpack_rgb8"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = vec3<f32>(in.uv.x, in.uv.y, 0.5 + 0.5 * sin(in.uv.x * 6.2831 + in.uv.y * 3.14));
    let p = pack_rgb8(src);
    let c = unpack_rgb8(p);
    return vec4<f32>(c, 1.0);
}
"#,
        },
        Composition {
            name: "pack_unorm_4x8",
            description: "Round-trip a vec4 colour (UV gradient + radial alpha) through pack_unorm_4x8 / unpack_unorm_4x8; the result should match the source up to 8-bit quantisation.",
            slugs: &["encode/pack_unorm_4x8", "encode/unpack_unorm_4x8"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = in.uv.x;
    let g = in.uv.y;
    let b = 1.0 - in.uv.x;
    let a = 1.0 - length(in.uv - vec2<f32>(0.5)) * 1.4;
    let src = vec4<f32>(r, g, b, clamp(a, 0.0, 1.0));
    let p = pack_unorm_4x8(src);
    let c = unpack_unorm_4x8(p);
    // Composite the unpacked alpha against a dark grey to make it visible.
    let bg = vec3<f32>(0.08);
    return vec4<f32>(mix(bg, c.rgb, c.a), 1.0);
}
"#,
        },
        Composition {
            name: "unpack_normal_octahedron",
            description: "Synthesise an octahedron-encoded vec2 directly from UV, decode it to a unit normal, and display the normal mapped to RGB ((n + 1) / 2). Top-down view of a sphere.",
            slugs: &["encode/unpack_normal_octahedron"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let e = in.uv * 2.0 - vec2<f32>(1.0);
    let n = unpack_normal_octahedron(e);
    return vec4<f32>(n * 0.5 + vec3<f32>(0.5), 1.0);
}
"#,
        },
        Composition {
            name: "unpack_rgb8",
            description: "Pack a synthetic byte triple per pixel into a u32 directly, then decode with unpack_rgb8 to display the resulting colour wheel.",
            slugs: &["encode/unpack_rgb8"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = u32(in.uv.x * 255.0);
    let g = u32(in.uv.y * 255.0);
    let b = u32((1.0 - in.uv.x * 0.5 - in.uv.y * 0.5) * 255.0);
    let p = r | (g << 8u) | (b << 16u);
    let c = unpack_rgb8(p);
    return vec4<f32>(c, 1.0);
}
"#,
        },
        Composition {
            name: "unpack_unorm_4x8",
            description: "Build a u32 with a different byte assigned to each channel directly from UV, decode it with unpack_unorm_4x8, and composite against a dark background using the alpha byte.",
            slugs: &["encode/unpack_unorm_4x8"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = u32(in.uv.x * 255.0);
    let g = u32((1.0 - in.uv.y) * 255.0);
    let b = u32((in.uv.x * in.uv.y) * 255.0);
    let radial = clamp(1.0 - length(in.uv - vec2<f32>(0.5)) * 1.4, 0.0, 1.0);
    let a = u32(radial * 255.0);
    let p = r | (g << 8u) | (b << 16u) | (a << 24u);
    let c = unpack_unorm_4x8(p);
    let bg = vec3<f32>(0.08);
    return vec4<f32>(mix(bg, c.rgb, c.a), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_encode");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering encode gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
