// Fullscreen swirl palette demo (shader-only)
// Ported from a Shadertoy-style fragment to WGSL

struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

const TAU: f32 = 6.283185307179586;

fn pal(t: f32, brightness: vec3<f32>, contrast: vec3<f32>, oscillation: vec3<f32>, phase: vec3<f32>) -> vec3<f32> {
  return brightness + contrast * cos(TAU * (oscillation * t + phase));
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0)
  );
  return VOut(vec4<f32>(p[i], 0.0, 1.0));
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  // Pixel coords -> normalized 0..1
  let frag = v.pos.xy;
  let normCoord = frag / resolution;

  // Map to -1..1 and fix aspect on x
  var uv = -1.0 + 2.0 * normCoord;
  uv.x = uv.x * (resolution.x / resolution.y);

  let slowTime = time * 0.25; // slower motion like iTime/4

  // Basic patterns
  let radius = length(uv);
  let rings = sin(slowTime - radius * 15.0);
  let angle = atan2(uv.y, uv.x);
  let radar = sin(angle + slowTime);
  let swirl = sin(rings + radar + slowTime);

  // Palette parameters (animated)
  let brightnessBlend = 0.5 * (sin(time + length(uv * 20.0)) + 1.0);
  let contrastBlend = 0.5 * (sin(slowTime) + 1.0);
  let brightness = vec3<f32>(0.1 + brightnessBlend * (0.7 - 0.1));
  let contrast   = vec3<f32>(0.2 + contrastBlend * (0.5 - 0.2));
  let oscillation = vec3<f32>(0.4, 0.5 * (sin(slowTime) + 1.0), 0.2);
  let phase = vec3<f32>(0.7, 0.4, 0.1);

  let color = pal(swirl, brightness, contrast, oscillation, phase);
  return vec4<f32>(color, 1.0);
}
