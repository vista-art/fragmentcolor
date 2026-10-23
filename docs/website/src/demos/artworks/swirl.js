export const swirl = {
  id: "swirl",
  title: "Swirl",
  caption:
    "The site's original swirl: a cosine palette folded over rings and radar arms. Horizontal motion adds arms; vertical motion winds them into a spiral.",
  hint: "move to add and wind the arms · drag to spin · hold to tighten · scroll to zoom",
  slugs: [],
  source: /* wgsl */ `
const TAU: f32 = 6.283185307179586;

fn pal(t: f32, brightness: vec3<f32>, contrast: vec3<f32>, oscillation: vec3<f32>, phase: vec3<f32>) -> vec3<f32> {
  return brightness + contrast * cos(TAU * (oscillation * t + phase));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let uv = centered(in.uv);
  let t = u.time;
  let slow = t * (0.25 + 0.35 * u.press);

  // the cursor's x sets the arm count, a whole number so the arms meet across
  // the atan2 seam, and its y winds them into a spiral either way
  let m = clamp(pointer_uv(), vec2<f32>(0.0), vec2<f32>(1.0));
  let arms = 1.0 + floor(m.x * 5.99);
  let wind = (m.y - 0.5) * 12.0;

  // drag spins the radar arms, a press tightens the rings, a click sends a ripple
  let radius = length(uv);
  let ripple = u.pulse * 2.0 * exp(-radius * 3.0);
  let rings = sin(slow - radius * (15.0 + 6.0 * u.press) + ripple);
  let angle = atan2(uv.y, uv.x) + u.drag.x * 1.5;
  let radar = sin(arms * (angle + wind * radius) + slow);
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
