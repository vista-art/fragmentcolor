//! Catalog gallery for the `lighting/` registry category.
//!
//! Renders one 256x256 PNG per lighting / BRDF helper in
//! `docs/website/public/shaders/lighting/`. Each entry pulls one or more
//! registry slugs into a fullscreen fragment that shades a unit sphere with
//! the helper, against a muted background.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_lighting

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
            name: "attenuation_inverse_square",
            description: "Inverse-square falloff; sphere brightness scales with 1/d^2 of a virtual point light.",
            slugs: &["lighting/attenuation_inverse_square", "lighting/lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    // Virtual point light positioned in front of and to the side of the sphere.
    let light_pos = vec3<f32>(0.6, 0.9, 1.6);
    let p = vec3<f32>(uv.x / 0.85, uv.y / 0.85, z);
    let to_light = light_pos - p;
    let d = length(to_light);
    let l = to_light / d;
    let atten = attenuation_inverse_square(d, 0.2);
    let albedo = vec3<f32>(0.85, 0.55, 0.35);
    let lit = albedo * lambert(n, l) * atten * 1.6;
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "attenuation_range",
            description: "Range-clamped inverse-square falloff (Unreal-style) with a finite cutoff.",
            slugs: &["lighting/attenuation_range", "lighting/lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let light_pos = vec3<f32>(0.7, 0.9, 1.4);
    let p = vec3<f32>(uv.x / 0.85, uv.y / 0.85, z);
    let to_light = light_pos - p;
    let d = length(to_light);
    let l = to_light / d;
    let atten = attenuation_range(d, 2.2);
    let albedo = vec3<f32>(0.55, 0.75, 0.95);
    let lit = albedo * lambert(n, l) * atten * 1.4;
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "blinn_phong",
            description: "Blinn half-vector specular highlight on a Lambert-shaded sphere.",
            slugs: &["lighting/blinn_phong", "lighting/lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let albedo = vec3<f32>(0.45, 0.55, 0.85);
    let diffuse = albedo * lambert(n, l) * 0.6;
    let spec = vec3<f32>(1.0) * blinn_phong(n, l, v, 64.0);
    return vec4<f32>(diffuse + spec, 1.0);
}
"#,
        },
        Composition {
            name: "cook_torrance",
            description: "Cook-Torrance specular BRDF (D*G*F/(4*NdV*NdL)), reddish copper f0, roughness 0.35.",
            slugs: &["lighting/cook_torrance", "lighting/lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let f0 = vec3<f32>(0.95, 0.64, 0.54);
    let albedo = vec3<f32>(0.55, 0.30, 0.20);
    let diffuse = albedo * lambert(n, l) * 0.4;
    let spec = cook_torrance(n, l, v, f0, 0.35) * max(dot(normalize(n), l), 0.0) * 4.0;
    return vec4<f32>(diffuse + spec, 1.0);
}
"#,
        },
        Composition {
            name: "disney_diffuse",
            description: "Burley/Disney diffuse term — energy-preserving rough diffuse.",
            slugs: &["lighting/disney_diffuse"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let albedo = vec3<f32>(0.85, 0.55, 0.45);
    let lit = albedo * disney_diffuse(n, l, v, 0.6) * 3.14159265;
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "fresnel_schlick",
            description: "Schlick Fresnel reflectance at grazing angles, plastic f0 = 0.04.",
            slugs: &["lighting/fresnel_schlick"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let cos_theta = max(dot(n, v), 0.0);
    let f0 = vec3<f32>(0.04);
    let f = fresnel_schlick(cos_theta, f0);
    let albedo = vec3<f32>(0.18, 0.30, 0.55);
    let lit = mix(albedo, vec3<f32>(1.0), f);
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "fresnel_schlick_roughness",
            description: "Roughness-aware Schlick (Lagarde) — Fresnel attenuated by surface roughness.",
            slugs: &["lighting/fresnel_schlick_roughness"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let cos_theta = max(dot(n, v), 0.0);
    let f0 = vec3<f32>(0.04);
    let f = fresnel_schlick_roughness(cos_theta, f0, 0.5);
    let albedo = vec3<f32>(0.20, 0.55, 0.45);
    let lit = mix(albedo, vec3<f32>(0.9), f);
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "ggx_d",
            description: "GGX (Trowbridge-Reitz) normal distribution — narrow specular lobe at roughness 0.25.",
            slugs: &["lighting/ggx_d"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let h = normalize(l + v);
    let d = ggx_d(n, h, 0.25);
    let albedo = vec3<f32>(0.20, 0.22, 0.30);
    let lit = albedo + vec3<f32>(1.0, 0.95, 0.85) * d * 0.05;
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "half_lambert",
            description: "Valve-style wrapped diffuse — softer terminator than pure Lambert.",
            slugs: &["lighting/half_lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let albedo = vec3<f32>(0.85, 0.65, 0.45);
    let lit = albedo * half_lambert(n, l);
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "lambert",
            description: "Classic Lambertian diffuse — clamped n.l shading.",
            slugs: &["lighting/lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let albedo = vec3<f32>(0.70, 0.80, 0.95);
    let lit = albedo * lambert(n, l);
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "oren_nayar",
            description: "Oren-Nayar rough-surface diffuse, roughness 0.7 — flat dusty look.",
            slugs: &["lighting/oren_nayar"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let albedo = vec3<f32>(0.85, 0.75, 0.60);
    let lit = albedo * oren_nayar(n, l, v, 0.7);
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "phong",
            description: "Classic Phong specular highlight; reflect+exponent term.",
            slugs: &["lighting/phong", "lighting/lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let albedo = vec3<f32>(0.55, 0.85, 0.45);
    let diffuse = albedo * lambert(n, l) * 0.6;
    let spec = vec3<f32>(1.0) * phong(n, l, v, 32.0);
    return vec4<f32>(diffuse + spec, 1.0);
}
"#,
        },
        Composition {
            name: "rim",
            description: "Rim / back-light term — (1 - n.v)^power adds an edge glow.",
            slugs: &["lighting/rim", "lighting/lambert"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let albedo = vec3<f32>(0.20, 0.25, 0.35);
    let diffuse = albedo * lambert(n, l) * 0.6;
    let rim_term = rim(n, v, 2.5);
    let lit = diffuse + vec3<f32>(0.95, 0.7, 1.0) * rim_term;
    return vec4<f32>(lit, 1.0);
}
"#,
        },
        Composition {
            name: "smith_g",
            description: "Smith masking/shadowing geometry term combined for view + light directions.",
            slugs: &["lighting/smith_g"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let r = length(uv);
    if (r > 0.85) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let z = sqrt(0.85*0.85 - r*r) / 0.85;
    let n = normalize(vec3<f32>(uv.x / 0.85, uv.y / 0.85, z));
    let v = vec3<f32>(0.0, 0.0, 1.0);
    let l = normalize(vec3<f32>(0.4, 0.6, 0.7));
    let g = smith_g(n, v, l, 0.4);
    let albedo = vec3<f32>(0.55, 0.65, 0.85);
    let lit = albedo * g;
    return vec4<f32>(lit, 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_lighting");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering lighting gallery into {}/", out_dir.display());

        let mut errors = 0usize;
        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  FAIL {} -- {}", comp.name, e);
                errors += 1;
            }
        }
        if errors > 0 {
            return Err(format!("{} composition(s) failed", errors).into());
        }
        Ok(())
    })
}
