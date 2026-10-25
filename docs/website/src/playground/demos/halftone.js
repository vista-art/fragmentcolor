// A lit sphere printed as a halftone: dots on a rotated grid, sized by the local tone.

export default {
  id: "halftone",
  title: "Halftone",
  summary: "The dots pattern on a rotated grid, each dot sized by the tone of the image under it. The light circles the sphere and follows the pointer.",
  category: "Shaders",
  thumbnail: "/playground/thumbs/halftone.webp",
  kind: "fragment",
  library: ["pattern/dots", "map/rotate2", "lighting/lambert"],
  values: {
    "u.cells": 60,
    "u.angle": 0.785,
    "u.ink_color": [0.13, 0.15, 0.24],
    "u.paper_color": [0.96, 0.94, 0.89],
  },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: {
    "shader.wgsl": /* wgsl */ `struct Params {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  time: f32,
  cells: f32, // 20..160
  angle: f32, // 0..1.57
  ink_color: vec3<f32>,
  paper_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Params;

// ink coverage of the source image at p: a lit sphere over moving waves
fn tone(p: vec2<f32>, light: vec3<f32>) -> f32 {
  let center = vec2<f32>(0.0, 0.03 * sin(u.time));
  let q = (p - center) / 0.34;
  let h = 1.0 - dot(q, q);
  let waves = 0.18 + 0.1 * sin(p.x * 9.0 + p.y * 4.0 + u.time);
  return select(waves, 1.0 - lambert(vec3<f32>(q, sqrt(max(h, 0.0))), light), h > 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let fit = u.resolution / max(min(u.resolution.x, u.resolution.y), 1.0);
  let mouse = u.mouse / max(u.resolution, vec2<f32>(1.0));
  let p = (in.uv - 0.5) * fit;
  let m = (vec2<f32>(mouse.x, 1.0 - mouse.y) - 0.5) * fit;
  let orbit = 0.6 * vec2<f32>(cos(u.time * 0.5), sin(u.time * 0.5));
  let light = vec3<f32>(m * 2.5 + orbit, 0.8);

  // dot area grows with the ink; dots() takes its radius in cell units
  let g = rotate2(p, u.angle);
  let ink = tone(p, light);
  let fw = u.cells / max(min(u.resolution.x, u.resolution.y), 1.0);
  let mark = dots(g, u.cells, 0.72 * sqrt(clamp(ink, 0.0, 1.0)), fw);
  return vec4<f32>(mix(u.paper_color, u.ink_color, mark), 1.0);
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
