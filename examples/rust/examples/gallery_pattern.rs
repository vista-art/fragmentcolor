//! Catalog gallery for the `pattern/` registry category.
//!
//! Renders one 256x256 PNG per repeating-pattern shader in
//! `docs/website/public/shaders/pattern/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that calls the pattern function and tints
//! the resulting mask between a muted background and a single accent.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_pattern

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
            name: "brick",
            description: "Running-bond brick grid, 6 wide x 10 tall, thin mortar lines.",
            slugs: &["pattern/brick"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = brick(in.uv, vec2<f32>(6.0, 10.0), 0.06, 0.01);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.85, 0.40, 0.32);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "chevron",
            description: "V-shaped stripes scaled by 4 with period 0.6 and slant 0.7.",
            slugs: &["pattern/chevron"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 4.0;
    let m = chevron(p, 0.6, 0.7, 0.04);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.78, 0.35);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "dots",
            description: "Circular dots, 12 cells per UV unit, radius 0.25 of a cell.",
            slugs: &["pattern/dots"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = dots(in.uv, 12.0, 0.25, 0.02);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.65, 0.40);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "hexgrid",
            description: "Hexagonal grid line mask. Returns (edge_dist, hex_id.x, hex_id.y).",
            slugs: &["pattern/hexgrid"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let h = hexgrid(in.uv, 8.0);
    let line = 1.0 - smoothstep(0.04, 0.06, h.x);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.55, 0.85, 0.95);
    return vec4<f32>(mix(bg, fg, line), 1.0);
}
"#,
        },
        Composition {
            name: "stripes",
            description: "Diagonal stripes, period 0.12, 50% duty cycle, anti-aliased.",
            slugs: &["pattern/stripes"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = stripes(in.uv, vec2<f32>(1.0, 0.6), 0.12, 0.5, 0.04);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.75, 0.55, 0.95);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "trigrid",
            description: "Triangular grid line mask, scale 8, line thickness 0.04.",
            slugs: &["pattern/trigrid"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = trigrid(in.uv, 8.0, 0.04, 0.01);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.50, 0.95, 0.65);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "truchet",
            description: "Truchet quarter-arc tiles with random rotation, scale 8.",
            slugs: &["pattern/truchet"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = truchet(in.uv, 8.0);
    let line = 1.0 - smoothstep(0.05, 0.08, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.55, 0.65);
    return vec4<f32>(mix(bg, fg, line), 1.0);
}
"#,
        },
        Composition {
            name: "weave",
            description: "Over/under weave pattern at 8 cells per UV unit.",
            slugs: &["pattern/weave"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = weave(in.uv, 8.0, 0.05);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.85, 0.75, 0.55);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "zebra",
            description: "Wavy zebra stripes: freq 24, warp 0.4, smooth-stepped.",
            slugs: &["pattern/zebra"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = zebra(in.uv, 24.0, 0.4, 0.0, 0.05);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.95, 0.95);
    return vec4<f32>(mix(bg, fg, m), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_pattern");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering pattern gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} -- {}", comp.name, e);
            }
        }
        Ok(())
    })
}
