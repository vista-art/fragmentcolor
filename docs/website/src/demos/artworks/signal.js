export const signal = {
  id: "signal",
  title: "Signal",
  caption:
    "A late broadcast from nowhere, curving across old glass. It degrades beautifully when touched.",
  hint: "move to tune · click to glitch · hold for interference",
  slugs: [
    "postfx/crt_curvature",
    "postfx/scanlines",
    "postfx/film_grain",
    "noise/fbm2",
    "gradient/palette_iq",
    "hash/hash12",
  ],
  source: /* wgsl */ `
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let curved = crt_curvature(in.uv, 0.18);
  if (curved.z < 0.0) {
    // bezel, catching a little phosphor spill
    let spill = exp(curved.z * 26.0) * 0.05;
    return vec4<f32>(vec3<f32>(0.012, 0.012, 0.016) + vec3<f32>(0.3, 0.5, 0.6) * spill, 1.0);
  }
  var suv = curved.xy;
  let t = u.time;

  // horizontal tearing, worse while pressed or right after a click
  let unrest = clamp(0.22 * u.press + 1.0 * u.pulse, 0.0, 1.0);
  let band = floor(suv.y * 26.0);
  let gate = step(0.55, hash12(vec2<f32>(band, floor(t * 7.0))));
  suv.x = suv.x + (hash12(vec2<f32>(band, floor(t * 13.0))) - 0.5) * 0.38 * unrest * gate;

  let aspect = u.resolution.x / max(u.resolution.y, 1.0);
  let p = (suv * 2.0 - vec2<f32>(1.0)) * vec2<f32>(aspect, 1.0) / u.zoom;
  // warp the cursor through the same lens so ripples ring its apparent spot
  let mw = crt_curvature(u.mouse, 0.18).xy;
  let m = (mw * 2.0 - vec2<f32>(1.0)) * vec2<f32>(aspect, 1.0) / u.zoom;

  // plasma carrier plus interference rippling out from the cursor
  let dm = length(p - m);
  var v = sin(p.x * 3.0 + t * 1.1);
  v = v + sin(p.y * 4.0 + t * 0.7);
  v = v + sin((p.x + p.y) * 2.5 + t * 1.7);
  v = v + sin(sqrt(p.x * p.x * 9.0 + p.y * p.y * 4.0 + 1.0) + t * 2.3);
  v = v + 1.9 * sin(dm * 9.0 - t * 3.5) * exp(-dm * 1.3) * (0.55 + 0.45 * u.press);
  v = v + fbm2(p * 2.0 + vec2<f32>(t * 0.15, -t * 0.1), 4u) * 2.0 - 1.0;
  v = v * 0.22;

  // phosphor interference: thin glowing contour lines over a dark tube
  let bands = pow(abs(sin(v * 16.0 - t * 0.9)), 8.0);
  let fringe = pow(abs(sin(v * 16.0 - t * 0.9 + 0.4)), 28.0);
  let sel = sin(v * 3.1 + t * 0.35);
  let wa = pow(max(sel, 0.0), 1.4);
  let wc = pow(max(-sel, 0.0), 1.4);
  let amber = vec3<f32>(1.00, 0.58, 0.12);
  let cyan = vec3<f32>(0.16, 0.80, 0.85);
  var col = (amber * wa + cyan * wc + vec3<f32>(0.30, 0.28, 0.24) * 0.5) * (0.05 + 1.35 * bands);
  col = col + (amber * wc + cyan * wa) * fringe * 0.45;
  col = col + palette_iq(
    v * 0.25 + 0.5,
    vec3<f32>(0.10, 0.09, 0.13),
    vec3<f32>(0.08, 0.07, 0.11),
    vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(0.10, 0.40, 0.70)
  ) * 0.4;

  // aperture grille and scanlines
  let px = suv.x * u.resolution.x;
  col = col * (vec3<f32>(0.90) + vec3<f32>(0.10) * vec3<f32>(
    0.5 + 0.5 * sin(px * 1.047),
    0.5 + 0.5 * sin(px * 1.047 + 2.094),
    0.5 + 0.5 * sin(px * 1.047 + 4.188)
  ));
  col = col * scanlines(suv, u.resolution.y * 0.22, 0.16);

  // rolling brightness bar drifting up the tube
  let roll = fract(suv.y + t * 0.045);
  col = col * (0.90 + 0.10 * exp(-pow((roll - 0.5) * 6.0, 2.0)));

  // snow
  col = col + vec3<f32>(film_grain(suv, fract(t) * 43.0)) * (0.05 + 0.30 * unrest);

  // tube shading toward the corners
  col = col * (0.55 + 0.45 * smoothstep(0.0, 0.25, curved.z));
  col = col * smoothstep(0.0, 0.012, curved.z);

  return vec4<f32>(col, 1.0);
}
`,
};
