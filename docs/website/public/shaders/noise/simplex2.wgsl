// simplex2 — 2D simplex noise in roughly [-1, 1]. Ashima-style implementation.
fn _sn2_mod289_v2(x: vec2<f32>) -> vec2<f32> { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn _sn2_mod289_v3(x: vec3<f32>) -> vec3<f32> { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn _sn2_permute(x: vec3<f32>) -> vec3<f32> { return _sn2_mod289_v3(((x * 34.0) + 1.0) * x); }

fn simplex2(v: vec2<f32>) -> f32 {
  let C = vec4<f32>(0.211324865405187, 0.366025403784439,
                    -0.577350269189626, 0.024390243902439);
  var i  = floor(v + dot(v, C.yy));
  let x0 = v - i + dot(i, C.xx);
  var i1 = vec2<f32>(0.0);
  if (x0.x > x0.y) { i1 = vec2<f32>(1.0, 0.0); } else { i1 = vec2<f32>(0.0, 1.0); }
  let x12 = vec4<f32>(x0.x, x0.y, x0.x, x0.y) + vec4<f32>(C.x, C.x, C.z, C.z)
            - vec4<f32>(i1.x, i1.y, 0.0, 0.0);
  i = _sn2_mod289_v2(i);
  let p = _sn2_permute(_sn2_permute(i.y + vec3<f32>(0.0, i1.y, 1.0)) + i.x + vec3<f32>(0.0, i1.x, 1.0));
  var m = max(vec3<f32>(0.5) - vec3<f32>(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3<f32>(0.0));
  m = m * m; m = m * m;
  let x = 2.0 * fract(p * C.www) - 1.0;
  let h = abs(x) - 0.5;
  let ox = floor(x + 0.5);
  let a0 = x - ox;
  m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));
  let g = vec3<f32>(a0.x * x0.x + h.x * x0.y,
                    a0.y * x12.x + h.y * x12.y,
                    a0.z * x12.z + h.z * x12.w);
  return 130.0 * dot(m, g);
}
