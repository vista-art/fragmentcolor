// Domain mapping: a Truchet pattern seen through a kaleidoscope and a twirl.

export default {
  id: "kaleidoscope",
  title: "Kaleidoscope",
  summary: "Map functions bend the coordinates before any pattern is drawn. The pointer sets the twirl center, repeated in every mirror segment.",
  category: "Shaders",
  thumbnail: "/playground/thumbs/kaleidoscope.webp",
  kind: "fragment",
  library: ["map/kaleidoscope", "map/twirl", "map/rotate2", "pattern/truchet", "gradient/palette_iq"],
  values: {
    "u.segments": 8,
    "u.twist": 2.5,
    "u.scale": 6,
    "u.line_color": [0.96, 0.94, 0.88],
  },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: {
    "shader.wgsl": /* wgsl */ `struct Params {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  time: f32,
  segments: f32, // 2..16
  twist: f32, // -8..8
  scale: f32, // 2..16
  line_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Params;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let fit = u.resolution / max(min(u.resolution.x, u.resolution.y), 1.0);
  let mouse = u.mouse / max(u.resolution, vec2<f32>(1.0));
  // square coordinates centered on (0.5, 0.5), the center the map functions expect
  let uv = (in.uv - 0.5) * fit + 0.5;
  let m = (vec2<f32>(mouse.x, 1.0 - mouse.y) - 0.5) * fit + 0.5;
  let n = u32(max(u.segments, 2.0));

  // fold the pixel and the pointer the same way, then twirl around the folded pointer
  let spin = u.time * 0.1;
  var k = kaleidoscope(rotate2(uv - 0.5, spin) + 0.5, n);
  k = twirl(k, kaleidoscope(rotate2(m - 0.5, spin) + 0.5, n), u.twist * sin(u.time * 0.4));

  let d = truchet(k + vec2<f32>(u.time * 0.03, 0.0), u.scale);
  let r = length(uv - 0.5);
  let base = palette_iq(
    r * 1.2 - u.time * 0.05,
    vec3<f32>(0.32, 0.34, 0.42),
    vec3<f32>(0.18, 0.16, 0.2),
    vec3<f32>(1.0),
    vec3<f32>(0.0, 0.1, 0.2),
  );
  let px = u.scale / max(min(u.resolution.x, u.resolution.y), 1.0);
  let line = 1.0 - smoothstep(0.06, 0.06 + px * 1.5, d);
  let col = mix(base * (0.6 + 0.4 * smoothstep(0.0, 0.3, d)), u.line_color, line);
  return vec4<f32>(col * (1.0 - 0.5 * r * r), 1.0);
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
