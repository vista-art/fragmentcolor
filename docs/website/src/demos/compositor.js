// Final pass: samples the offscreen artwork target(s), crossfades between
// rooms, and lays film grain, chromatic aberration, and a vignette on top.

export const COMPOSITOR_SLUGS = [
  "postfx/chromatic_offsets",
  "postfx/film_grain",
  "postfx/vignette",
  "easing/in_out_cubic",
];

export const COMPOSITOR_SOURCE = /* wgsl */ `
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

struct CompGlobals {
  resolution: vec2<f32>,
  time: f32,
  fade: f32,
  aberration: f32,
  grain: f32,
  // pads the block to 32 bytes for WebGL2
  pad0: f32,
  pad1: f32,
}

@group(0) @binding(0) var<uniform> u: CompGlobals;
@group(0) @binding(1) var tex_from: texture_2d<f32>;
@group(0) @binding(2) var tex_to: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  var corners = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0)
  );
  var out: VertexOutput;
  out.position = vec4<f32>(corners[i], 0.0, 1.0);
  out.uv = corners[i] * 0.5 + vec2<f32>(0.5);
  return out;
}

fn to_tex(uv: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(uv.x, 1.0 - uv.y);
}

fn zoom_at_center(uv: vec2<f32>, z: f32) -> vec2<f32> {
  return (uv - vec2<f32>(0.5)) / z + vec2<f32>(0.5);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let fade = in_out_cubic(clamp(u.fade, 0.0, 1.0));
  let spike = sin(3.14159265 * fade);
  let strength = u.aberration * (1.0 + 7.0 * spike);

  // outgoing room pushes in, incoming room settles from slightly above scale
  let uv_from = zoom_at_center(in.uv, 1.0 + 0.08 * fade);
  let uv_to = zoom_at_center(in.uv, 1.06 - 0.06 * fade);

  let o_from = chromatic_offsets(uv_from, strength);
  let col_from = vec3<f32>(
    textureSample(tex_from, samp, to_tex(o_from[0])).r,
    textureSample(tex_from, samp, to_tex(o_from[1])).g,
    textureSample(tex_from, samp, to_tex(o_from[2])).b
  );

  let o_to = chromatic_offsets(uv_to, strength);
  let col_to = vec3<f32>(
    textureSample(tex_to, samp, to_tex(o_to[0])).r,
    textureSample(tex_to, samp, to_tex(o_to[1])).g,
    textureSample(tex_to, samp, to_tex(o_to[2])).b
  );

  var col = mix(col_from, col_to, fade);

  // a breath of light at the crossing point
  col = col * (1.0 + 0.22 * spike);

  col = col + vec3<f32>(film_grain(in.uv, fract(u.time) * 61.0)) * u.grain;
  col = col * vignette(in.uv, 0.55, 0.62);

  return vec4<f32>(col, 1.0);
}
`;
