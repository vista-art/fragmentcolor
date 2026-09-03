export const monolith = {
  id: "monolith",
  title: "Monolith",
  caption:
    "An obsidian body carved from signed distance, orbited by three restless moons. Hold to charge its core.",
  hint: "drag to orbit · scroll to approach · hold to charge",
  slugs: [
    "sdf/octahedron",
    "sdf/torus",
    "sdf/box",
    "sdf/op_smooth_union",
    "sdf/op_smooth_subtract",
    "sdf/op_twist",
    "raymarch/normal_from_sdf_tetra",
    "lighting/fresnel_schlick",
    "lighting/rim",
    "gradient/palette_iq",
    "color/tonemap_aces",
  ],
  source: /* wgsl */ `
fn rot2(a: f32) -> mat2x2<f32> {
  let c = cos(a);
  let s = sin(a);
  return mat2x2<f32>(vec2<f32>(c, s), vec2<f32>(-s, c));
}

fn scene_sdf(p_world: vec3<f32>) -> f32 {
  var p = p_world;
  let spin = rot2(u.time * 0.12);
  let xz = spin * p.xz;
  p = vec3<f32>(xz.x, p.y, xz.y);

  let twisted = op_twist(p, 0.55 * sin(u.time * 0.13) + 0.3 * u.press);
  let core = octahedron(twisted, 1.05);
  let ring = torus(twisted, vec2<f32>(0.95, 0.26));
  let upright = vec3<f32>(twisted.x, twisted.z, twisted.y);
  let ring_v = torus(upright, vec2<f32>(0.90, 0.24));
  let cube = box(twisted, vec3<f32>(0.66, 0.66, 0.66));

  let fused = op_smooth_union(core, ring, 0.28);
  let carved = op_smooth_subtract(op_smooth_subtract(cube, ring, 0.16), ring_v, 0.16);
  let morph = 0.5 + 0.5 * sin(u.time * 0.17);
  // the 0.9 keeps the estimate conservative while the twist bends distances
  var d = mix(fused, carved, morph) * 0.9;

  // three moons, merging like mercury when they graze the body
  for (var i = 0; i < 3; i = i + 1) {
    let fi = f32(i);
    let a = u.time * (0.35 + 0.10 * fi) + fi * 2.094;
    let orbit = 1.9 + 0.15 * sin(u.time * 0.5 + fi * 1.7);
    let c = vec3<f32>(cos(a) * orbit, 0.55 * sin(u.time * 0.8 + fi * 2.4), sin(a) * orbit);
    d = op_smooth_union(d, length(p_world - c) - 0.14, 0.4);
  }

  // shock ripple on click
  d = d + sin(length(p_world) * 22.0 - u.time * 9.0) * 0.012 * u.pulse;
  return d;
}

fn sky(rd: vec3<f32>, warm_dir: vec3<f32>, cool_dir: vec3<f32>) -> vec3<f32> {
  var col = mix(vec3<f32>(0.010, 0.011, 0.020), vec3<f32>(0.035, 0.045, 0.075), rd.y * 0.5 + 0.5);
  col = col + vec3<f32>(0.30, 0.42, 0.70) * pow(max(dot(rd, cool_dir), 0.0), 3.0) * 0.16;
  col = col + vec3<f32>(0.55, 0.42, 0.28) * pow(max(dot(rd, warm_dir), 0.0), 5.0) * 0.10;
  return col;
}

fn soft_shadow(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
  var res = 1.0;
  var t = 0.06;
  for (var i = 0; i < 24; i = i + 1) {
    let h = scene_sdf(ro + rd * t);
    res = min(res, 10.0 * h / t);
    t = t + clamp(h, 0.02, 0.35);
    if (res < 0.005 || t > 8.0) { break; }
  }
  return clamp(res, 0.0, 1.0);
}

fn ambient_occ(p: vec3<f32>, n: vec3<f32>) -> f32 {
  var occ = 0.0;
  var w = 1.0;
  for (var i = 1; i <= 4; i = i + 1) {
    let h = 0.06 * f32(i);
    occ = occ + w * (h - scene_sdf(p + n * h));
    w = w * 0.6;
  }
  return clamp(1.0 - 2.2 * occ, 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let aspect = u.resolution.x / max(u.resolution.y, 1.0);
  let sp = (in.uv * 2.0 - vec2<f32>(1.0)) * vec2<f32>(aspect, 1.0);

  let yaw = u.drag.x + u.time * 0.05;
  let pitch = clamp(u.drag.y, -1.25, 1.25);
  // never dolly inside the moons' orbital shell
  let dist = max(4.4 / u.zoom, 2.4);
  let ro = dist * vec3<f32>(cos(pitch) * sin(yaw), sin(pitch), cos(pitch) * cos(yaw));

  let fwd = normalize(-ro);
  let right = normalize(cross(fwd, vec3<f32>(0.0, 1.0, 0.0)));
  let up = cross(right, fwd);
  let rd = normalize(fwd * 1.5 + right * sp.x + up * sp.y);

  let warm_dir = normalize(vec3<f32>(0.55, 0.65, 0.35));
  let cool_dir = normalize(vec3<f32>(-0.60, 0.25, -0.50));
  let warm = vec3<f32>(1.00, 0.80, 0.58);
  let cool = vec3<f32>(0.42, 0.58, 1.00);

  var t = 0.0;
  var glow = 0.0;
  var hit = false;
  for (var i = 0; i < 96; i = i + 1) {
    let p = ro + rd * t;
    let d = scene_sdf(p);
    glow = glow + 0.014 / (0.02 + abs(length(p) - 0.30) * 2.5) * exp(-t * 0.18);
    if (d < 0.0012 * t) { hit = true; break; }
    t = t + d * 0.85;
    if (t > 18.0) { break; }
  }

  var col = sky(rd, warm_dir, cool_dir);

  if (hit) {
    let p = ro + rd * t;
    let h = 0.0016;
    let n = normal_from_sdf_tetra(
      scene_sdf(p + vec3<f32>( h, -h, -h)),
      scene_sdf(p + vec3<f32>(-h, -h,  h)),
      scene_sdf(p + vec3<f32>(-h,  h, -h)),
      scene_sdf(p + vec3<f32>( h,  h,  h))
    );
    let v = -rd;
    let shadow = soft_shadow(p + n * 0.03, warm_dir);
    let ao = ambient_occ(p, n);

    let dif1 = max(dot(n, warm_dir), 0.0) * shadow;
    let dif2 = max(dot(n, cool_dir), 0.0);
    let h1 = normalize(warm_dir + v);
    let h2 = normalize(cool_dir + v);
    let spec1 = pow(max(dot(n, h1), 0.0), 90.0);
    let spec2 = pow(max(dot(n, h2), 0.0), 40.0);
    let nv = max(dot(n, v), 0.0);
    let fres = fresnel_schlick(nv, vec3<f32>(0.05, 0.05, 0.06));

    // oil-slick sheen riding the fresnel edge
    let sheen = palette_iq(
      1.0 - nv + u.time * 0.03,
      vec3<f32>(0.50, 0.50, 0.50),
      vec3<f32>(0.35, 0.35, 0.35),
      vec3<f32>(1.4, 1.4, 1.4),
      vec3<f32>(0.00, 0.33, 0.67)
    );

    let albedo = vec3<f32>(0.055, 0.050, 0.070);
    col = albedo * (dif1 * warm * 2.4 + dif2 * cool * 1.1 + vec3<f32>(0.20) * ao);
    col = col + (spec1 * warm * shadow * 1.3 + spec2 * cool * 0.45) * fres * 14.0;
    col = col + sheen * fres * ao * (0.7 + 0.35 * u.press);
    col = col + cool * rim(n, v, 3.0) * 0.45 * ao;
    col = col + sky(reflect(rd, n), warm_dir, cool_dir) * fres * ao * 2.5;

    let fog = 1.0 - exp(-0.012 * t * t);
    col = mix(col, sky(rd, warm_dir, cool_dir), fog);
  }

  // the charged heart, bleeding through everything
  let heart = vec3<f32>(1.00, 0.55, 0.25) * (0.16 + 1.7 * u.press + 1.2 * u.pulse);
  col = col + heart * glow * 0.55;

  col = tonemap_aces(col * 1.35);
  return vec4<f32>(col, 1.0);
}
`,
};
