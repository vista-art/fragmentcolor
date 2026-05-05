//! Catalog gallery for the `hash/` registry category.
//!
//! Renders one 256x256 PNG per hash function in
//! `out/gallery_hash/`. Each entry pulls a single registry slug into a
//! fullscreen fragment that calls the hash on a UV-derived coordinate and
//! visualizes the result as a noise field — grayscale for scalar outputs,
//! RGB for vec2/vec3 outputs.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_hash

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
            name: "hash11",
            description: "1D -> 1D float hash; sampled along uv.x * 96.",
            slugs: &["hash/hash11"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * vec2<f32>(96.0);
    // Combine x and y into a 1D coord so the field varies across both axes.
    let s = p.x + p.y * 137.0;
    let h = hash11(s);
    return vec4<f32>(h, h, h, 1.0);
}
"#,
        },
        Composition {
            name: "hash12",
            description: "2D -> 1D float hash, sampled at uv * 128.",
            slugs: &["hash/hash12"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 128.0;
    let h = hash12(p);
    return vec4<f32>(h, h, h, 1.0);
}
"#,
        },
        Composition {
            name: "hash13",
            description: "3D -> 1D float hash, sampled at vec3(uv * 100, 7.0).",
            slugs: &["hash/hash13"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec3<f32>(in.uv * 100.0, 7.0);
    let h = hash13(p);
    return vec4<f32>(h, h, h, 1.0);
}
"#,
        },
        Composition {
            name: "hash21",
            description: "1D -> 2D float hash; render as RG noise.",
            slugs: &["hash/hash21"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * vec2<f32>(112.0);
    let s = p.x + p.y * 137.0;
    let h = hash21(s);
    return vec4<f32>(h.x, h.y, 0.0, 1.0);
}
"#,
        },
        Composition {
            name: "hash22",
            description: "2D -> 2D float hash; render as RG noise.",
            slugs: &["hash/hash22"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 144.0;
    let h = hash22(p);
    return vec4<f32>(h.x, h.y, 0.0, 1.0);
}
"#,
        },
        Composition {
            name: "hash23",
            description: "2D -> 3D float hash; render as RGB noise.",
            slugs: &["hash/hash23"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 160.0;
    let h = hash23(p);
    return vec4<f32>(h, 1.0);
}
"#,
        },
        Composition {
            name: "hash33",
            description: "3D -> 3D float hash; render as RGB noise.",
            slugs: &["hash/hash33"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec3<f32>(in.uv * 176.0, 3.0);
    let h = hash33(p);
    return vec4<f32>(h, 1.0);
}
"#,
        },
        Composition {
            name: "iqint1",
            description: "Inigo Quilez integer hash u32 -> f32 in [0,1); pixel-grid keyed.",
            slugs: &["hash/iqint1"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cells = 192.0;
    let g = vec2<u32>(u32(in.uv.x * cells), u32(in.uv.y * cells));
    let key = g.x + g.y * 65537u;
    let h = iqint1(key);
    return vec4<f32>(h, h, h, 1.0);
}
"#,
        },
        Composition {
            name: "pcg",
            description: "PCG u32 -> u32 hash; pixel grid keyed, normalized to [0,1).",
            slugs: &["hash/pcg"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cells = 208.0;
    let g = vec2<u32>(u32(in.uv.x * cells), u32(in.uv.y * cells));
    let key = g.x + g.y * 1973u;
    let r = pcg(key);
    let h = f32(r) / 4294967295.0;
    return vec4<f32>(h, h, h, 1.0);
}
"#,
        },
        Composition {
            name: "pcg2d",
            description: "Mark Jarzynski PCG2D vec2<u32> -> vec2<u32>; render as RG.",
            slugs: &["hash/pcg2d"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cells = 224.0;
    let g = vec2<u32>(u32(in.uv.x * cells), u32(in.uv.y * cells));
    let r = pcg2d(g);
    let h = vec2<f32>(f32(r.x), f32(r.y)) / 4294967295.0;
    return vec4<f32>(h.x, h.y, 0.0, 1.0);
}
"#,
        },
        Composition {
            name: "pcg3d",
            description: "Mark Jarzynski PCG3D vec3<u32> -> vec3<u32>; render as RGB.",
            slugs: &["hash/pcg3d"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cells = 240.0;
    let g = vec3<u32>(u32(in.uv.x * cells), u32(in.uv.y * cells), 17u);
    let r = pcg3d(g);
    let h = vec3<f32>(f32(r.x), f32(r.y), f32(r.z)) / 4294967295.0;
    return vec4<f32>(h, 1.0);
}
"#,
        },
        Composition {
            name: "wang",
            description: "Wang u32 -> u32 hash; pixel grid keyed, normalized to [0,1).",
            slugs: &["hash/wang"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cells = 200.0;
    let g = vec2<u32>(u32(in.uv.x * cells), u32(in.uv.y * cells));
    let key = g.x + g.y * 2654435761u;
    let r = wang(key);
    let h = f32(r) / 4294967295.0;
    return vec4<f32>(h, h, h, 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_hash");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering hash gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
