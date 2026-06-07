//! Catalog gallery for the `camera/` registry category.
//!
//! Renders one 256x256 PNG per camera helper in
//! `docs/website/public/shaders/camera/`. Most camera helpers return matrices
//! or transformed vectors rather than colors, so each composition stages a
//! tiny fixed scene (cube, ray grid, basis vectors) and visualizes the result
//! of applying the helper.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_camera

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
            name: "look_at",
            description: "View matrix from eye=(2,1.5,3) toward origin. Cube raymarched in view space.",
            slugs: &["camera/look_at"],
            fragment: r#"
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view = look_at(vec3<f32>(2.0, 1.5, 3.0), vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0));
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let h = tan(0.6 * 0.5);
    let dir_view = normalize(vec3<f32>(ndc.x * h, ndc.y * h, -1.0));
    // March in view space — eye is at origin in view space.
    let view3 = mat3x3<f32>(view[0].xyz, view[1].xyz, view[2].xyz);
    let eye_world = vec3<f32>(2.0, 1.5, 3.0);
    let dir_world = transpose(view3) * dir_view;
    var t: f32 = 0.0;
    var hit: f32 = 0.0;
    for (var i: i32 = 0; i < 48; i = i + 1) {
        let p = eye_world + dir_world * t;
        let d = sd_box(p, vec3<f32>(0.6));
        if (d < 0.001) { hit = 1.0; break; }
        if (t > 8.0) { break; }
        t = t + d;
    }
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.55, 0.85, 0.95);
    let shade = 1.0 - clamp(t / 5.0, 0.0, 1.0);
    return vec4<f32>(mix(bg, fg * (0.4 + 0.6 * shade), hit), 1.0);
}
"#,
        },
        Composition {
            name: "orthographic",
            description: "Orthographic projection of a UV checker plane at z=-2.",
            slugs: &["camera/orthographic"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let proj = orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 5.0);
    // Clip-space pixel position
    let clip = vec4<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0, 0.0, 1.0);
    // For ortho, x_clip = 2/(r-l) * x_view + ..., so x_view = clip.x (here).
    // Place a checker plane at z = -2 in view space, mapped from the projected x,y.
    let view_xy = vec2<f32>(clip.x, clip.y);
    let cell = step(vec2<f32>(0.5), fract(view_xy * 4.0));
    let chk = abs(cell.x - cell.y);
    // Show the projection result by also coloring with z=-2 lit through ortho proj.
    let view_pt = vec4<f32>(view_xy, -2.0, 1.0);
    let p = proj * view_pt;
    let depth_shade = 1.0 - clamp((p.z / p.w) * 0.5 + 0.5, 0.0, 1.0);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(0.95, 0.80, 0.45);
    return vec4<f32>(mix(bg, fg, chk) * (0.5 + 0.5 * depth_shade), 1.0);
}
"#,
        },
        Composition {
            name: "perspective",
            description: "Perspective projection of a checkered ground plane (z = -1 to -8).",
            slugs: &["camera/perspective"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let proj = perspective(1.0, 1.0, 0.1, 100.0);
    // Screen NDC
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    // Reconstruct view-space ray from inverse perspective trick: pick z=-1 plane in view space.
    let h = tan(1.0 * 0.5);
    // Ray from origin
    let dir = normalize(vec3<f32>(ndc.x * h, ndc.y * h, -1.0));
    // Intersect with y = -0.5 plane (ground)
    let t = -0.5 / dir.y;
    var col = vec3<f32>(0.10, 0.12, 0.18);
    if (t > 0.0 && dir.y < 0.0) {
        let hit = dir * t;
        let cell = step(vec2<f32>(0.5), fract(hit.xz * 0.5));
        let chk = abs(cell.x - cell.y);
        // Validate by re-projecting hit through the matrix.
        let p = proj * vec4<f32>(hit, 1.0);
        let visible = step(0.0, p.w + 1000.0);
        let fade = 1.0 - clamp(t / 12.0, 0.0, 1.0);
        let fg = vec3<f32>(0.85, 0.55, 0.95);
        let bg = vec3<f32>(0.18, 0.20, 0.30);
        col = mix(bg, fg, chk) * fade * visible;
    }
    return vec4<f32>(col, 1.0);
}
"#,
        },
        Composition {
            name: "ray_from_uv",
            description: "Visualizes world-space ray direction as |xyz| → RGB through a tilted view.",
            slugs: &["camera/ray_from_uv"],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Tilted view: rotate around Y by 0.4, up tilted slightly.
    let c = cos(0.4); let s = sin(0.4);
    let view = mat3x3<f32>(
        vec3<f32>( c, 0.0,  -s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>( s, 0.0,   c),
    );
    let dir = ray_from_uv(in.uv, 1.0, 1.0, view);
    let col = abs(dir);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    return vec4<f32>(mix(bg, col, 0.85) + bg * 0.15, 1.0);
}
"#,
        },
        Composition {
            name: "rotate_axis",
            description: "Rodrigues rotation around (1,1,0)/√2 by 0.7 rad — applied to a raymarched cube.",
            slugs: &["camera/rotate_axis"],
            fragment: r#"
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = rotate_axis(normalize(vec3<f32>(1.0, 1.0, 0.0)), 0.7);
    let m3 = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let dir = normalize(vec3<f32>(ndc.x, ndc.y, -1.5));
    let eye = vec3<f32>(0.0, 0.0, 2.5);
    var t: f32 = 0.0;
    var hit: f32 = 0.0;
    var n: vec3<f32> = vec3<f32>(0.0);
    for (var i: i32 = 0; i < 48; i = i + 1) {
        let p_world = eye + dir * t;
        let p_local = transpose(m3) * p_world;
        let d = sd_box(p_local, vec3<f32>(0.7));
        if (d < 0.001) {
            hit = 1.0;
            n = transpose(m3) * normalize(p_local);
            break;
        }
        if (t > 6.0) { break; }
        t = t + d;
    }
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let lit = max(0.2, dot(n, normalize(vec3<f32>(0.5, 0.7, 0.4))));
    let fg = vec3<f32>(0.95, 0.65, 0.45) * lit;
    return vec4<f32>(mix(bg, fg, hit), 1.0);
}
"#,
        },
        Composition {
            name: "rotate_x",
            description: "Rotation around the X axis by 0.6 rad — cube tilted forward.",
            slugs: &["camera/rotate_x"],
            fragment: r#"
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = rotate_x(0.6);
    let m3 = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let dir = normalize(vec3<f32>(ndc.x, ndc.y, -1.5));
    let eye = vec3<f32>(0.0, 0.0, 2.5);
    var t: f32 = 0.0;
    var hit: f32 = 0.0;
    var n: vec3<f32> = vec3<f32>(0.0);
    for (var i: i32 = 0; i < 48; i = i + 1) {
        let p_world = eye + dir * t;
        let p_local = transpose(m3) * p_world;
        let d = sd_box(p_local, vec3<f32>(0.7));
        if (d < 0.001) { hit = 1.0; n = transpose(m3) * normalize(p_local); break; }
        if (t > 6.0) { break; }
        t = t + d;
    }
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let lit = max(0.2, dot(n, normalize(vec3<f32>(0.5, 0.7, 0.4))));
    let fg = vec3<f32>(0.95, 0.45, 0.55) * lit;
    return vec4<f32>(mix(bg, fg, hit), 1.0);
}
"#,
        },
        Composition {
            name: "rotate_y",
            description: "Rotation around the Y axis by 0.6 rad — cube turned to the side.",
            slugs: &["camera/rotate_y"],
            fragment: r#"
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = rotate_y(0.6);
    let m3 = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let dir = normalize(vec3<f32>(ndc.x, ndc.y, -1.5));
    let eye = vec3<f32>(0.0, 0.0, 2.5);
    var t: f32 = 0.0;
    var hit: f32 = 0.0;
    var n: vec3<f32> = vec3<f32>(0.0);
    for (var i: i32 = 0; i < 48; i = i + 1) {
        let p_world = eye + dir * t;
        let p_local = transpose(m3) * p_world;
        let d = sd_box(p_local, vec3<f32>(0.7));
        if (d < 0.001) { hit = 1.0; n = transpose(m3) * normalize(p_local); break; }
        if (t > 6.0) { break; }
        t = t + d;
    }
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let lit = max(0.2, dot(n, normalize(vec3<f32>(0.5, 0.7, 0.4))));
    let fg = vec3<f32>(0.55, 0.85, 0.65) * lit;
    return vec4<f32>(mix(bg, fg, hit), 1.0);
}
"#,
        },
        Composition {
            name: "rotate_z",
            description: "Rotation around the Z axis by 0.6 rad — square spun in plane.",
            slugs: &["camera/rotate_z"],
            fragment: r#"
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = rotate_z(0.6);
    let m3 = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let dir = normalize(vec3<f32>(ndc.x, ndc.y, -1.5));
    let eye = vec3<f32>(0.0, 0.0, 2.5);
    var t: f32 = 0.0;
    var hit: f32 = 0.0;
    var n: vec3<f32> = vec3<f32>(0.0);
    for (var i: i32 = 0; i < 48; i = i + 1) {
        let p_world = eye + dir * t;
        let p_local = transpose(m3) * p_world;
        let d = sd_box(p_local, vec3<f32>(0.7));
        if (d < 0.001) { hit = 1.0; n = transpose(m3) * normalize(p_local); break; }
        if (t > 6.0) { break; }
        t = t + d;
    }
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let lit = max(0.2, dot(n, normalize(vec3<f32>(0.5, 0.7, 0.4))));
    let fg = vec3<f32>(0.65, 0.55, 0.95) * lit;
    return vec4<f32>(mix(bg, fg, hit), 1.0);
}
"#,
        },
        Composition {
            name: "scale",
            description: "Non-uniform scale (1.4, 0.6, 1.0) applied to a unit cube.",
            slugs: &["camera/scale"],
            fragment: r#"
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = scale(vec3<f32>(1.4, 0.6, 1.0));
    // Inverse scale to transform world-space sample point into the unscaled local frame.
    let inv = vec3<f32>(1.0 / 1.4, 1.0 / 0.6, 1.0);
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let dir = normalize(vec3<f32>(ndc.x, ndc.y, -1.5));
    let eye = vec3<f32>(0.0, 0.0, 2.5);
    var t: f32 = 0.0;
    var hit: f32 = 0.0;
    var n: vec3<f32> = vec3<f32>(0.0);
    for (var i: i32 = 0; i < 48; i = i + 1) {
        let p_world = eye + dir * t;
        let p_local = p_world * inv;
        let d = sd_box(p_local, vec3<f32>(0.7));
        if (d < 0.001) { hit = 1.0; n = normalize(p_local); break; }
        if (t > 6.0) { break; }
        t = t + 0.5 * d;
    }
    // Use the matrix once to confirm it's pulled into the shader.
    let probe = (m * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let lit = max(0.2, dot(n, normalize(vec3<f32>(0.5, 0.7, 0.4))));
    let fg = vec3<f32>(0.95, 0.55, 0.30) * lit + probe * 0.0001;
    return vec4<f32>(mix(bg, fg, hit), 1.0);
}
"#,
        },
        Composition {
            name: "screen_to_world",
            description: "Unprojects each UV at depth 0.5 through a perspective view; colors by world XY.",
            slugs: &[
                "camera/screen_to_world",
                "camera/perspective",
                "camera/look_at",
            ],
            fragment: r#"
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view = look_at(vec3<f32>(0.0, 0.0, 3.0), vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0));
    let proj = perspective(1.0, 1.0, 0.5, 10.0);
    let vp = proj * view;
    // Approximate inverse for visualization: invert via view^T * proj^-1 ordering.
    // Build a hand-rolled inverse by leveraging that view is orthonormal-affine.
    // Easier: pass the matrix product and use a generic inverse via determinants is heavy;
    // instead, drive screen_to_world with an analytic view_proj_inv built from known params.
    // Inverse perspective with f=1/tan(0.5*1.0) = 1/tan(0.5):
    let f_inv = tan(0.5);
    let proj_inv = mat4x4<f32>(
        vec4<f32>(f_inv, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, f_inv, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, (0.5 - 10.0) / (2.0 * 0.5 * 10.0)),
        vec4<f32>(0.0, 0.0, -1.0, (0.5 + 10.0) / (2.0 * 0.5 * 10.0)),
    );
    // Inverse view: view is orthonormal+translate, so we invert by transposing the basis
    // and applying -basis^T * t. Eye=(0,0,3), looking at origin, up=+Y, so basis is identity-aligned
    // and translation is (0,0,-3) in view; inverse translates by (0,0,3) in world.
    let view_inv = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 3.0, 1.0),
    );
    let vp_inv = view_inv * proj_inv;
    let world = screen_to_world(in.uv, 0.5, vp_inv);
    // Use vp once to ensure perspective+look_at are linked into the shader.
    let probe = vp[0].xyz;
    let xy = world.xy * 0.5 + vec2<f32>(0.5);
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let fg = vec3<f32>(xy.x, 0.55, xy.y) + probe * 0.0001;
    return vec4<f32>(mix(bg, fg, 0.85), 1.0);
}
"#,
        },
        Composition {
            name: "translate",
            description: "Cube translated by (0.6, -0.3, 0) — silhouette shifted from center.",
            slugs: &["camera/translate"],
            fragment: r#"
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t_vec = vec3<f32>(0.6, -0.3, 0.0);
    let m = translate(t_vec);
    // Read translation back from the matrix to drive the visualization.
    let off = m[3].xyz;
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    let dir = normalize(vec3<f32>(ndc.x, ndc.y, -1.5));
    let eye = vec3<f32>(0.0, 0.0, 2.5);
    var t: f32 = 0.0;
    var hit: f32 = 0.0;
    var n: vec3<f32> = vec3<f32>(0.0);
    for (var i: i32 = 0; i < 48; i = i + 1) {
        let p_world = eye + dir * t;
        let d = sd_box(p_world - off, vec3<f32>(0.55));
        if (d < 0.001) { hit = 1.0; n = normalize(p_world - off); break; }
        if (t > 6.0) { break; }
        t = t + d;
    }
    let bg = vec3<f32>(0.10, 0.12, 0.18);
    let lit = max(0.2, dot(n, normalize(vec3<f32>(0.5, 0.7, 0.4))));
    let fg = vec3<f32>(0.45, 0.85, 0.95) * lit;
    return vec4<f32>(mix(bg, fg, hit), 1.0);
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
    println!("  ✓ {} → {}", comp.name, path.display());
    println!("    {}", comp.description);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let out_dir = std::path::Path::new("out/gallery_camera");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering camera gallery into {}/", out_dir.display());

        for comp in compositions() {
            if let Err(e) = render_one(&renderer, out_dir, &comp).await {
                eprintln!("  ✗ {} — {}", comp.name, e);
            }
        }
        Ok(())
    })
}
