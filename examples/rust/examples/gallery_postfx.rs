//! Catalog gallery for the `postfx/` registry category.
//!
//! Renders one 256x256 PNG per post-processing shader in
//! `docs/website/public/shaders/postfx/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that builds a synthetic input image, then
//! applies the postfx so the effect is visible in isolation.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_postfx

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

/// Synthetic input function used by every postfx preview. It produces a
/// gentle gradient with concentric soft rings so multiplicative, additive,
/// and convolution effects all have something visible to chew on.
const SYNTHETIC_INPUT: &str = r#"
fn synthetic_input(uv: vec2<f32>) -> vec3<f32> {
    let d = length(uv - vec2<f32>(0.5));
    let rings = 0.5 + 0.5 * sin(d * 30.0);
    let g = vec3<f32>(0.45 + 0.4 * uv.x, 0.55 + 0.3 * uv.y, 0.6);
    return mix(g, vec3<f32>(0.95, 0.92, 0.85), rings * 0.35);
}
fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
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
            name: "chromatic_offsets",
            description: "Three UV samples (R/G/B) split radially. We sample synthetic_input at each.",
            slugs: &["postfx/chromatic_offsets"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let off = chromatic_offsets(in.uv, 0.04);
    let r = synthetic_input(off[0]).r;
    let g = synthetic_input(off[1]).g;
    let b = synthetic_input(off[2]).b;
    return vec4<f32>(r, g, b, 1.0);
}
"#,
        },
        Composition {
            name: "crt_curvature",
            description: "Barrel-warped UVs; out-of-bounds pixels darken to black.",
            slugs: &["postfx/crt_curvature"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let warp = crt_curvature(in.uv, 0.35);
    let inside = step(0.0, warp.z);
    let col = synthetic_input(warp.xy) * inside;
    return vec4<f32>(col, 1.0);
}
"#,
        },
        Composition {
            name: "film_grain",
            description: "Additive per-pixel grain (seed=0.7) on top of synthetic_input.",
            slugs: &["postfx/film_grain"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = synthetic_input(in.uv);
    let n = film_grain(in.uv, 0.7);
    return vec4<f32>(clamp(base + vec3<f32>(n) * 0.18, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "gaussian_weight_1d",
            description: "Plot of the 1D Gaussian (sigma=0.18) along x as a vertical bar height.",
            slugs: &["postfx/gaussian_weight_1d"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = in.uv.x - 0.5;
    let w = gaussian_weight_1d(x, 0.18);
    // Normalize peak (1/(sigma*sqrt(2pi))) to ~1 for display.
    let peak = gaussian_weight_1d(0.0, 0.18);
    let h = w / peak;
    let bar = step(1.0 - h, in.uv.y);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.55, 0.85, 0.95);
    return vec4<f32>(mix(bg, fg, bar), 1.0);
}
"#,
        },
        Composition {
            name: "invert",
            description: "1 - synthetic_input, channel-wise.",
            slugs: &["postfx/invert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = synthetic_input(in.uv);
    return vec4<f32>(invert(c), 1.0);
}
"#,
        },
        Composition {
            name: "pixelate_uv",
            description: "Snap UV to a 16x16 grid, then sample synthetic_input.",
            slugs: &["postfx/pixelate_uv"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let snapped = pixelate_uv(in.uv, vec2<f32>(16.0));
    return vec4<f32>(synthetic_input(snapped), 1.0);
}
"#,
        },
        Composition {
            name: "posterize",
            description: "Quantize synthetic_input to 4 steps per channel.",
            slugs: &["postfx/posterize"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = synthetic_input(in.uv);
    return vec4<f32>(posterize(c, 4.0), 1.0);
}
"#,
        },
        Composition {
            name: "scanlines",
            description: "Horizontal scanline multiplier (40 lines, 0.5 strength) over synthetic_input.",
            slugs: &["postfx/scanlines"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let s = scanlines(in.uv, 40.0, 0.5);
    let c = synthetic_input(in.uv);
    return vec4<f32>(c * s, 1.0);
}
"#,
        },
        Composition {
            name: "sharpen_3x3",
            description: "3x3 unsharp pass (amount=0.7) on synthetic_input, sampled at neighbor offsets.",
            slugs: &["postfx/sharpen_3x3"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = 1.0 / 256.0;
    let c00 = synthetic_input(in.uv + vec2<f32>(-px, -px));
    let c10 = synthetic_input(in.uv + vec2<f32>( 0.0, -px));
    let c20 = synthetic_input(in.uv + vec2<f32>( px, -px));
    let c01 = synthetic_input(in.uv + vec2<f32>(-px,  0.0));
    let c11 = synthetic_input(in.uv);
    let c21 = synthetic_input(in.uv + vec2<f32>( px,  0.0));
    let c02 = synthetic_input(in.uv + vec2<f32>(-px,  px));
    let c12 = synthetic_input(in.uv + vec2<f32>( 0.0,  px));
    let c22 = synthetic_input(in.uv + vec2<f32>( px,  px));
    let s = sharpen_3x3(c00, c10, c20, c01, c11, c21, c02, c12, c22, 0.7);
    return vec4<f32>(clamp(s, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#,
        },
        Composition {
            name: "sobel_magnitude",
            description: "Sobel edge magnitude on synthetic_input luminance, white edges on black.",
            slugs: &["postfx/sobel_magnitude"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = 1.0 / 256.0;
    let l00 = luma(synthetic_input(in.uv + vec2<f32>(-px, -px)));
    let l10 = luma(synthetic_input(in.uv + vec2<f32>( 0.0, -px)));
    let l20 = luma(synthetic_input(in.uv + vec2<f32>( px, -px)));
    let l01 = luma(synthetic_input(in.uv + vec2<f32>(-px,  0.0)));
    let l11 = luma(synthetic_input(in.uv));
    let l21 = luma(synthetic_input(in.uv + vec2<f32>( px,  0.0)));
    let l02 = luma(synthetic_input(in.uv + vec2<f32>(-px,  px)));
    let l12 = luma(synthetic_input(in.uv + vec2<f32>( 0.0,  px)));
    let l22 = luma(synthetic_input(in.uv + vec2<f32>( px,  px)));
    let m = sobel_magnitude(l00, l10, l20, l01, l11, l21, l02, l12, l22);
    let v = clamp(m * 6.0, 0.0, 1.0);
    return vec4<f32>(vec3<f32>(v), 1.0);
}
"#,
        },
        Composition {
            name: "threshold",
            description: "Binary step at t=0.5 across a horizontal gradient (vec3(uv.x)).",
            slugs: &["postfx/threshold"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let g = vec3<f32>(in.uv.x);
    return vec4<f32>(threshold(g, 0.5), 1.0);
}
"#,
        },
        Composition {
            name: "vignette",
            description: "Radial darkening (radius=0.4, softness=0.45) applied to synthetic_input.",
            slugs: &["postfx/vignette"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = vignette(in.uv, 0.4, 0.45);
    let c = synthetic_input(in.uv);
    return vec4<f32>(c * v, 1.0);
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
    let body = format!("{}{}{}", FULLSCREEN_VS, SYNTHETIC_INPUT, comp.fragment);
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
        let out_dir = std::path::Path::new("out/gallery_postfx");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering postfx gallery into {}/", out_dir.display());

        let mut failures: Vec<String> = Vec::new();
        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
                failures.push(comp.name.to_string());
            }
        }
        if !failures.is_empty() {
            return Err(format!("{} composition(s) failed: {:?}", failures.len(), failures).into());
        }
        Ok(())
    })
}
