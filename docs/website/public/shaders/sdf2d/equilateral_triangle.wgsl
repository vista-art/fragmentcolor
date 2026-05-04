// sdf2d/equilateral_triangle — centered, pointing +y, circumradius `r`.
fn equilateral_triangle(p: vec2<f32>, r: f32) -> f32 {
  let k = sqrt(3.0);
  var q = vec2<f32>(abs(p.x) - r, p.y + r / k);
  if (q.x + k * q.y > 0.0) {
    q = vec2<f32>(q.x - k * q.y, -k * q.x - q.y) * 0.5;
  }
  q.x = q.x - clamp(q.x, -2.0 * r, 0.0);
  return -length(q) * sign(q.y);
}
