//! Catalog gallery for the `sdf/` registry category.
//!
//! Renders one 256x256 PNG per 3D signed-distance shader in
//! `docs/website/public/shaders/sdf/`. Each entry pulls one or more
//! registry slugs into a fullscreen fragment that runs a minimal
//! sphere-trace + Lambert shade against a muted background.
//!
//! Run with:
//!   cargo run --release -p fce --example gallery_sdf

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
            description: "Axis-aligned box with half-extents (0.55, 0.55, 0.55).",
            slugs: &["sdf/box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return box(p, vec3<f32>(0.55, 0.55, 0.55));
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.55, 0.85);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "capsule",
            description: "Vertical capsule between (0, -0.55, 0) and (0, 0.55, 0), radius 0.35.",
            slugs: &["sdf/capsule"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return capsule(p, vec3<f32>(0.0, -0.55, 0.0), vec3<f32>(0.0, 0.55, 0.0), 0.35);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.55, 0.85, 0.75);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "cone",
            description: "Cone with half-angle ~25 deg, apex up and centered.",
            slugs: &["sdf/cone"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    // Center the cone in view; flip y so the infinite cone (which opens
    // toward -y) points apex-up on screen, then cap the base. c = (sin, cos)
    // of the slant; (sin 65, cos 65) gives a slim cone (~25-deg half-angle
    // measured from the axis) rather than the squat wide-angle default.
    let q = p + vec3<f32>(0.0, 0.7, 0.0);
    let c = vec2<f32>(0.9063, 0.4226);
    return max(cone(vec3<f32>(q.x, -q.y, q.z), c), q.y - 1.4);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.70, 0.35);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "cylinder",
            description: "Capped vertical cylinder, half-height 0.55, radius 0.45.",
            slugs: &["sdf/cylinder"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return cylinder(p, 0.55, 0.45);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.65, 0.55, 0.95);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "ellipsoid",
            description: "Ellipsoid with axis radii (0.7, 0.45, 0.5).",
            slugs: &["sdf/ellipsoid"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return ellipsoid(p, vec3<f32>(0.7, 0.45, 0.5));
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.85, 0.80, 0.45);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "hexagonal_prism",
            description: "Hexagonal prism with circumradius 0.55, half-height 0.5.",
            slugs: &["sdf/hexagonal_prism"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return hexagonal_prism(p, vec2<f32>(0.55, 0.5));
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.45, 0.85, 0.55);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "octahedron",
            description: "Regular octahedron with half-diagonal 0.7.",
            slugs: &["sdf/octahedron"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return octahedron(p, 0.7);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.45, 0.50);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_bend",
            description: "op_bend applied to a tall rounded box: bends the box around the Z axis with k = 1.2.",
            slugs: &["sdf/op_bend", "sdf/rounded_box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let q = op_bend(p, 1.2);
    return rounded_box(q, vec3<f32>(0.7, 0.18, 0.18), 0.05);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 96; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d * 0.6;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.85, 0.55, 0.40);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_elongate",
            description: "op_elongate applied to a sphere: stretches along x by h = (0.4, 0, 0).",
            slugs: &["sdf/op_elongate", "sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let q = op_elongate(p, vec3<f32>(0.4, 0.0, 0.0));
    return sphere(q, 0.4);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.55, 0.75, 0.95);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_intersect",
            description: "op_intersect of a sphere (r 0.55) and a box (b 0.45) — lens-shaped intersection.",
            slugs: &["sdf/op_intersect", "sdf/sphere", "sdf/box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let a = sphere(p, 0.6);
    let b = box(p, vec3<f32>(0.45));
    return op_intersect(a, b);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.85, 0.65, 0.95);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_mirror",
            description: "op_mirror reflects negative-x half onto positive-x; applied to an offset sphere.",
            slugs: &["sdf/op_mirror", "sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let q = op_mirror(p, vec3<f32>(1.0, 0.0, 0.0));
    return sphere(q - vec3<f32>(0.4, 0.0, 0.0), 0.35);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.85, 0.55);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_onion",
            description: "op_onion turns a sphere SDF into a thin shell (thickness 0.04). The cone slices the shell open.",
            slugs: &["sdf/op_onion", "sdf/sphere", "sdf/op_subtract", "sdf/box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let shell = op_onion(sphere(p, 0.65), 0.04);
    let cutter = box(p - vec3<f32>(0.5, 0.0, -0.5), vec3<f32>(0.7));
    return op_subtract(shell, cutter);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 100; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d * 0.7;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.65, 0.95, 0.85);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_repeat",
            description: "op_repeat tiles a small sphere (r 0.18) on a 0.55-unit grid.",
            slugs: &["sdf/op_repeat", "sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let q = op_repeat(p, vec3<f32>(0.55, 0.55, 0.55));
    return sphere(q, 0.18);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 96; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.65, 0.40);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_round",
            description: "op_round inflates an octahedron by r = 0.12, creating a softened solid.",
            slugs: &["sdf/op_round", "sdf/octahedron"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return op_round(octahedron(p, 0.45), 0.12);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.55, 0.55);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_smooth_intersect",
            description: "op_smooth_intersect of two offset spheres (r 0.55), blend k = 0.15.",
            slugs: &["sdf/op_smooth_intersect", "sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let a = sphere(p - vec3<f32>(0.25, 0.0, 0.0), 0.55);
    let b = sphere(p + vec3<f32>(0.25, 0.0, 0.0), 0.55);
    return op_smooth_intersect(a, b, 0.15);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.55, 0.65, 0.95);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_smooth_subtract",
            description: "op_smooth_subtract: subtract a sphere (r 0.4) from a box (b 0.5), blend k = 0.1.",
            slugs: &["sdf/op_smooth_subtract", "sdf/sphere", "sdf/box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let a = box(p, vec3<f32>(0.5));
    let b = sphere(p - vec3<f32>(0.0, 0.0, -0.5), 0.4);
    return op_smooth_subtract(a, b, 0.1);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.45, 0.85, 0.65);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_smooth_union",
            description: "op_smooth_union of two spheres (r 0.45) at +-0.3 along x, blend k = 0.2.",
            slugs: &["sdf/op_smooth_union", "sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let a = sphere(p - vec3<f32>(0.3, 0.0, 0.0), 0.45);
    let b = sphere(p + vec3<f32>(0.3, 0.0, 0.0), 0.45);
    return op_smooth_union(a, b, 0.2);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.75, 0.55);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_subtract",
            description: "op_subtract: cut a sphere (r 0.45) out of a box (b 0.55).",
            slugs: &["sdf/op_subtract", "sdf/sphere", "sdf/box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let a = box(p, vec3<f32>(0.55));
    let b = sphere(p - vec3<f32>(0.3, 0.3, -0.3), 0.45);
    return op_subtract(a, b);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.85, 0.45, 0.85);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_twist",
            description: "op_twist applied to a long box: twists around the Y axis with k = 2.0.",
            slugs: &["sdf/op_twist", "sdf/box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let q = op_twist(p, 2.0);
    return box(q, vec3<f32>(0.5, 0.18, 0.18));
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 96; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d * 0.6;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.55, 0.95, 0.65);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "op_union",
            description: "op_union of a sphere (r 0.5) and an offset box (b 0.35) — boolean union.",
            slugs: &["sdf/op_union", "sdf/sphere", "sdf/box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let a = sphere(p - vec3<f32>(0.25, 0.0, 0.0), 0.5);
    let b = box(p + vec3<f32>(0.25, 0.0, 0.0), vec3<f32>(0.35));
    return op_union(a, b);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.45, 0.65);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "plane",
            description: "Tilted plane (normal tilted in +y, +z) bounded by a sphere so the trace terminates.",
            slugs: &["sdf/plane", "sdf/sphere", "sdf/op_intersect"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    let pl = plane(p, vec3<f32>(0.0, 1.0, -0.4), 0.2);
    let bound = sphere(p, 0.95);
    return op_intersect(pl, bound);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 96; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.75, 0.85, 0.95);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "rounded_box",
            description: "Box with half-extents 0.5 and corner radius 0.18.",
            slugs: &["sdf/rounded_box"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return rounded_box(p, vec3<f32>(0.5), 0.18);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.65, 0.45);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "sphere",
            description: "Sphere of radius 0.7 at the origin.",
            slugs: &["sdf/sphere"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return sphere(p, 0.7);
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.95, 0.55, 0.45);
    return vec4<f32>(albedo * lambert, 1.0);
}
"#,
        },
        Composition {
            name: "torus",
            description: "Torus in the XZ-plane, major radius 0.55, minor (tube) radius 0.18.",
            slugs: &["sdf/torus"],
            fragment: r#"
fn scene(p: vec3<f32>) -> f32 {
    return torus(p, vec2<f32>(0.55, 0.18));
}
fn march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 80; i = i + 1) {
        let p = ro + rd * t;
        let d = scene(p);
        if (d < 0.001) { return t; }
        if (t > 8.0) { return -1.0; }
        t = t + d;
    }
    return -1.0;
}
fn nrm(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        scene(p + e.xyy) - scene(p - e.xyy),
        scene(p + e.yxy) - scene(p - e.yxy),
        scene(p + e.yyx) - scene(p - e.yyx),
    ));
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - vec2<f32>(1.0);
    let ro = vec3<f32>(1.4, 1.25, -1.7);
    let ta = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uu * uv.x + vv * uv.y + ww * 1.5);
    let t = march(ro, rd);
    if (t < 0.0) { return vec4<f32>(0.10, 0.12, 0.18, 1.0); }
    let p = ro + rd * t;
    let n = nrm(p);
    let l = normalize(vec3<f32>(0.4, 0.7, -0.6));
    let lambert = 0.2 + 0.8 * max(0.0, dot(n, l));
    let albedo = vec3<f32>(0.55, 0.95, 0.95);
    return vec4<f32>(albedo * lambert, 1.0);
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
        let out_dir = std::path::Path::new("out/gallery_sdf");
        std::fs::create_dir_all(out_dir)?;
        println!("Rendering sdf gallery into {}/", out_dir.display());

        let comps = compositions();
        let total = comps.len();
        let mut ok = 0usize;
        for comp in comps {
            match render_one(&renderer, out_dir, &comp).await {
                Ok(()) => ok += 1,
                Err(e) => eprintln!("  ERR {} -- {}", comp.name, e),
            }
        }
        println!("done: {}/{}", ok, total);
        Ok(())
    })
}
