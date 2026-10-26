// 2D signed distance fields, merged with a smooth union and lit by their own distance.

export default {
  id: "shapes",
  title: "Shapes",
  summary:
    "A circle, a star and a hexagon, the last two from Inigo Quilez's 2D distance functions, merged with his smooth union. The distance lights the scene: an azure glow and drifting rings outside, violet to magenta inside. The circle follows the pointer.",
  category: "Shaders",
  thumbnail: "/playground/thumbs/shapes.webp",
  chip: { x: 0.26, y: 0.5, zoom: 1.6 },
  kind: "fragment",
  library: ["sdf2d/star", "sdf2d/hexagon", "sdf2d/circle", "sdf/op_smooth_union", "map/rotate2"],
  values: {
    "u.blend": 0.12,
    "u.rings": 20,
    "u.glow": 10,
    "u.inside_color": [1, 0, 0.514],
    "u.rim_color": [0.514, 0, 1],
    "u.outside_color": [0, 0.514, 1],
  },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: {
    "shader.wgsl": /* wgsl */ `// The smooth union follows Inigo Quilez: https://iquilezles.org/articles/smin/
// The star and hexagon distances follow him too: https://iquilezles.org/articles/distfunctions2d/

struct Params {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  time: f32,
  blend: f32, // 0..0.4
  rings: f32, // 5..60
  glow: f32, // 2..30
  inside_color: vec3<f32>,
  rim_color: vec3<f32>,
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

  // glow and drifting rings outside, violet to magenta inside, a white edge
  let outside = max(d, 0.0);
  let px = fwidth(d);
  var col = vec3<f32>(0.05, 0.06, 0.08) + u.outside_color * 0.45 * exp(-u.glow * outside);
  let ring = abs(fract(d * u.rings - t * 0.5 + 0.5) - 0.5) / (u.rings * px);
  col = col + u.outside_color * 0.35 * exp(-3.0 * outside) * (1.0 - smoothstep(0.5, 1.5, ring));
  col = select(col, mix(u.rim_color, u.inside_color, smoothstep(0.0, 0.12, -d)), d < 0.0);
  col = mix(col, vec3<f32>(1.0), 1.0 - smoothstep(px * 0.5, px * 1.5, abs(d)));
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
