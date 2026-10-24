// 2D signed distance fields, merged with a smooth union and drawn as contours.

export default {
  id: "shapes",
  title: "Shapes",
  summary:
    "A star and a hexagon from Inigo Quilez's 2D distance functions and a circle, merged with his smooth union and drawn with his contour coloring. The circle follows the pointer.",
  kind: "fragment",
  library: ["sdf2d/star", "sdf2d/hexagon", "sdf2d/circle", "sdf/op_smooth_union", "map/rotate2"],
  values: {
    "u.blend": 0.12,
    "u.bands": 40,
    "u.inside_color": [0.38, 0.62, 0.92],
    "u.outside_color": [0.92, 0.6, 0.32],
  },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: {
    "shader.wgsl": /* wgsl */ `// The star and hexagon distances, the smooth union and the contour coloring
// follow Inigo Quilez: https://iquilezles.org/articles/distfunctions2d/

struct Params {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  time: f32,
  blend: f32, // 0..0.4
  bands: f32, // 5..120
  inside_color: vec3<f32>,
  outside_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Params;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let fit = u.resolution / max(min(u.resolution.x, u.resolution.y), 1.0);
  let mouse = u.mouse / max(u.resolution, vec2<f32>(1.0));
  let p = (in.uv - 0.5) * fit;
  let m = (vec2<f32>(mouse.x, 1.0 - mouse.y) - 0.5) * fit;
  let t = u.time;

  // spin the star, bob the hexagon, and park a circle under the pointer
  let d_star = star(rotate2(p - vec2<f32>(-0.3, 0.0), t * 0.4), 0.2, 5u, 3.0);
  let d_hex = hexagon(p - vec2<f32>(0.3, 0.05 * sin(t)), 0.15);
  let d_dot = circle(p - m, 0.08);
  let d = op_smooth_union(op_smooth_union(d_star, d_hex, u.blend), d_dot, u.blend);

  // tint by sign, darken near the surface, ripple the contours outward
  var col = select(u.outside_color, u.inside_color, d < 0.0);
  col = col * (1.0 - 0.75 * exp(-8.0 * abs(d)));
  col = col * (0.85 + 0.15 * cos(6.2831853 * (u.bands * d - t * 0.5)));
  let px = fwidth(d);
  col = mix(col, vec3<f32>(1.0), 1.0 - smoothstep(px, px * 2.0, abs(d)));
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
