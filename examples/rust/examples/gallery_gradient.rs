//! Catalog gallery for the `gradient/` registry category.
//!
//! Renders one 256x256 PNG per gradient/colormap shader in
//! `docs/website/public/shaders/gradient/`. Each entry pulls a single
//! registry slug into a fullscreen fragment that drives the colormap with a
//! UV-derived parameter `t` (linear, radial, or conic) and writes the result
//! straight to the framebuffer.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_gradient

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
            name: "cividis",
            description: "Cividis colormap driven horizontally across the frame.",
            slugs: &["gradient/cividis"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(cividis(t), 1.0);
}
"#,
        },
        Composition {
            name: "cubehelix",
            description: "Cubehelix ramp with default Dave Green parameters (start=0.5, rot=-1.5, hue=1.0, gamma=1.0).",
            slugs: &["gradient/cubehelix"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(cubehelix(t, 0.5, -1.5, 1.0, 1.0), 1.0);
}
"#,
        },
        Composition {
            name: "grayscale",
            description: "Linear gray ramp from black to white.",
            slugs: &["gradient/grayscale"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(grayscale(t), 1.0);
}
"#,
        },
        Composition {
            name: "inferno",
            description: "Inferno colormap driven horizontally across the frame.",
            slugs: &["gradient/inferno"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(inferno(t), 1.0);
}
"#,
        },
        Composition {
            name: "jet",
            description: "Classic MATLAB jet, swept radially from the centre to highlight all bands.",
            slugs: &["gradient/jet"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = length(in.uv - vec2<f32>(0.5)) * 1.4142136;
    let t = clamp(r, 0.0, 1.0);
    return vec4<f32>(jet(t), 1.0);
}
"#,
        },
        Composition {
            name: "magma",
            description: "Magma colormap driven horizontally across the frame.",
            slugs: &["gradient/magma"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(magma(t), 1.0);
}
"#,
        },
        Composition {
            name: "palette_iq",
            description: "Inigo Quilez cosine palette swept around the frame as a conic gradient.",
            slugs: &["gradient/palette_iq"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv - vec2<f32>(0.5);
    let t = atan2(p.y, p.x) / 6.28318530718 + 0.5;
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.0, 0.33, 0.67);
    return vec4<f32>(palette_iq(t, a, b, c, d), 1.0);
}
"#,
        },
        Composition {
            name: "plasma",
            description: "Plasma colormap driven horizontally across the frame.",
            slugs: &["gradient/plasma"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(plasma(t), 1.0);
}
"#,
        },
        Composition {
            name: "spectral",
            description: "ColorBrewer spectral rainbow, swept radially to show the diverging palette.",
            slugs: &["gradient/spectral"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = length(in.uv - vec2<f32>(0.5)) * 1.4142136;
    let t = clamp(r, 0.0, 1.0);
    return vec4<f32>(spectral(t), 1.0);
}
"#,
        },
        Composition {
            name: "turbo",
            description: "Google Turbo colormap driven horizontally across the frame.",
            slugs: &["gradient/turbo"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(turbo(t), 1.0);
}
"#,
        },
        Composition {
            name: "viridis",
            description: "Viridis colormap driven horizontally across the frame.",
            slugs: &["gradient/viridis"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x;
    return vec4<f32>(viridis(t), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_gradient");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering gradient gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  fail {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
