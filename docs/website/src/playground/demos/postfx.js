// A chain of post effects over a procedural test card, split at the pointer.

export default {
  id: "postfx",
  title: "Post Effects",
  summary: "Curvature, chromatic offsets, posterize, scanlines, vignette and grain over a test card. Left of the pointer is the raw image.",
  kind: "fragment",
  library: [
    "postfx/crt_curvature",
    "postfx/chromatic_offsets",
    "postfx/posterize",
    "postfx/scanlines",
    "postfx/vignette",
    "postfx/film_grain",
  ],
  values: {
    "u.curvature": 0.12,
    "u.steps": 6,
    "u.scanlines": 0.35,
    "u.grain": 0.05,
  },
  frame: { time: "u.time", resolution: "u.resolution", pointer: "u.mouse" },
  files: {
    "shader.wgsl": /* wgsl */ `struct Params {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  time: f32,
  curvature: f32, // 0..0.4
  steps: f32, // 2..32
  scanlines: f32, // 0..1
  grain: f32, // 0..0.4
}

@group(0) @binding(0) var<uniform> u: Params;

// the source image: color bars over a gray ramp, with a sweeping dial
fn card(uv: vec2<f32>) -> vec3<f32> {
  let bar = u32(clamp(uv.x * 7.0, 0.0, 6.0));
  let bars = vec3<f32>(f32(1u - (bar / 2u) % 2u), f32(bar < 4u), f32(1u - bar % 2u)) * 0.8;
  var col = select(vec3<f32>(floor(uv.x * 12.0) / 11.0), bars, uv.y > 0.3);
  let c = (uv - 0.5) * u.resolution / max(min(u.resolution.x, u.resolution.y), 1.0);
  let r = length(c);
  let a = fract(atan2(c.y, c.x) / 6.2831853 - u.time * 0.25);
  let dial = select(vec3<f32>(0.1), vec3<f32>(0.1 + 0.8 * a * a), r < 0.26);
  col = select(col, dial, r < 0.28);
  return select(col, vec3<f32>(0.95), abs(r - 0.28) < 0.006);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let split = u.mouse.x / max(u.resolution.x, 1.0);
  if (in.uv.x < split) {
    return vec4<f32>(card(in.uv), 1.0);
  }

  let crt = crt_curvature(in.uv, u.curvature);
  let rgb = chromatic_offsets(crt.xy, 0.012);
  var col = vec3<f32>(card(rgb[0]).r, card(rgb[1]).g, card(rgb[2]).b);
  col = posterize(col, u.steps);
  col *= scanlines(crt.xy, u.resolution.y * 0.25, u.scanlines);
  col *= vignette(crt.xy, 0.45, 0.35);
  col += film_grain(in.uv, u.time) * u.grain;
  col *= step(0.0, crt.z); // black outside the curved screen
  let edge = 1.0 - smoothstep(0.0, 1.5 / u.resolution.x, in.uv.x - split);
  return vec4<f32>(mix(col, vec3<f32>(1.0), edge), 1.0);
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
