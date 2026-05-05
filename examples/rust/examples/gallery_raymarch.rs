//! Catalog gallery for the `raymarch/` registry category.
//!
//! Renders one 256x256 PNG per raymarching helper in
//! `docs/website/public/shaders/raymarch/`. Each entry pulls the helper
//! plus `sdf/sphere` into a fullscreen fragment that runs an inline
//! sphere-trace loop, then uses the helper to estimate the surface
//! normal at the hit point and shade the result with simple Lambert.
//!
//! The helpers in this category are pure normal estimators: the caller
//! evaluates its scene SDF at a few offsets and passes the resulting
//! distances. The host fragment owns the raymarching loop and the scene
//! definition (here, a single sphere of radius 0.7 at the origin).
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_raymarch

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
            name: "normal_from_sdf_taps",
            description:
                "6-tap central-difference normal from sphere SDF samples after a sphere-trace hit.",
            slugs: &["raymarch/normal_from_sdf_taps", "sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return sphere(p, 0.7);
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(uv.x, -uv.y, 1.5));

    var t: f32 = 0.0;
    var hit: bool = false;
    for (var i: i32 = 0; i < 64; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { hit = true; break; }
        t = t + d;
        if (t > 10.0) { break; }
    }

    if (hit) {
        let p = ro + rd * t;
        let e: f32 = 0.001;
        let d_px = scene(p + vec3<f32>( e, 0.0, 0.0));
        let d_nx = scene(p + vec3<f32>(-e, 0.0, 0.0));
        let d_py = scene(p + vec3<f32>(0.0,  e, 0.0));
        let d_ny = scene(p + vec3<f32>(0.0, -e, 0.0));
        let d_pz = scene(p + vec3<f32>(0.0, 0.0,  e));
        let d_nz = scene(p + vec3<f32>(0.0, 0.0, -e));
        let n = normal_from_sdf_taps(d_px, d_nx, d_py, d_ny, d_pz, d_nz);
        let l = normalize(vec3<f32>(0.4, 0.6, -0.7));
        let lambert = max(0.0, dot(n, l));
        let base = vec3<f32>(0.92, 0.62, 0.45);
        return vec4<f32>(base * (0.2 + 0.8 * lambert), 1.0);
    }
    return vec4<f32>(0.10, 0.12, 0.18, 1.0);
}
"#,
        },
        Composition {
            name: "normal_from_sdf_tetra",
            description:
                "Cheaper 4-tap tetrahedral normal from sphere SDF samples after a sphere-trace hit.",
            slugs: &["raymarch/normal_from_sdf_tetra", "sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return sphere(p, 0.7);
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(uv.x, -uv.y, 1.5));

    var t: f32 = 0.0;
    var hit: bool = false;
    for (var i: i32 = 0; i < 64; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { hit = true; break; }
        t = t + d;
        if (t > 10.0) { break; }
    }

    if (hit) {
        let p = ro + rd * t;
        let e: f32 = 0.001;
        let k1 = scene(p + e * vec3<f32>( 1.0, -1.0, -1.0));
        let k2 = scene(p + e * vec3<f32>(-1.0, -1.0,  1.0));
        let k3 = scene(p + e * vec3<f32>(-1.0,  1.0, -1.0));
        let k4 = scene(p + e * vec3<f32>( 1.0,  1.0,  1.0));
        let n = normal_from_sdf_tetra(k1, k2, k3, k4);
        let l = normalize(vec3<f32>(0.4, 0.6, -0.7));
        let lambert = max(0.0, dot(n, l));
        let base = vec3<f32>(0.55, 0.85, 0.75);
        return vec4<f32>(base * (0.2 + 0.8 * lambert), 1.0);
    }
    return vec4<f32>(0.10, 0.12, 0.18, 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_raymarch");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering raymarch gallery into {}/", out_dir.display());

        let mut failed = false;
        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} -- {}", comp.name, e);
                failed = true;
            }
        }
        if failed {
            return Err("one or more compositions failed to render".into());
        }
        Ok(())
    })
}
