// FragmentColor's light types and light grid constants. The PBR shader and
// the light binning pass both compose this file.

// One glTF KHR_lights_punctual light, 64 bytes. `kind` selects the fields
// the shading reads:
//   0 = directional: parallel rays along `direction`
//   1 = point: inverse-square falloff from `position`, cut off at `range`
//       when it is above 0
//   2 = spot: a point light lit only where the angle to `direction` has a
//       cosine of at least `outer_cone_cos`, fading in from `inner_cone_cos`
// `intensity` scales `color`.
struct Light {
  kind: u32,
  intensity: f32,
  range: f32,
  inner_cone_cos: f32,
  position: vec3<f32>,
  outer_cone_cos: f32,
  direction: vec3<f32>,
  color: vec3<f32>,
}

// The clusters a pass bins its lights into, 112 bytes. `view_proj` maps a
// world point to the grid's clip space, and `depth_plane` gives its depth.
// `slicing` is 0 when the pass has no grid, 1 for logarithmic depth slices
// and 2 for linear ones, and a slice is `log2(depth)` or `depth` times
// `slice_scale` plus `slice_bias`. No bounded light reaches past `reach`.
struct LightGrid {
  view_proj: mat4x4<f32>,
  depth_plane: vec4<f32>,
  size: vec3<u32>,
  slicing: u32,
  slice_scale: f32,
  slice_bias: f32,
  reach: f32,
}

// Tiles along the long side of the grid's view, along the short side, and
// depth slices. A portrait view puts the long side vertical.
const GRID_LONG: u32 = 16u;
const GRID_SHORT: u32 = 9u;
const GRID_SLICES: u32 = 24u;

// How far outside the grid, in clusters, a point still takes the nearest
// cluster. The binning widens every cluster by the same margin.
const GRID_MARGIN: f32 = 0.015625;

// The two `light_cell` results that name no cluster: read every light, or
// read only the lights without a range.
const CELL_EVERY: u32 = 0xffffffffu;
const CELL_GLOBAL: u32 = 0xfffffffeu;
