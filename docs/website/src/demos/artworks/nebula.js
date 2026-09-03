export const nebula = {
  id: "nebula",
  title: "Nebula",
  caption:
    "Fractal noise folded over itself until it ignites. The cursor is a gravity well; hold to feed it.",
  hint: "move to bend the cloud · hold to feed the well · scroll to zoom",
  slugs: ["noise/fbm2", "hash/hash12", "color/tonemap_aces"],
  source: /* wgsl */ `
fn star_layer(p: vec2<f32>, density: f32, t: f32) -> f32 {
  let cell = floor(p * density);
  let seed = hash12(cell);
  if (seed < 0.985) { return 0.0; }
  let jitter = vec2<f32>(hash12(cell + vec2<f32>(17.0, 3.0)), hash12(cell + vec2<f32>(9.0, 41.0)));
  let local = fract(p * density) - 0.5 - (jitter - 0.5) * 0.7;
  let twinkle = 0.55 + 0.45 * sin(t * (2.0 + 5.0 * seed) + seed * 60.0);
  return twinkle * smoothstep(0.10, 0.0, length(local)) * (seed - 0.985) * 66.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let st = centered(in.uv);
  let m = centered(u.mouse);
  let t = u.time * 0.05;

  // gravity well: pull the sampling domain toward the cursor
  let toward = m - st;
  let dist = length(toward);
  let strength = (0.30 + 0.55 * u.press + 0.35 * u.pulse) / (1.0 + dist * dist * 3.0);
  let p = st + toward * strength;

  // double domain warp
  let q = vec2<f32>(
    fbm2(p * 1.5 + vec2<f32>(t * 0.9, -t * 0.6), 5u),
    fbm2(p * 1.5 + vec2<f32>(4.7, 9.2) + vec2<f32>(-t, t * 0.8), 5u)
  );
  let r = vec2<f32>(
    fbm2(p * 1.8 + q * 3.0 + vec2<f32>(1.7, 9.2), 5u),
    fbm2(p * 1.8 + q * 3.0 + vec2<f32>(8.3, 2.8) + vec2<f32>(t * 0.7), 5u)
  );
  let f = fbm2(p * 2.1 + r * 2.6, 6u);

  // clouds condense out of the void; the void wins by default
  let density = pow(smoothstep(0.48, 0.92, f), 1.7);
  var col = vec3<f32>(0.010, 0.012, 0.024);

  let cold_dust = vec3<f32>(0.13, 0.24, 0.72);
  let warm_dust = vec3<f32>(0.96, 0.34, 0.10);
  var cloud = mix(cold_dust, warm_dust, smoothstep(0.25, 0.80, q.x));
  cloud = cloud + vec3<f32>(0.48, 0.16, 0.66) * smoothstep(0.45, 0.90, r.y) * 0.9;
  col = col + cloud * density * 1.4;

  // hot young cores deep inside the folds
  col = col + vec3<f32>(1.0, 0.78, 0.48) * pow(f, 9.0) * 1.3;

  // starfield behind the gas, dimmed where the cloud is dense
  let clear_sky = smoothstep(0.60, 0.30, f);
  col = col + vec3<f32>(0.85, 0.90, 1.0) * star_layer(st, 14.0, u.time) * clear_sky;
  col = col + vec3<f32>(1.0, 0.95, 0.85) * star_layer(st + vec2<f32>(31.7, 7.3), 7.0, u.time * 0.7) * clear_sky;

  // the well itself glows
  let well = exp(-dist * dist * 5.0);
  col = col + vec3<f32>(0.55, 0.70, 1.0) * well * (0.10 + 0.55 * u.press + 0.75 * u.pulse);

  col = tonemap_aces(col * 1.3);
  return vec4<f32>(col, 1.0);
}
`,
};
