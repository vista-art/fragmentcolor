//! Catalog gallery for the `math/` registry category.
//!
//! Renders one 256x256 PNG per math helper in
//! `docs/website/public/shaders/math/`. Scalar 1D functions are visualized
//! as a `y = fn(uv.x)` plot on a dark grid; 2D functions are rendered as a
//! field across UV space.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_math

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
            name: "checker",
            description: "Classic 0/1 checkerboard with 8 cells per UV.",
            slugs: &["math/checker"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = checker(in.uv, 8.0);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.65, 0.40);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "grid_aa",
            description: "Anti-aliased grid lines, period 0.1, line half-width 0.005.",
            slugs: &["math/grid_aa"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = grid_aa(in.uv, 0.1, 0.005);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.55, 0.85, 0.75);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "line_aa",
            description: "Vertical anti-aliased line at x = 0.5 with half-width 0.02.",
            slugs: &["math/line_aa"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = line_aa(0.5, in.uv.x, 0.02);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.65, 0.40);
    return vec4<f32>(mix(bg, fg, m), 1.0);
}
"#,
        },
        Composition {
            name: "ndc_to_uv",
            description: "Visualises the NDC->UV remap as the y-flipped UV gradient.",
            slugs: &["math/ndc_to_uv"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let mapped = ndc_to_uv(ndc);
    return vec4<f32>(mapped.x, mapped.y, 0.35, 1.0);
}
"#,
        },
        Composition {
            name: "pulse",
            description: "Hard pulse plot: 1.0 inside [0.3, 0.7], 0 elsewhere.",
            slugs: &["math/pulse"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    let y = pulse(0.3, 0.7, uv.x);
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
}
"#,
        },
        Composition {
            name: "remap",
            description: "Linear remap [0, 1] -> [0.2, 0.8] (no clamp); plotted vs uv.x.",
            slugs: &["math/remap"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    let y = remap(uv.x, 0.0, 1.0, 0.2, 0.8);
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
}
"#,
        },
        Composition {
            name: "remap_clamped",
            description: "Linear remap [0.2, 0.8] -> [0.05, 0.95] with clamping; plotted vs uv.x.",
            slugs: &["math/remap_clamped"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    let y = remap_clamped(uv.x, 0.2, 0.8, 0.05, 0.95);
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
}
"#,
        },
        Composition {
            name: "rsqrt",
            description: "y = rsqrt(x) plotted over uv.x in [0, 1], scaled by 0.25 to fit.",
            slugs: &["math/rsqrt"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    // rsqrt blows up near zero; scale so the curve stays in view.
    let x = max(uv.x, 0.05);
    let y = rsqrt(x) * 0.25;
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
}
"#,
        },
        Composition {
            name: "saturate",
            description: "y = saturate(2x - 0.5) plotted vs uv.x; clips to [0, 1].",
            slugs: &["math/saturate"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    let y = saturate(uv.x * 2.0 - 0.5);
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
}
"#,
        },
        Composition {
            name: "smooth_pulse",
            description: "Soft pulse with smoothstep edges of width 0.05 between 0.25 and 0.75.",
            slugs: &["math/smooth_pulse"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    let y = smooth_pulse(0.25, 0.75, 0.05, uv.x);
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
}
"#,
        },
        Composition {
            name: "smootherstep",
            description: "Perlin's 6t^5 - 15t^4 + 10t^3 over uv.x in [0, 1].",
            slugs: &["math/smootherstep"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    let y = smootherstep(0.0, 1.0, uv.x);
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
}
"#,
        },
        Composition {
            name: "step_aa",
            description: "Anti-aliased step with edge 0.5, fwidth proxy 0.04, plotted vs uv.x.",
            slugs: &["math/step_aa"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let plot_y = 1.0 - uv.y;
    let y = step_aa(0.5, uv.x, 0.04);
    let grid = step(0.97, max(fract(uv.x * 10.0), fract(uv.y * 10.0)));
    let bg = mix(vec3<f32>(0.10, 0.12, 0.18), vec3<f32>(0.18, 0.20, 0.26), grid);
    let dist = abs(plot_y - y);
    let curve = 1.0 - smoothstep(0.0, 0.012, dist);
    return vec4<f32>(mix(bg, vec3<f32>(0.95, 0.65, 0.40), curve), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_math");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering math gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
