// sdf2d/heart — signed distance to a heart shape. Lobes sit at positive y,
// apex (the pointy bottom) at origin. Scale by `s`. For UV space (y-down),
// flip p.y at the call site: `heart(vec2<f32>(p.x, -p.y), s)`.
//
// Implementation: Inigo Quilez's 2D heart formula
// (https://iquilezles.org/articles/distfunctions2d/) plus a clamp that
// fixes the formula's only flaw: it treats the wedge diagonal y = -x as
// extending infinitely below the apex, leaving phantom zero-crossings
// down-and-out from the heart. We clamp the result to the radial distance
// from the apex (origin) when we're below it, which gives a real lower
// bound and erases the phantom artifact.
fn heart(p: vec2<f32>, s: f32) -> f32 {
  var q = p / s;
  q.x = abs(q.x);

  // Upper lobe: distance to a circle of radius sqrt(2)/4 at (0.25, 0.75).
  let lobe = length(q - vec2<f32>(0.25, 0.75)) - sqrt(2.0) / 4.0;

  // Body: unsigned distance to either the top dip (0, 1) or the wedge
  // diagonal y = -x, signed by which side of y = x we are on.
  let to_top_sq = dot(q - vec2<f32>(0.0, 1.0), q - vec2<f32>(0.0, 1.0));
  let to_diag_sq = 0.5 * (q.x + q.y) * (q.x + q.y);
  let body = sqrt(min(to_top_sq, to_diag_sq)) * sign(q.x - q.y);

  // Clamp below the apex: when q.y < 0, the closest point on the heart
  // is the apex (origin), so the true distance is at least length(q).
  let below = step(q.y, 0.0);
  let clamped = mix(body, max(body, length(q)), below);

  // Branchless region select: above the diagonal -> lobe, below -> body.
  let in_lobe = step(1.0, q.x + q.y);
  return s * mix(clamped, lobe, in_lobe);
}
