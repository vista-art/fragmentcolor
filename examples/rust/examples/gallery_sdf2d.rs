//! Catalog gallery for the `sdf2d/` registry category.
//!
//! Renders one 256x256 PNG per 2D signed-distance shader in
//! `docs/website/public/shaders/sdf2d/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that calls the SDF and threshold-shades it
//! against a muted background.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_sdf2d

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
            name: "box",
            description: "Axis-aligned rectangle with half-extents (0.6, 0.45).",
            slugs: &["sdf2d/box"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let d = box(p, vec2<f32>(0.6, 0.45));
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.55, 0.85);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "circle",
            description: "Solid disc of radius 0.7 at the origin.",
            slugs: &["sdf2d/circle"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let d = circle(p, 0.7);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.92, 0.62, 0.45);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "equilateral_triangle",
            description: "Equilateral triangle, circumradius 0.75, pointing +y in math coords.",
            slugs: &["sdf2d/equilateral_triangle"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    // Flip y so the triangle points up on screen (UVs are y-down).
    let d = equilateral_triangle(vec2<f32>(p.x, -p.y), 0.75);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.55, 0.85, 0.75);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "heart",
            description: "Heart shape, centered and filling most of the canvas.",
            slugs: &["sdf2d/heart"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Larger heart scale ~doubles the on-screen size vs the old preview; the
    // y offset recenters the shape's body on the canvas before the y-flip.
    let p = (in.uv - vec2<f32>(0.5, 0.87)) * 2.0;
    let d = heart(vec2<f32>(p.x, -p.y), 1.45);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.40, 0.50);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "hexagon",
            description: "Regular hexagon, flat top, circumradius 0.7.",
            slugs: &["sdf2d/hexagon"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let d = hexagon(p, 0.7);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.85, 0.80, 0.45);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "pie",
            description: "Pie-slice spanning 100 degrees, radius 0.85, point at center.",
            slugs: &["sdf2d/pie"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Recenter so the slice's tip sits near the midline; flip y so the
    // wedge opens upward on a y-down UV surface.
    let p = (in.uv - vec2<f32>(0.5, 0.6)) * 2.0;
    let half_angle = 0.872665; // ~50 degrees in radians
    let c = vec2<f32>(sin(half_angle), cos(half_angle));
    let d = pie(vec2<f32>(p.x, -p.y), c, 0.85);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.70, 0.35);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "rhombus",
            description: "Rhombus with axis half-lengths (0.7, 0.5).",
            slugs: &["sdf2d/rhombus"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let d = rhombus(p, vec2<f32>(0.7, 0.5));
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.65, 0.55, 0.95);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "ring",
            description: "Annulus with outer radius 0.7 and thickness 0.08.",
            slugs: &["sdf2d/ring"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let d = ring(p, 0.65, 0.08);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.45, 0.85, 0.90);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "rounded_box",
            description: "Rectangle with per-corner radii (0.20, 0.05, 0.20, 0.05).",
            slugs: &["sdf2d/rounded_box"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let r = vec4<f32>(0.20, 0.05, 0.20, 0.05);
    let d = rounded_box(p, vec2<f32>(0.6, 0.45), r);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.55, 0.65);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "segment",
            description: "Distance-shaded line segment from (-0.6, -0.5) to (0.6, 0.5), thickness 0.06.",
            slugs: &["sdf2d/segment"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    // segment returns unsigned distance; subtract a thickness to get an SDF.
    let d = segment(p, vec2<f32>(-0.6, -0.5), vec2<f32>(0.6, 0.5)) - 0.06;
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.85, 0.85, 0.95);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "star",
            description: "5-pointed star, outer radius 0.75, sharpness m = 3.0.",
            slugs: &["sdf2d/star"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let d = star(p, 0.75, 5u, 3.0);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.85, 0.45);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "trapezoid",
            description: "Isoceles trapezoid: top half-width 0.35, bottom 0.65, half-height 0.45.",
            slugs: &["sdf2d/trapezoid"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    // Flip y so the wider base sits at the bottom of the image.
    let d = trapezoid(vec2<f32>(p.x, -p.y), 0.35, 0.65, 0.45);
    let inside = 1.0 - smoothstep(-0.005, 0.005, d);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.55, 0.75, 0.95);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_sdf2d");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering sdf2d gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
