// kelvin_to_rgb — approximate blackbody color temperature → linear RGB (1000..40000 K).
// Tanner Helland's piecewise approximation, adapted.
fn kelvin_to_rgb(k: f32) -> vec3<f32> {
  let t = clamp(k, 1000.0, 40000.0) / 100.0;
  var r = 1.0; var g = 1.0; var b = 1.0;
  if (t <= 66.0) {
    g = clamp(0.39008157876901960784 * log(t) - 0.63184144378862745098, 0.0, 1.0);
    if (t <= 19.0) { b = 0.0; }
    else { b = clamp(0.54320678911019607843 * log(t - 10.0) - 1.19625408914, 0.0, 1.0); }
  } else {
    r = clamp(1.29293618606274509804 * pow(t - 60.0, -0.1332047592), 0.0, 1.0);
    g = clamp(1.12989086089529411765 * pow(t - 60.0, -0.0755148492), 0.0, 1.0);
  }
  return vec3<f32>(r, g, b);
}
