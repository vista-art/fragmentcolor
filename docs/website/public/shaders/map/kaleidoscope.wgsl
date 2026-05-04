// map/kaleidoscope — n-fold rotational symmetry fold around UV center.
fn kaleidoscope(uv: vec2<f32>, n: u32) -> vec2<f32> {
  let c = uv - vec2<f32>(0.5);
  let r = length(c);
  let a = atan2(c.y, c.x);
  let segment = 6.28318530718 / f32(n);
  let folded = abs(a - segment * round(a / segment));
  return vec2<f32>(0.5) + r * vec2<f32>(cos(folded), sin(folded));
}
