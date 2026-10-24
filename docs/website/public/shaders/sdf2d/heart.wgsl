// sdf2d/heart — signed distance to a heart shape. Lobes sit at positive y,
// apex (the pointy bottom) at origin. Scale by `s`. For UV space (y-down),
// flip p.y at the call site: `heart(vec2<f32>(p.x, -p.y), s)`.
//
// Exact distance, after Inigo Quilez's 2D heart
// (https://iquilezles.org/articles/distfunctions2d/).
fn heart(p: vec2<f32>, s: f32) -> f32 {
  var q = p / s;
  q.x = abs(q.x);

  // Upper lobe: distance to a circle of radius sqrt(2)/4 at (0.25, 0.75).
  let lobe = length(q - vec2<f32>(0.25, 0.75)) - sqrt(2.0) / 4.0;

  // Body: distance to the top dip (0, 1) or to the side, the segment on
  // y = x from the apex to (0.5, 0.5), signed by which side of y = x we are on.
  let to_top = q - vec2<f32>(0.0, 1.0);
  let to_side = q - vec2<f32>(0.5 * max(q.x + q.y, 0.0));
  let body = sqrt(min(dot(to_top, to_top), dot(to_side, to_side))) * sign(q.x - q.y);

  // Branchless region select: above the line x + y = 1 -> lobe, below -> body.
  return s * select(body, lobe, q.x + q.y > 1.0);
}
