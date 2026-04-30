// linear_srgb_to_oklab — linear-light sRGB → Björn Ottosson's OkLab.
fn linear_srgb_to_oklab(c: vec3<f32>) -> vec3<f32> {
  let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
  let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
  let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
  let l_ = pow(l, 1.0 / 3.0);
  let m_ = pow(m, 1.0 / 3.0);
  let s_ = pow(s, 1.0 / 3.0);
  return vec3<f32>(
    0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
    1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
    0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
  );
}
