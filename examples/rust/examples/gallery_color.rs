//! Catalog gallery for the `color/` registry category.
//!
//! Renders one 256x256 PNG per color helper in
//! `docs/website/public/shaders/color/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that exercises the function across the
//! frame — typically a top-half input gradient against a bottom-half
//! transformed output, with a 2-pixel separator.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_color

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
            name: "contrast",
            description: "Hue ramp on top half, contrast(c, 1.8) on bottom — pushes shadows/highlights apart.",
            slugs: &["color/contrast", "color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = hsv_to_rgb(vec3<f32>(in.uv.x, 0.85, 0.95));
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    return vec4<f32>(clamp(contrast(c, 1.8), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "hsl_to_rgb",
            description: "x = hue, y = lightness (top to bottom), saturation fixed at 0.85.",
            slugs: &["color/hsl_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let h = in.uv.x;
    let l = 1.0 - in.uv.y;
    let rgb = hsl_to_rgb(vec3<f32>(h, 0.85, l));
    return vec4<f32>(rgb, 1.0);
}
"#,
        },
        Composition {
            name: "hsv_to_rgb",
            description: "x = hue, y = value (top to bottom), saturation fixed at 0.9.",
            slugs: &["color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let h = in.uv.x;
    let v = 1.0 - in.uv.y;
    let rgb = hsv_to_rgb(vec3<f32>(h, 0.9, v));
    return vec4<f32>(rgb, 1.0);
}
"#,
        },
        Composition {
            name: "hue_shift",
            description: "Top: input hue ramp. Bottom: hue_shift by +2.0 rad rotates the spectrum.",
            slugs: &["color/hue_shift", "color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = hsv_to_rgb(vec3<f32>(in.uv.x, 0.85, 0.95));
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    return vec4<f32>(clamp(hue_shift(c, 2.0), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "kelvin_to_rgb",
            description: "Color temperature ramp from 1500 K (warm red) to 12000 K (cool blue).",
            slugs: &["color/kelvin_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let k = mix(1500.0, 12000.0, in.uv.x);
    return vec4<f32>(kelvin_to_rgb(k), 1.0);
}
"#,
        },
        Composition {
            name: "lift_gamma_gain",
            description: "Top: neutral gradient. Bottom: lift +0.05 (warm shadows), gamma 1.2, gain 1.05 (cool highlights).",
            slugs: &["color/lift_gamma_gain"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = vec3<f32>(in.uv.x);
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    let lift  = vec3<f32>(0.08, 0.04, -0.02);
    let gamma = vec3<f32>(1.20, 1.10, 0.95);
    let gain  = vec3<f32>(0.95, 1.00, 1.10);
    return vec4<f32>(clamp(lift_gamma_gain(c, lift, gamma, gain), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "linear_srgb_to_oklab",
            description: "Top: hue ramp in linear sRGB. Bottom: a/b channels visualized after OkLab conversion.",
            slugs: &["color/linear_srgb_to_oklab", "color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = hsv_to_rgb(vec3<f32>(in.uv.x, 0.85, 0.9));
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    let lab = linear_srgb_to_oklab(c);
    // L → red, a → green offset, b → blue offset (visualization only)
    return vec4<f32>(clamp(vec3<f32>(lab.x, 0.5 + lab.y * 2.5, 0.5 + lab.z * 2.5), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "linear_to_srgb",
            description: "Top: linear gradient. Bottom: same values after sRGB encode — midtones brighten.",
            slugs: &["color/linear_to_srgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = vec3<f32>(in.uv.x);
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    return vec4<f32>(linear_to_srgb(c), 1.0);
}
"#,
        },
        Composition {
            name: "luminance",
            description: "Top: hue ramp. Bottom: per-pixel Rec.709 luminance (grayscale).",
            slugs: &["color/luminance", "color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = hsv_to_rgb(vec3<f32>(in.uv.x, 0.9, 0.95));
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    let y = luminance(c);
    return vec4<f32>(vec3<f32>(y), 1.0);
}
"#,
        },
        Composition {
            name: "oklab_to_linear_srgb",
            description: "L = 0.7 plane, x sweeps a in [-0.3, 0.3], y sweeps b in [-0.3, 0.3].",
            slugs: &["color/oklab_to_linear_srgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let a = (in.uv.x - 0.5) * 0.6;
    let b = (0.5 - in.uv.y) * 0.6;
    let rgb = oklab_to_linear_srgb(vec3<f32>(0.7, a, b));
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "oklch_to_linear_srgb",
            description: "L = 0.7, x sweeps hue in [0, 2pi], y sweeps chroma in [0, 0.25].",
            slugs: &["color/oklch_to_linear_srgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let h = in.uv.x * 6.2831853;
    let c = (1.0 - in.uv.y) * 0.25;
    let rgb = oklch_to_linear_srgb(vec3<f32>(0.7, c, h));
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "rgb_to_hsl",
            description: "Top: hue ramp (rgb input). Bottom: HSL channels mapped back to RGB for display.",
            slugs: &["color/rgb_to_hsl", "color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = hsv_to_rgb(vec3<f32>(in.uv.x, 0.85, 0.95));
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    let hsl = rgb_to_hsl(c);
    // Visualize: H → R, S → G, L → B
    return vec4<f32>(hsl, 1.0);
}
"#,
        },
        Composition {
            name: "rgb_to_hsv",
            description: "Top: hue ramp (rgb input). Bottom: HSV channels mapped to RGB for visualization.",
            slugs: &["color/rgb_to_hsv", "color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = hsv_to_rgb(vec3<f32>(in.uv.x, 0.85, 0.95));
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    let hsv = rgb_to_hsv(c);
    return vec4<f32>(hsv, 1.0);
}
"#,
        },
        Composition {
            name: "saturation",
            description: "Top: hue ramp. Bottom: x maps to saturation s in [0, 1.6] applied to a fixed teal.",
            slugs: &["color/saturation", "color/hsv_to_rgb"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = hsv_to_rgb(vec3<f32>(in.uv.x, 0.85, 0.95));
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    let s = in.uv.x * 1.6;
    return vec4<f32>(clamp(saturation(c, s), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "srgb_to_linear",
            description: "Top: sRGB-encoded gradient. Bottom: same after EOTF — midtones darken.",
            slugs: &["color/srgb_to_linear"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let c = vec3<f32>(in.uv.x);
    if (in.uv.y < 0.5) { return vec4<f32>(c, 1.0); }
    return vec4<f32>(srgb_to_linear(c), 1.0);
}
"#,
        },
        Composition {
            name: "tonemap_aces",
            description: "Top: 0..6x HDR ramp. Bottom: ACES-fitted tone curve mapped to [0, 1].",
            slugs: &["color/tonemap_aces"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let hdr = vec3<f32>(in.uv.x * 6.0, in.uv.x * 4.5, in.uv.x * 3.0);
    if (in.uv.y < 0.5) { return vec4<f32>(min(hdr, vec3<f32>(1.0)), 1.0); }
    return vec4<f32>(tonemap_aces(hdr), 1.0);
}
"#,
        },
        Composition {
            name: "tonemap_filmic",
            description: "Top: 0..6x HDR ramp clipped. Bottom: Hejl filmic curve (already gamma-encoded).",
            slugs: &["color/tonemap_filmic"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let hdr = vec3<f32>(in.uv.x * 6.0, in.uv.x * 4.5, in.uv.x * 3.0);
    if (in.uv.y < 0.5) { return vec4<f32>(min(hdr, vec3<f32>(1.0)), 1.0); }
    return vec4<f32>(tonemap_filmic(hdr), 1.0);
}
"#,
        },
        Composition {
            name: "tonemap_reinhard",
            description: "Top: 0..6x HDR ramp clipped. Bottom: Reinhard c/(1+c) — soft roll-off, washed highlights.",
            slugs: &["color/tonemap_reinhard"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let hdr = vec3<f32>(in.uv.x * 6.0, in.uv.x * 4.5, in.uv.x * 3.0);
    if (in.uv.y < 0.5) { return vec4<f32>(min(hdr, vec3<f32>(1.0)), 1.0); }
    return vec4<f32>(tonemap_reinhard(hdr), 1.0);
}
"#,
        },
        Composition {
            name: "tonemap_uncharted2",
            description: "Top: 0..6x HDR ramp clipped. Bottom: Hable Uncharted 2 with W=11.2 white point.",
            slugs: &["color/tonemap_uncharted2"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let band = abs(in.uv.y - 0.5);
    if (band < 0.008) { return vec4<f32>(0.05, 0.05, 0.06, 1.0); }
    let hdr = vec3<f32>(in.uv.x * 6.0, in.uv.x * 4.5, in.uv.x * 3.0);
    if (in.uv.y < 0.5) { return vec4<f32>(min(hdr, vec3<f32>(1.0)), 1.0); }
    return vec4<f32>(clamp(tonemap_uncharted2(hdr), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_color");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering color gallery into {}/", out_dir.display());

        let mut failed = 0usize;
        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  fail {} - {}", comp.name, e);
                failed += 1;
            }
        }
        if failed > 0 {
            return Err(format!("{} composition(s) failed", failed).into());
        }
        Ok(())
    })
}
