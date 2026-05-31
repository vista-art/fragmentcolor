//! Catalog gallery for the `geom/` registry category.
//!
//! Renders one 256x256 PNG per geometry helper in
//! `docs/website/public/shaders/geom/`. Each entry pulls a single registry
//! slug into a fullscreen fragment that calls the helper and visualises the
//! result over a muted background.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_geom

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
            name: "barycentric",
            description: "Barycentric coords of UV inside a fixed triangle, mapped to RGB.",
            slugs: &["geom/barycentric"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Place a triangle in the UV plane at fixed corners (z = 0).
    let a = vec3<f32>(0.5, 0.10, 0.0);
    let b = vec3<f32>(0.10, 0.85, 0.0);
    let c = vec3<f32>(0.90, 0.85, 0.0);
    let p = vec3<f32>(in.uv, 0.0);
    let bc = barycentric(p, a, b, c);
    let inside = step(0.0, bc.x) * step(0.0, bc.y) * step(0.0, bc.z);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(bc.x, bc.y, bc.z);
    return vec4<f32>(mix(bg, fg, inside), 1.0);
}
"#,
        },
        Composition {
            name: "quaternion_conjugate",
            description: "Conjugate of a quaternion built from UV-driven axis/angle, |q'.xyz|.",
            slugs: &[
                "geom/quaternion_from_axis_angle",
                "geom/quaternion_conjugate",
            ],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let axis = normalize(vec3<f32>(p.x, p.y, 0.5));
    let angle = length(p) * 3.14159;
    let q = quaternion_from_axis_angle(axis, angle);
    let qc = quaternion_conjugate(q);
    return vec4<f32>(abs(qc.xyz), 1.0);
}
"#,
        },
        Composition {
            name: "quaternion_from_axis_angle",
            description: "|xyz| of unit quaternion from radial axis + angle proportional to radius.",
            slugs: &["geom/quaternion_from_axis_angle"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let axis = normalize(vec3<f32>(p.x, p.y, 0.4));
    let angle = length(p) * 3.14159;
    let q = quaternion_from_axis_angle(axis, angle);
    return vec4<f32>(abs(q.xyz), 1.0);
}
"#,
        },
        Composition {
            name: "quaternion_mul",
            description: "Hamilton product of two UV-driven quaternions, abs of imaginary part.",
            slugs: &["geom/quaternion_from_axis_angle", "geom/quaternion_mul"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let qx = quaternion_from_axis_angle(vec3<f32>(1.0, 0.0, 0.0), p.x * 3.14159);
    let qy = quaternion_from_axis_angle(vec3<f32>(0.0, 1.0, 0.0), p.y * 3.14159);
    let q = quaternion_mul(qx, qy);
    return vec4<f32>(abs(q.xyz), 1.0);
}
"#,
        },
        Composition {
            name: "quaternion_rotate_vec",
            description: "Rotate +Z by a UV-driven quaternion; |result.xyz| as RGB.",
            slugs: &[
                "geom/quaternion_from_axis_angle",
                "geom/quaternion_rotate_vec",
            ],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let axis = normalize(vec3<f32>(p.x, p.y, 0.3));
    let angle = length(p) * 3.14159;
    let q = quaternion_from_axis_angle(axis, angle);
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let r = quaternion_rotate_vec(q, v);
    return vec4<f32>(abs(r), 1.0);
}
"#,
        },
        Composition {
            name: "quaternion_slerp",
            description: "Slerp between two fixed unit quaternions over t = u.x; |result.xyz|.",
            slugs: &["geom/quaternion_from_axis_angle", "geom/quaternion_slerp"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let qa = quaternion_from_axis_angle(vec3<f32>(1.0, 0.0, 0.0), 0.0);
    let qb = quaternion_from_axis_angle(vec3<f32>(0.0, 1.0, 0.0), 3.14159);
    // Use uv.x as interpolation parameter, modulate brightness with uv.y banding.
    let q = quaternion_slerp(qa, qb, in.uv.x);
    let band = 0.6 + 0.4 * sin(in.uv.y * 18.84);
    return vec4<f32>(abs(q.xyz) * band, 1.0);
}
"#,
        },
        Composition {
            name: "rotate_2d",
            description: "rotate_2d applied to UV grid; encode rotated coords as RG.",
            slugs: &["geom/rotate_2d"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let m = rotate_2d(0.7);
    let q = m * p;
    // Visualise rotated grid as stripes along q.x.
    let stripes = 0.5 + 0.5 * sin(q.x * 18.84);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.85, 0.55, 0.95);
    return vec4<f32>(mix(bg, fg, stripes), 1.0);
}
"#,
        },
        Composition {
            name: "rotate_3d_x",
            description: "Rotate vec3(p.x, p.y, 0.5) around +X by p.x*pi; show |result|.",
            slugs: &["geom/rotate_3d_x"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let m = rotate_3d_x(p.x * 3.14159);
    let v = vec3<f32>(p.x, p.y, 0.5);
    let r = m * v;
    return vec4<f32>(abs(r), 1.0);
}
"#,
        },
        Composition {
            name: "rotate_3d_y",
            description: "Rotate around +Y by p.x*pi; show |result|.",
            slugs: &["geom/rotate_3d_y"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let m = rotate_3d_y(p.x * 3.14159);
    let v = vec3<f32>(p.x, p.y, 0.5);
    let r = m * v;
    return vec4<f32>(abs(r), 1.0);
}
"#,
        },
        Composition {
            name: "rotate_3d_z",
            description: "Rotate around +Z by p.x*pi; show |result|.",
            slugs: &["geom/rotate_3d_z"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let m = rotate_3d_z(p.x * 3.14159);
    let v = vec3<f32>(p.x, p.y, 0.5);
    let r = m * v;
    return vec4<f32>(abs(r), 1.0);
}
"#,
        },
        Composition {
            name: "tbn_from_normal",
            description: "TBN basis from a UV-driven normal: rows visualised as tangent (R), bitangent (G), normal (B).",
            slugs: &["geom/tbn_from_normal"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let n = normalize(vec3<f32>(p.x, p.y, 0.6));
    let m = tbn_from_normal(n);
    // Combine basis lengths along axes so each region shows a different basis vector.
    let t = m[0];
    let b = m[1];
    let nn = m[2];
    let col = abs(t) * 0.4 + abs(b) * 0.3 + abs(nn) * 0.3;
    return vec4<f32>(col, 1.0);
}
"#,
        },
        Composition {
            name: "triangle_area",
            description: "Area of triangle (origin, (uv.x,0,0), (0,uv.y,0)) as a heat field.",
            slugs: &["geom/triangle_area"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let a = vec3<f32>(0.0, 0.0, 0.0);
    let b = vec3<f32>(in.uv.x, 0.0, 0.0);
    let c = vec3<f32>(0.0, in.uv.y, 0.0);
    let area = triangle_area(a, b, c);
    // area in [0, 0.5]; remap to [0, 1] for the gradient.
    let t = clamp(area * 2.0, 0.0, 1.0);
    let cold = vec3<f32>(0.10, 0.30, 0.85);
    let warm = vec3<f32>(0.95, 0.45, 0.20);
    return vec4<f32>(mix(cold, warm, t), 1.0);
}
"#,
        },
        Composition {
            name: "triangle_normal",
            description: "Normal of a triangle whose third vertex tracks the UV; |normal| as RGB.",
            slugs: &["geom/triangle_normal"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - vec2<f32>(0.5)) * 2.0;
    let a = vec3<f32>(-1.0, 0.0, 0.0);
    let b = vec3<f32>( 1.0, 0.0, 0.0);
    let c = vec3<f32>(p.x, p.y, 0.5);
    let n = triangle_normal(a, b, c);
    return vec4<f32>(abs(n), 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_geom");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering geom gallery into {}/", out_dir.display());

        let mut failures: Vec<String> = Vec::new();
        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
                failures.push(comp.name.to_string());
            }
        }
        if !failures.is_empty() {
            return Err(format!("failures: {:?}", failures).into());
        }
        Ok(())
    })
}
