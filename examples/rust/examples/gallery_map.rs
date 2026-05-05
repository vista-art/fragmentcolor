//! Catalog gallery for the `map/` registry category.
//!
//! Renders one 256x256 PNG per coordinate-mapping shader in
//! `docs/website/public/shaders/map/`. Each entry pulls a single registry
//! slug into a fullscreen fragment, synthesizes a colored checkerboard
//! input pattern, applies the map, and renders the warped result.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_map

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

fn input_pattern(uv: vec2<f32>) -> vec3<f32> {
    let cell = step(vec2<f32>(0.5), fract(uv * 8.0));
    let c = cell.x * cell.y + (1.0 - cell.x) * (1.0 - cell.y);
    let g = vec3<f32>(uv.x, uv.y, 0.5);
    return mix(g, vec3<f32>(0.95, 0.95, 0.95), c * 0.6);
}

fn safe_uv(uv: vec2<f32>) -> vec2<f32> {
    return clamp(uv, vec2<f32>(0.0), vec2<f32>(0.999));
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
            name: "barrel",
            description: "Barrel lens distortion (k = 0.6) — bulges the center outward.",
            slugs: &["map/barrel"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = barrel(in.uv, 0.6);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "cartesian",
            description: "Inverse polar mapping: (angle, radius) → uv around (0.5, 0.5).",
            slugs: &["map/cartesian"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = cartesian(in.uv, vec2<f32>(0.5, 0.5));
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "fisheye",
            description: "Spherical fisheye remap with strength = 1.2.",
            slugs: &["map/fisheye"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = fisheye(in.uv, 1.2);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "kaleidoscope",
            description: "6-fold rotational symmetry fold around UV center.",
            slugs: &["map/kaleidoscope"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = kaleidoscope(in.uv, 6u);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "mirror_x",
            description: "Fold UV across x = 0.5; left and right halves mirror.",
            slugs: &["map/mirror_x"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = mirror_x(in.uv);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "mirror_y",
            description: "Fold UV across y = 0.5; top and bottom halves mirror.",
            slugs: &["map/mirror_y"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = mirror_y(in.uv);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "pincushion",
            description: "Pincushion distortion (k = 1.5) — pinches edges inward.",
            slugs: &["map/pincushion"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = pincushion(in.uv, 1.5);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "polar",
            description: "Cartesian → polar around UV center: x is angle/TAU, y is radius.",
            slugs: &["map/polar"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = polar(in.uv, vec2<f32>(0.5, 0.5));
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "ripple",
            description: "Radial ripple from center (freq = 40, amp = 0.04).",
            slugs: &["map/ripple"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = ripple(in.uv, vec2<f32>(0.5, 0.5), 40.0, 0.0, 0.04);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "rotate2",
            description: "Rotate UV around center by 30 degrees (~0.5236 rad).",
            slugs: &["map/rotate2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv - vec2<f32>(0.5);
    let r = rotate2(p, 0.5236);
    let m = r + vec2<f32>(0.5);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "scale2",
            description: "Non-uniform scale around UV pivot (sx = 1.5, sy = 0.75).",
            slugs: &["map/scale2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = scale2(in.uv, vec2<f32>(0.5, 0.5), vec2<f32>(1.5, 0.75));
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "tile",
            description: "Tile UVs at 3x3 cells per unit; each cell repeats the pattern.",
            slugs: &["map/tile"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = tile(in.uv, vec2<f32>(3.0, 3.0));
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "translate2",
            description: "Translate UV by (-0.2, -0.1); pattern shifts up and right.",
            slugs: &["map/translate2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = translate2(in.uv, vec2<f32>(-0.2, -0.1));
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "twirl",
            description: "Twirl around center (strength = 3.5); falls off with radius.",
            slugs: &["map/twirl"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = twirl(in.uv, vec2<f32>(0.5, 0.5), 3.5);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
}
"#,
        },
        Composition {
            name: "wave",
            description: "Horizontal sine displacement (freq = 18, amp = 0.04).",
            slugs: &["map/wave"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = wave(in.uv, 18.0, 0.0, 0.04);
    return vec4<f32>(input_pattern(safe_uv(m)), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_map");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering map gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} -- {}", comp.name, e);
            }
        }
        Ok(())
    })
}
