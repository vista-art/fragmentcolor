//! Catalog gallery for the `intersect/` registry category.
//!
//! Renders one 256x256 PNG per ray-vs-shape intersection shader in
//! `docs/website/public/shaders/intersect/`. Each entry pulls a single
//! registry slug into a fullscreen fragment that casts a per-pixel ray
//! from a fixed camera and shades by hit distance / normal / slab depths.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_intersect

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
            name: "ray_box",
            description: "Ray vs AABB; encodes (t_near, t_far) as the red/green channels.",
            slugs: &["intersect/ray_box"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(ndc.x, -ndc.y, 1.5));
    let hit = ray_box(ro, rd, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.7, 0.5, 0.6));
    if (hit.x < 0.0) {
        return vec4<f32>(0.10, 0.12, 0.18, 1.0);
    }
    let near = clamp((hit.x - 1.5) * 0.7, 0.0, 1.0);
    let far  = clamp((hit.y - 1.5) * 0.7, 0.0, 1.0);
    return vec4<f32>(near, far, 0.2, 1.0);
}
"#,
        },
        Composition {
            name: "ray_capsule",
            description: "Ray vs capsule between (-0.5,-0.5,0) and (0.5,0.5,0), radius 0.35.",
            slugs: &["intersect/ray_capsule"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(ndc.x, -ndc.y, 1.5));
    let pa = vec3<f32>(-0.6, -0.5, 0.0);
    let pb = vec3<f32>( 0.6,  0.5, 0.0);
    let r  = 0.35;
    let t  = ray_capsule(ro, rd, pa, pb, r);
    if (t < 0.0) {
        return vec4<f32>(0.10, 0.12, 0.18, 1.0);
    }
    // Normal at hit: nearest segment point -> hit - segPoint, normalized.
    let p   = ro + rd * t;
    let ba  = pb - pa;
    let h   = clamp(dot(p - pa, ba) / dot(ba, ba), 0.0, 1.0);
    let n   = normalize(p - (pa + ba * h));
    let shade = clamp(n * 0.5 + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(shade, 1.0);
}
"#,
        },
        Composition {
            name: "ray_cylinder",
            description: "Ray vs infinite Y-axis cylinder (radius 0.6); RG = clamped (t_near, t_far).",
            slugs: &["intersect/ray_cylinder"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(ndc.x, -ndc.y, 1.5));
    let hit = ray_cylinder(ro, rd, 0.6);
    if (hit.x < 0.0 || hit.y < 0.0) {
        return vec4<f32>(0.10, 0.12, 0.18, 1.0);
    }
    let near = clamp((hit.x - 1.5) * 0.7, 0.0, 1.0);
    let far  = clamp((hit.y - 1.5) * 0.7, 0.0, 1.0);
    return vec4<f32>(near, far, 0.25, 1.0);
}
"#,
        },
        Composition {
            name: "ray_disk",
            description: "Ray vs disk centered at origin, normal toward the camera, radius 0.85.",
            slugs: &["intersect/ray_disk"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(ndc.x, -ndc.y, 1.5));
    // Tilt the disk slightly so it isn't pure facing — gives a nicer preview.
    let n  = normalize(vec3<f32>(0.25, 0.35, -1.0));
    let t  = ray_disk(ro, rd, vec3<f32>(0.0, 0.0, 0.0), n, 0.85);
    if (t < 0.0) {
        return vec4<f32>(0.10, 0.12, 0.18, 1.0);
    }
    let p = ro + rd * t;
    // Radial gradient inside the disk.
    let r = length(p) / 0.85;
    let col = mix(vec3<f32>(0.95, 0.65, 0.35), vec3<f32>(0.25, 0.15, 0.45), r);
    return vec4<f32>(col, 1.0);
}
"#,
        },
        Composition {
            name: "ray_plane",
            description: "Ray vs ground plane (y = -0.6); shaded by hit distance.",
            slugs: &["intersect/ray_plane"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.5, -2.5);
    let rd = normalize(vec3<f32>(ndc.x, -ndc.y, 1.5));
    // Plane: y = -0.6  ->  normal (0,1,0), distance d = 0.6.
    let n  = vec3<f32>(0.0, 1.0, 0.0);
    let d  = 0.6;
    let t  = ray_plane(ro, rd, n, d);
    if (t > 1.0e20 || t <= 0.0) {
        return vec4<f32>(0.10, 0.12, 0.18, 1.0);
    }
    let p = ro + rd * t;
    // Checker pattern on the plane to make the hit visible.
    let cell = step(0.0, sin(p.x * 4.0) * sin(p.z * 4.0));
    let shade = clamp(1.0 - t * 0.12, 0.05, 1.0);
    let col = mix(vec3<f32>(0.20, 0.30, 0.45), vec3<f32>(0.85, 0.85, 0.95), cell) * shade;
    return vec4<f32>(col, 1.0);
}
"#,
        },
        Composition {
            name: "ray_sphere",
            description: "Ray vs unit-ish sphere at origin, radius 0.85; shaded by surface normal.",
            slugs: &["intersect/ray_sphere"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(ndc.x, -ndc.y, 1.5));
    let center = vec3<f32>(0.0, 0.0, 0.0);
    let radius = 0.85;
    let hit = ray_sphere(ro, rd, center, radius);
    if (hit.x < 0.0 && hit.y < 0.0) {
        return vec4<f32>(0.10, 0.12, 0.18, 1.0);
    }
    let t = max(hit.x, 0.0);
    let p = ro + rd * t;
    let n = normalize(p - center);
    let shade = clamp(n * 0.5 + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(shade, 1.0);
}
"#,
        },
        Composition {
            name: "ray_triangle",
            description: "Möller-Trumbore vs a single large triangle; shaded by barycentric (u, v, 1-u-v).",
            slugs: &["intersect/ray_triangle"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ndc = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(ndc.x, -ndc.y, 1.5));
    let a = vec3<f32>( 0.0,  0.85, 0.0);
    let b = vec3<f32>(-0.85, -0.6, 0.0);
    let c = vec3<f32>( 0.85, -0.6, 0.0);
    let h = ray_triangle(ro, rd, a, b, c);
    if (h.x < 0.0) {
        return vec4<f32>(0.10, 0.12, 0.18, 1.0);
    }
    let u = h.y;
    let v = h.z;
    let w = 1.0 - u - v;
    return vec4<f32>(w, u, v, 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_intersect");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering intersect gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} - {}", comp.name, e);
            }
        }
        Ok(())
    })
}
