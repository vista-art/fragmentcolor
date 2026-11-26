// The lights of the built-in material shaders: the two roots the renderer
// fills and the loops that read them. Composes after `light.wgsl`. Each
// shader that composes it declares a `Surface` struct, with what its shading
// reads at a fragment, and a `shade_light(lt, world, n, v, surface)` that
// returns the light `lt` sends toward the eye from that surface.

// Every light that reaches the shader. Lights without a range come first,
// `global_count` of them, then the lights with one. `grid` locates each
// fragment's cluster in `light_grid`.
struct Lights {
  count: u32,
  global_count: u32,
  promoted_count: u32,
  ambient: vec3<f32>,
  grid: LightGrid,
  lights: array<Light>,
}

// `light_grid` starts with `promoted_count` light indices that every cluster
// reads. Cluster `c` follows at word `promoted_count + 2 * c`, as the word
// where its index list starts and the length of that list.
@group(0) @binding(1) var<storage, read> lights: Lights;
@group(0) @binding(2) var<storage, read> light_grid: array<u32>;

// The direction from `world` toward the light `lt`, and the light that
// arrives there. KHR_lights_punctual falloff:
//   directional: unattenuated parallel rays from `-direction`
//   point: inverse-square distance from `position`, with the
//          (1 - (d/range)^4) window when `range` is above 0
//   spot: point falloff times a smooth cone between the cone cosines
// The light grid relies on it: no light arrives past a light's range, or
// outside a spot light's outer cone.
struct LightRay {
  direction: vec3<f32>,
  radiance: vec3<f32>,
}

fn light_ray(lt: Light, world: vec3<f32>) -> LightRay {
  var l: vec3<f32>;
  var attenuation: f32 = 1.0;
  if (lt.kind == 0u) {
    l = normalize(-lt.direction);
  } else {
    let to_light = lt.position - world;
    let dist = length(to_light);
    l = to_light / max(dist, 1.0e-6);
    attenuation = 1.0 / max(dist * dist, 1.0e-4);
    if (lt.range > 0.0) {
      attenuation = attenuation * max(0.0, 1.0 - pow(dist / lt.range, 4.0));
    }
    if (lt.kind == 2u) {
      let spot_axis = normalize(-lt.direction);
      let cos_angle = dot(spot_axis, l);
      // smoothstep's curve between the cone cosines. Equal or inverted
      // angles leave no band, where smoothstep is undefined, so they get a
      // hard edge at the outer angle.
      let band = lt.inner_cone_cos - lt.outer_cone_cos;
      var cone = step(lt.outer_cone_cos, cos_angle);
      if (band > 0.0) {
        let t = clamp((cos_angle - lt.outer_cone_cos) / band, 0.0, 1.0);
        cone = t * t * (3.0 - 2.0 * t);
      }
      attenuation = attenuation * cone;
    }
  }
  return LightRay(l, lt.color * lt.intensity * attenuation);
}

// The cluster that holds `world`, or CELL_GLOBAL past every light's reach,
// or CELL_EVERY with no grid, behind the grid's eye, or outside the grid.
fn light_cell(world: vec3<f32>) -> u32 {
  let grid = lights.grid;
  if (grid.slicing == 0u) {
    return CELL_EVERY;
  }
  let point = vec4<f32>(world, 1.0);
  let depth = dot(grid.depth_plane, point);
  if (depth > grid.reach) {
    return CELL_GLOBAL;
  }
  let clip = grid.view_proj * point;
  if (clip.w <= 0.0) {
    return CELL_EVERY;
  }
  let size = vec3<f32>(grid.size);
  var slice = depth * grid.slice_scale + grid.slice_bias;
  if (grid.slicing == 1u) {
    slice = log2(max(depth, 1.0e-30)) * grid.slice_scale + grid.slice_bias;
  }
  let cell = vec3<f32>((clip.xy / clip.w * 0.5 + 0.5) * size.xy, slice);
  if (any(cell < vec3<f32>(-GRID_MARGIN)) || any(cell > size + GRID_MARGIN)) {
    return CELL_EVERY;
  }
  let index = min(vec3<u32>(max(cell, vec3<f32>(0.0))), grid.size - vec3<u32>(1u));
  return index.x + grid.size.x * (index.y + grid.size.y * index.z);
}

// The light every light that reaches `world` sends toward `v` from `surface`.
fn accumulate_lights(
  world: vec3<f32>,
  n: vec3<f32>,
  v: vec3<f32>,
  surface: Surface,
) -> vec3<f32> {
  var total = vec3<f32>(0.0);
  let cell = light_cell(world);
  if (cell == CELL_EVERY) {
    for (var i: u32 = 0u; i < lights.count; i = i + 1u) {
      total = total + shade_light(lights.lights[i], world, n, v, surface);
    }
    return total;
  }
  for (var i: u32 = 0u; i < lights.global_count; i = i + 1u) {
    total = total + shade_light(lights.lights[i], world, n, v, surface);
  }
  if (cell == CELL_GLOBAL) {
    return total;
  }
  for (var k: u32 = 0u; k < lights.promoted_count; k = k + 1u) {
    total = total + shade_light(lights.lights[light_grid[k]], world, n, v, surface);
  }
  let entry = lights.promoted_count + 2u * cell;
  let first = light_grid[entry];
  let listed = light_grid[entry + 1u];
  for (var k: u32 = 0u; k < listed; k = k + 1u) {
    total = total + shade_light(lights.lights[light_grid[first + k]], world, n, v, surface);
  }
  return total;
}
