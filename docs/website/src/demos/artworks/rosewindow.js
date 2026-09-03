export const rosewindow = {
  id: "rosewindow",
  title: "Rose Window",
  caption:
    "A pane of living stained glass. Horizontal motion refolds the symmetry; vertical motion twists the light.",
  hint: "move to refold the glass · hold to bloom · scroll to zoom",
  slugs: [
    "map/kaleidoscope",
    "map/twirl",
    "noise/fbm2",
    "gradient/palette_iq",
    "color/tonemap_aces",
  ],
  source: /* wgsl */ `
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // the epsilon keeps the exact center pixel off the atan2(0, 0) singularity
  let c = centered(in.uv) * 0.72 + vec2<f32>(1e-4, 0.0);
  let t = u.time * 0.1;

  // fold count follows the cursor's x, always an even count
  let seg = max(6.0 + floor(clamp(u.mouse.x, 0.0, 1.0) * 7.0) * 2.0, 2.0);
  var k = kaleidoscope(c + vec2<f32>(0.5), u32(seg));
  let twist = (u.mouse.y - 0.5) * 3.2 + 0.35 * sin(t * 0.9);
  k = twirl(k, vec2<f32>(0.5), twist);

  let p = (k - vec2<f32>(0.5)) * 2.0;
  let radius = length(p);
  let angle = atan2(p.y, p.x);

  let marble = fbm2(p * 3.0 + vec2<f32>(t, -t * 0.7), 5u);
  let rings = sin(radius * (16.0 + 5.0 * sin(t * 0.6)) - t * 2.6 + marble * 4.5);
  let petals = sin(angle * seg * 0.5 + marble * 3.0 + t * 0.8);
  let field = smoothstep(-0.25, 0.95, rings * 0.5 + 0.5) * (0.55 + 0.45 * petals);

  var col = palette_iq(
    field * 0.85 + radius * 0.55 - t * 0.45,
    vec3<f32>(0.50, 0.42, 0.42),
    vec3<f32>(0.48, 0.44, 0.42),
    vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(0.00, 0.15, 0.35)
  );
  col = col * (0.30 + 1.05 * field);

  // dark lead seams between the panes
  let seam = smoothstep(0.0, 0.10, abs(rings) * (0.35 + 0.65 * abs(petals)));
  col = col * (0.18 + 0.82 * seam);

  // sunlight through the center, swelling while held
  let bloom = 0.10 + 0.45 * u.press + 0.75 * u.pulse;
  col = col + vec3<f32>(1.0, 0.92, 0.70) * bloom * exp(-radius * radius * 2.2);

  // faint dust motes drifting in the beam
  let motes = fbm2(c * 6.0 + vec2<f32>(0.0, u.time * 0.12), 4u);
  col = col + vec3<f32>(1.0, 0.95, 0.80) * pow(motes, 5.0) * 0.22;

  col = tonemap_aces(col * 1.15);
  return vec4<f32>(col, 1.0);
}
`,
};
