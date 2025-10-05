// FragmentColor demo shader: blur-x
// 1D blur horizontally; assumes a 1280x720 target in UV math.

@group(0) @binding(0)
var samp: sampler;
@group(0) @binding(1)
var src: texture_2d<f32>;

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VSOut {
  var out: VSOut;
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0),
  );
  let p = positions[vi];
  out.pos = vec4<f32>(p, 0.0, 1.0);
  out.uv = p * 0.5 + vec2<f32>(0.5, 0.5);
  return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
  let uv = in.uv;
  let dx = 1.0 / 1280.0;
  var sum = vec4<f32>(0.0);
  sum += textureSample(src, samp, uv + vec2<f32>(-2.0*dx, 0.0)) * 0.12;
  sum += textureSample(src, samp, uv + vec2<f32>(-1.0*dx, 0.0)) * 0.22;
  sum += textureSample(src, samp, uv)                         * 0.32;
  sum += textureSample(src, samp, uv + vec2<f32>( 1.0*dx, 0.0)) * 0.22;
  sum += textureSample(src, samp, uv + vec2<f32>( 2.0*dx, 0.0)) * 0.12;
  return vec4<f32>(sum.rgb, 1.0);
}
