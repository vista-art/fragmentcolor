export const plankton = {
  id: "plankton",
  title: "Plankton",
  caption:
    "Bioluminescent cells adrift in a dark tide. Your hand is a light they cannot resist pressing against.",
  hint: "move to stir the tide · hold to glow · scroll to zoom",
  slugs: ["noise/worley2", "noise/fbm2", "hash/hash12", "color/tonemap_aces"],
  source: /* wgsl */ `
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let st = centered(in.uv);
  let m = centered(u.mouse);
  let t = u.time;

  // the cursor bulges space outward, shouldering the cells aside
  let away = st - m;
  let d = length(away);
  let shove = 1.0 + (0.55 + 0.65 * u.press + 0.9 * u.pulse) * exp(-d * d * 2.4);
  var p = m + away * shove;

  // slow tidal drift
  p = p + vec2<f32>(
    fbm2(p * 0.8 + vec2<f32>(t * 0.05, 0.0), 3u),
    fbm2(p * 0.8 + vec2<f32>(7.0, -t * 0.04), 3u)
  ) * 0.5 - vec2<f32>(0.25);

  // large cells
  let w = worley2(p * 2.4 + vec2<f32>(0.0, t * 0.06));
  let edge = w.y - w.x;
  let membrane = smoothstep(0.10, 0.015, edge);
  let vein = smoothstep(0.030, 0.0, edge);
  let body = exp(-w.x * w.x * 2.6);

  // small deep layer
  let w2 = worley2(p * 6.0 + vec2<f32>(3.7, -t * 0.09));
  let deep = exp(-w2.x * w2.x * 8.0);

  var col = vec3<f32>(0.004, 0.014, 0.030);
  col = col + vec3<f32>(0.00, 0.14, 0.20) * body * (0.35 + 0.35 * sin(t * 0.8 + w.x * 7.0));
  col = col + vec3<f32>(0.08, 0.28, 0.36) * deep * 0.30;
  let breath = 0.45 + 0.40 * sin(t * 1.3 + w.y * 5.0);
  col = col + vec3<f32>(0.12, 0.65, 0.78) * membrane * breath;
  col = col + vec3<f32>(0.75, 1.0, 1.0) * vein * (0.35 + 0.30 * breath);

  // drifting bioluminescent motes
  let fcell = floor(p * 20.0 + vec2<f32>(0.0, t * 0.15));
  let flocal = fract(p * 20.0 + vec2<f32>(0.0, t * 0.15)) - 0.5;
  let fh = hash12(fcell);
  let fpos = vec2<f32>(hash12(fcell + vec2<f32>(11.0, 5.0)), hash12(fcell + vec2<f32>(27.0, 3.0))) - 0.5;
  let fd = length(flocal - fpos * 0.6);
  let firefly = step(0.992, fh) * smoothstep(0.10, 0.02, fd) * (0.5 + 0.5 * sin(t * 3.0 + fh * 90.0));
  col = col + vec3<f32>(0.50, 1.0, 0.90) * firefly * 1.6;

  // the hand light itself
  col = col + vec3<f32>(0.35, 0.85, 1.0) * exp(-d * d * 4.5) * (0.14 + 0.45 * u.press + 0.7 * u.pulse);

  col = tonemap_aces(col * 1.25);
  return vec4<f32>(col, 1.0);
}
`,
};
