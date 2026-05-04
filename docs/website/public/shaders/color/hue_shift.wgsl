// hue_shift — rotate hue of a linear-RGB color by `radians` via YIQ-style rotation.
fn hue_shift(c: vec3<f32>, radians: f32) -> vec3<f32> {
  let cosA = cos(radians);
  let sinA = sin(radians);
  let k = 1.0 / 3.0;
  let sqrt3 = 1.732050808;
  let m = mat3x3<f32>(
    vec3<f32>(cosA + (1.0 - cosA) * k, k * (1.0 - cosA) - sqrt3 * k * sinA, k * (1.0 - cosA) + sqrt3 * k * sinA),
    vec3<f32>(k * (1.0 - cosA) + sqrt3 * k * sinA, cosA + k * (1.0 - cosA), k * (1.0 - cosA) - sqrt3 * k * sinA),
    vec3<f32>(k * (1.0 - cosA) - sqrt3 * k * sinA, k * (1.0 - cosA) + sqrt3 * k * sinA, cosA + k * (1.0 - cosA))
  );
  return m * c;
}
