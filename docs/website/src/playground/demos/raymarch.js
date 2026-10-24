// A sphere-traced scene built from 3D distance functions and lit with library terms.

export default {
  id: "raymarch",
  title: "Raymarch",
  summary: "A twisted box melts into a torus through a smooth union, sphere traced and lit with Blinn-Phong and rim light. Move the pointer to orbit.",
  kind: "fragment",
  library: [
    "sdf/rounded_box",
    "sdf/torus",
    "sdf/op_twist",
    "sdf/op_smooth_union",
    "raymarch/normal_from_sdf_tetra",
    "lighting/half_lambert",
    "lighting/blinn_phong",
    "lighting/rim",
  ],
  values: {
    "u.blend": 0.3,
    "u.twist": 1.2,
    "u.shine": 48,
    "u.surface_color": [0.85, 0.42, 0.3],
  },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: {
    "shader.wgsl": /* wgsl */ `struct Params {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  time: f32,
  blend: f32, // 0.01..0.8
  twist: f32, // 0..3
  shine: f32, // 4..128
  surface_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Params;

fn scene(p: vec3<f32>) -> f32 {
  let body = rounded_box(op_twist(p, u.twist * sin(u.time * 0.5)), vec3<f32>(0.45, 0.7, 0.45), 0.08);
  let ring = torus(p - vec3<f32>(0.0, 0.55 * sin(u.time * 0.8), 0.0), vec2<f32>(0.85, 0.14));
  return op_smooth_union(body, ring, u.blend);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let fit = u.resolution / max(min(u.resolution.x, u.resolution.y), 1.0);
  let mouse = u.mouse / max(u.resolution, vec2<f32>(1.0));
  let p = (in.uv - 0.5) * fit;

  // orbit camera: time and pointer x turn it, pointer y raises it
  let yaw = u.time * 0.2 + (mouse.x - 0.5) * 6.2831853;
  let ro = vec3<f32>(sin(yaw), 1.2 - mouse.y * 1.6, cos(yaw)) * 3.4;
  let fwd = normalize(-ro);
  let right = normalize(cross(fwd, vec3<f32>(0.0, 1.0, 0.0)));
  let rd = normalize(fwd * 1.6 + right * p.x + cross(right, fwd) * p.y);

  var t = 0.0;
  for (var i = 0; i < 128; i++) {
    let d = scene(ro + rd * t);
    if (d < 0.001 || t > 12.0) { break; }
    t += d * 0.8; // short steps: the twist bends distances
  }

  var col = vec3<f32>(0.09, 0.1, 0.12) + 0.06 * in.uv.y;
  if (t < 12.0) {
    let pos = ro + rd * t;
    let e = vec2<f32>(0.001, -0.001);
    let n = normal_from_sdf_tetra(scene(pos + e.xyy), scene(pos + e.yyx), scene(pos + e.yxy), scene(pos + e.xxx));
    let light = normalize(vec3<f32>(0.6, 0.8, 0.4));
    col = u.surface_color * (0.15 + 0.85 * half_lambert(n, light));
    col += vec3<f32>(blinn_phong(n, light, -rd, u.shine)) * 0.6;
    col += vec3<f32>(0.5, 0.65, 0.8) * rim(n, -rd, 3.0) * 0.4;
  }
  return vec4<f32>(col, 1.0);
}

// Fullscreen triangle: three vertices that cover the canvas.
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  var corners = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
  );
  var out: VertexOutput;
  out.position = vec4<f32>(corners[i], 0.0, 1.0);
  out.uv = corners[i] * 0.5 + vec2<f32>(0.5);
  return out;
}
`,
  },
};
