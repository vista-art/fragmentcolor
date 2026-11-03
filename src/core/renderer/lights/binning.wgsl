// Bins a pass's bounded lights into the clusters of its light grid. Composes
// after `src/scene/material/light.wgsl`. One thread per cluster: it counts the
// lights whose box holds the cluster and whose sphere reaches it, claims room
// for their indices, then writes them in ascending order.

// One bounded light. `cells` packs the box of clusters the CPU found, as
// x0, x1, y0, y1, z0 and z1 at five bits each, all inclusive.
struct Bin {
  center: vec3<f32>,
  radius: f32,
  light: u32,
  cells: u32,
}

// The grid, the bins and the cursor that hands out room in `light_grid`.
// The index lists start at word `list_base`, after the promoted lights and
// two words per cluster.
struct Bins {
  view_proj: mat4x4<f32>,
  depth_plane: vec4<f32>,
  size: vec3<u32>,
  slicing: u32,
  slice_scale: f32,
  slice_bias: f32,
  list_base: u32,
  count: u32,
  cursor: atomic<u32>,
  bins: array<Bin>,
}

@group(0) @binding(0) var<storage, read_write> bins: Bins;
@group(0) @binding(1) var<storage, read_write> light_grid: array<u32>;

const BATCH: u32 = 64u;
var<workgroup> batch: array<Bin, 64>;
var<workgroup> loaded: u32;

// The depth where slice coordinate `slice` lies.
fn depth_at(slice: f32) -> f32 {
  let value = (slice - bins.slice_bias) / bins.slice_scale;
  if (bins.slicing == 1u) {
    return exp2(value);
  }
  return value;
}

// The clip-space x or y of tile coordinate `cell` on an axis of `size` tiles.
fn to_ndc(cell: f32, size: u32) -> f32 {
  return cell / f32(size) * 2.0 - 1.0;
}

// Whether a sphere reaches the half-space where `dot(plane, point)` is at least 0.
fn reaches(plane: vec4<f32>, center: vec3<f32>, radius: f32) -> bool {
  return dot(plane.xyz, center) + plane.w >= -radius * length(plane.xyz);
}

// The bounds of one cluster, widened by the grid margin.
struct Cluster {
  cell: vec3<u32>,
  left: vec4<f32>,
  right: vec4<f32>,
  bottom: vec4<f32>,
  top: vec4<f32>,
  near: f32,
  far: f32,
}

fn find_cluster(index: u32) -> Cluster {
  let size = bins.size;
  let cell = vec3<u32>(index % size.x, (index / size.x) % size.y, index / (size.x * size.y));
  let m = transpose(bins.view_proj);
  let low = vec3<f32>(cell) - vec3<f32>(GRID_MARGIN);
  let high = vec3<f32>(cell) + vec3<f32>(1.0 + GRID_MARGIN);
  var cluster: Cluster;
  cluster.cell = cell;
  cluster.left = m[0] - to_ndc(low.x, size.x) * m[3];
  cluster.right = to_ndc(high.x, size.x) * m[3] - m[0];
  cluster.bottom = m[1] - to_ndc(low.y, size.y) * m[3];
  cluster.top = to_ndc(high.y, size.y) * m[3] - m[1];
  cluster.near = depth_at(low.z);
  cluster.far = depth_at(high.z);
  return cluster;
}

// Whether `bin` belongs in `cluster`: its box holds the cluster and its
// sphere, widened a little for rounding, reaches every bound.
fn covers(bin: Bin, cluster: Cluster) -> bool {
  let cells = bin.cells;
  let cell = cluster.cell;
  let low = vec3<u32>(cells & 31u, (cells >> 10u) & 31u, (cells >> 20u) & 31u);
  let high = vec3<u32>((cells >> 5u) & 31u, (cells >> 15u) & 31u, (cells >> 25u) & 31u);
  if (any(cell < low) || any(cell > high)) {
    return false;
  }
  let radius = bin.radius * 1.0001;
  let depth = dot(bins.depth_plane, vec4<f32>(bin.center, 1.0));
  return depth + radius >= cluster.near
    && depth - radius <= cluster.far
    && reaches(cluster.left, bin.center, radius)
    && reaches(cluster.right, bin.center, radius)
    && reaches(cluster.bottom, bin.center, radius)
    && reaches(cluster.top, bin.center, radius);
}

@compute @workgroup_size(64)
fn bin_lights(
  @builtin(global_invocation_id) id: vec3<u32>,
  @builtin(local_invocation_index) local: u32,
) {
  if (local == 0u) {
    loaded = bins.count;
  }
  let count = workgroupUniformLoad(&loaded);
  let batches = (count + BATCH - 1u) / BATCH;
  let clusters = bins.size.x * bins.size.y * bins.size.z;
  let cluster = find_cluster(id.x);
  let live = id.x < clusters;

  var found = 0u;
  for (var chunk = 0u; chunk < batches; chunk = chunk + 1u) {
    let index = chunk * BATCH + local;
    if (index < count) {
      batch[local] = bins.bins[index];
    }
    workgroupBarrier();
    let size = min(BATCH, count - chunk * BATCH);
    for (var j = 0u; j < size; j = j + 1u) {
      if (live && covers(batch[j], cluster)) {
        found = found + 1u;
      }
    }
    workgroupBarrier();
  }

  var first = 0u;
  if (live) {
    first = bins.list_base + atomicAdd(&bins.cursor, found);
    let entry = bins.list_base - 2u * clusters + 2u * id.x;
    if (entry + 1u < arrayLength(&light_grid)) {
      light_grid[entry] = first;
      light_grid[entry + 1u] = found;
    }
  }

  var written = 0u;
  for (var chunk = 0u; chunk < batches; chunk = chunk + 1u) {
    let index = chunk * BATCH + local;
    if (index < count) {
      batch[local] = bins.bins[index];
    }
    workgroupBarrier();
    let size = min(BATCH, count - chunk * BATCH);
    for (var j = 0u; j < size; j = j + 1u) {
      if (live && covers(batch[j], cluster)) {
        let slot = first + written;
        if (slot < arrayLength(&light_grid)) {
          light_grid[slot] = batch[j].light;
        }
        written = written + 1u;
      }
    }
    workgroupBarrier();
  }
}
