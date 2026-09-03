export const swirl = {
  id: "swirl",
  title: "Swirl",
  caption:
    "The site's original swirl: a cosine palette folded over rings and a radar arm, now following your hand.",
  hint: "move to steer the eye · drag to spin · hold to tighten · scroll to zoom",
  slugs: [],
  source: /* wgsl */ `
const TAU: f32 = 6.283185307179586;

fn pal(t: f32, brightness: vec3<f32>, contrast: vec3<f32>, oscillation: vec3<f32>, phase: vec3<f32>) -> vec3<f32> {
  return brightness + contrast * cos(TAU * (oscillation * t + phase));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // the eye of the swirl drifts toward the cursor
  let m = centered(u.mouse);
  let uv = centered(in.uv) - m * 0.6;
  let t = u.time;
  let slow = t * (0.25 + 0.35 * u.press);

  // drag spins the radar arm, a press tightens the rings, a click sends a ripple
  let radius = length(uv);
  let ripple = u.pulse * 2.0 * exp(-radius * 3.0);
  let rings = sin(slow - radius * (15.0 + 6.0 * u.press) + ripple);
  let angle = atan2(uv.y, uv.x) + u.drag.x * 1.5;
  let radar = sin(angle + slow);
  let swirl = sin(rings + radar + slow);

  let brightnessBlend = 0.5 * (sin(t + length(uv * 20.0)) + 1.0);
  let contrastBlend = 0.5 * (sin(slow) + 1.0);
  let brightness = vec3<f32>(0.1 + brightnessBlend * 0.6);
  let contrast = vec3<f32>(0.2 + contrastBlend * 0.3 + 0.15 * u.press);
  let oscillation = vec3<f32>(0.4, 0.5 * (sin(slow) + 1.0), 0.2);
  let phase = vec3<f32>(0.7, 0.4, 0.1) + vec3<f32>(u.drag.y * 0.15);

  let color = pal(swirl, brightness, contrast, oscillation, phase);
  return vec4<f32>(color, 1.0);
}
`,
};
