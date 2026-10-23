// Shared WGSL prelude for every artwork: a fullscreen triangle vertex stage
// and one uniform block fed each frame. Library functions are prepended by
// slug from each artwork's `slugs`.

export const PRELUDE = /* wgsl */ `
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

struct Globals {
  resolution: vec2<f32>,
  mouse: vec2<f32>, // pointer in canvas pixels, y down
  drag: vec2<f32>, // -3..3
  time: f32,
  zoom: f32, // 0.5..2.4
  press: f32, // 0..1
  pulse: f32, // 0..1.4
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

// The pointer in UV space: 0 to 1 across, y up.
fn pointer_uv() -> vec2<f32> {
  return vec2<f32>(u.mouse.x, u.resolution.y - u.mouse.y) / max(u.resolution, vec2<f32>(1.0));
}

// Centered coordinates, aspect corrected, y up, zoom applied.
fn centered(uv: vec2<f32>) -> vec2<f32> {
  let aspect = u.resolution.x / max(u.resolution.y, 1.0);
  return (uv * 2.0 - vec2<f32>(1.0)) * vec2<f32>(aspect, 1.0) / u.zoom;
}
`;
