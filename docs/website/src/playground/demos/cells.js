// Worley cells, warped by fractal noise and colored by a cosine palette.

export default {
  id: "cells",
  title: "Cells",
  summary: "Worley noise warped by fractal noise. The pointer magnifies the cells under it.",
  kind: "fragment",
  library: ["noise/worley2", "noise/fbm2", "gradient/palette_iq"],
  values: {
    "u.scale": 7,
    "u.warp": 1.4,
    "u.edge": 0.08,
    "u.edge_color": [0.05, 0.03, 0.08],
  },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: {
    "shader.wgsl": /* wgsl */ `struct Params {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  time: f32,
  scale: f32, // 2..24
  warp: f32, // 0..3
  edge: f32, // 0.01..0.4
  edge_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Params;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let aspect = u.resolution.x / max(u.resolution.y, 1.0);
  let mouse = u.mouse / max(u.resolution, vec2<f32>(1.0));
  var p = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);
  let m = (vec2<f32>(mouse.x, 1.0 - mouse.y) - 0.5) * vec2<f32>(aspect, 1.0);

  // magnify the cells under the pointer
  let d = p - m;
  p = m + d * (1.0 - 0.45 * exp(-dot(d, d) * 14.0));

  p = p * u.scale;
  let t = u.time * 0.15;
  let w = vec2<f32>(fbm2(p * 0.25 + t, 4u), fbm2(p * 0.25 - t + 5.2, 4u));
  let f = worley2(p + (w - 0.5) * u.warp);

  let shade = palette_iq(
    f.x * 0.8 + t,
    vec3<f32>(0.55, 0.5, 0.48),
    vec3<f32>(0.3, 0.25, 0.25),
    vec3<f32>(1.0),
    vec3<f32>(0.0, 0.1, 0.2),
  ) * (1.0 - 0.6 * f.x);
  let line = 1.0 - smoothstep(0.0, u.edge, f.y - f.x);
  return vec4<f32>(mix(shade, u.edge_color, line), 1.0);
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
