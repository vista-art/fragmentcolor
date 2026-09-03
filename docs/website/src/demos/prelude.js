// Shared WGSL prelude for every artwork: a fullscreen triangle vertex stage
// and one uniform block the engine feeds each frame. Catalog helpers from
// fragmentcolor.org/shaders are prepended by slug in each artwork's `parts`.

export const PRELUDE = /* wgsl */ `
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

struct Globals {
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  drag: vec2<f32>,
  time: f32,
  zoom: f32,
  press: f32,
  pulse: f32,
  // pads the block to 48 bytes; WebGL2 rejects uniform blocks that are not a multiple of 16
  pad0: f32,
  pad1: f32,
}

@group(0) @binding(0) var<uniform> u: Globals;

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

// Centered coordinates, aspect corrected, y up, zoom applied.
fn centered(uv: vec2<f32>) -> vec2<f32> {
  let aspect = u.resolution.x / max(u.resolution.y, 1.0);
  return (uv * 2.0 - vec2<f32>(1.0)) * vec2<f32>(aspect, 1.0) / u.zoom;
}
`;
