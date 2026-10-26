// Particle life: four species follow or flee each other's density over a grid.
// Four passes run every frame and share their storage buffers by name. The host
// program, clear.wgsl, present.wgsl, and the code tabs come from swarm.js.

import { CLEAR, PRESENT, RELEASE, swarm } from "./swarm.js";

const FIELD = /* wgsl */ `// Blurs the particles counted per grid cell into density slopes. main.js puts
// the sizes (COUNT, ROWS, COLUMNS, CELLS, PIXELS) ahead of every file.
// update.wgsl reads this buffer and declares Field the same way.

// Density slopes at one cell: each species along x and along y, then the crowd
// of all species, with its slope in xy and its density in z.
struct Field {
  slope_x: vec4<f32>,
  slope_y: vec4<f32>,
  crowd: vec4<f32>,
}

@group(0) @binding(0) var<uniform> columns: u32;
@group(0) @binding(1) var<storage, read> counts: array<vec4<u32>, CELLS>;
@group(0) @binding(2) var<storage, read_write> field: array<Field, CELLS>;

const RADIUS: i32 = 4; // kernel radius in cells
const SPREAD: f32 = 2.0; // how far species sense each other, in cells
const CROWD: f32 = 0.8; // how far crowding pushes, in cells

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  if (gid.x >= columns || gid.y >= ROWS) {
    return;
  }
  // Densities relative to the mean keep forces alike at any particle count.
  let scale = f32(columns * ROWS) / f32(COUNT);

  // Gaussian-weighted slopes, normalized so a linear ramp reads as its own slope.
  var slope_x = vec4<f32>(0.0);
  var slope_y = vec4<f32>(0.0);
  var crowd = vec3<f32>(0.0);
  var sense_norm = 0.0;
  var crowd_norm = vec2<f32>(0.0);
  for (var dy = -RADIUS; dy <= RADIUS; dy++) {
    for (var dx = -RADIUS; dx <= RADIUS; dx++) {
      let x = u32(i32(gid.x) + dx + i32(columns)) % columns;
      let y = u32(i32(gid.y) + dy + i32(ROWS)) % ROWS;
      let density = vec4<f32>(counts[y * COLUMNS + x]) * scale;
      let total = density.x + density.y + density.z + density.w;
      let offset = vec2<f32>(f32(dx), f32(dy));
      let squared = dot(offset, offset);
      let sense = exp(-squared / (2.0 * SPREAD * SPREAD));
      let push = exp(-squared / (2.0 * CROWD * CROWD));
      slope_x += density * sense * offset.x;
      slope_y += density * sense * offset.y;
      crowd += vec3<f32>(offset, 1.0) * total * push;
      sense_norm += sense * offset.x * offset.x;
      crowd_norm += vec2<f32>(push * offset.x * offset.x, push);
    }
  }
  field[gid.y * COLUMNS + gid.x] = Field(
    slope_x / sense_norm,
    slope_y / sense_norm,
    vec4<f32>(crowd.xy / crowd_norm.x, crowd.z / crowd_norm.y, 0.0),
  );
}
`;

const UPDATE = /* wgsl */ `// Moves every particle along the density slopes, counts it into the grid with
// atomicAdd, and adds its color to the image. field.wgsl declares Field the same
// way, and clear.wgsl and present.wgsl read counts and image as plain u32.

// Position in [0, 1) across the canvas, velocity in cells per second.
struct Particle {
  pos: vec2<f32>,
  vel: vec2<f32>,
}

struct Field {
  slope_x: vec4<f32>,
  slope_y: vec4<f32>,
  crowd: vec4<f32>,
}

struct Params {
  mouse: vec2<f32>, // pointer in [0, 1) across the canvas
  grip: f32, // 1 while the pointer moves, fading to 0 once it rests
  pull: f32, // -1200..1200, pointer force: positive attracts, negative repels
  heat: f32, // 0..200, strength of random kicks
  seed: u32, // 0..100, picks who follows whom
  mode: u32, // 0..2, colors by species, speed, or density
  columns: u32, // grid columns in use
  frame: u32,
  image: vec2<u32>, // image size in pixels
}

const SPECIES: u32 = 4u;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> swarm: array<Particle, COUNT>;
@group(0) @binding(2) var<storage, read> field: array<Field, CELLS>;
@group(0) @binding(3) var<storage, read_write> counts: array<atomic<u32>, CELLS * SPECIES>;
@group(0) @binding(4) var<storage, read_write> image: array<atomic<u32>, PIXELS * 3u>;

const DT: f32 = 1.0 / 60.0;
const FORCE: f32 = 60.0; // acceleration per unit of density slope
const COHESION: f32 = 0.6; // added to every attraction, which gathers crowds into colonies
const PRESSURE: f32 = 0.3; // how hard crowds push back, relative to FORCE
const PACKED: f32 = 12.0; // crowd density, in multiples of the mean, where cohesion turns into a push
const FRICTION: f32 = 4.0; // velocity decay per second
const TOP_SPEED: f32 = 40.0; // cells per second
const REACH: f32 = 16.0; // pointer reach in cells
const HOT: f32 = 10.0; // speed that shades as the hottest color

// The index of a grid cell, wrapping around the edges.
fn index(x: i32, y: i32) -> u32 {
  let column = u32(x + i32(params.columns)) % params.columns;
  let row = u32(y + i32(ROWS)) % ROWS;
  return row * COLUMNS + column;
}

// Two random numbers in [0, 1).
fn random(a: u32, b: u32) -> vec2<f32> {
  return vec2<f32>(pcg2d(vec2<u32>(a, b)) >> vec2<u32>(8u)) / 16777216.0;
}

// Row \`species\` of the attraction table that seed picks: how that species
// follows (+) or flees (-) each of the four.
fn attraction(species: u32) -> vec4<f32> {
  let row = vec4<f32>(random(params.seed, species * 2u), random(params.seed, species * 2u + 1u));
  return row * 2.0 - 1.0;
}

fn blend(a: Field, b: Field, t: f32) -> Field {
  return Field(mix(a.slope_x, b.slope_x, t), mix(a.slope_y, b.slope_y, t), mix(a.crowd, b.crowd, t));
}

// The field at a point, blended from the four nearest cell centers.
fn sample(pos: vec2<f32>) -> Field {
  let corner = pos - 0.5;
  let x = i32(floor(corner.x));
  let y = i32(floor(corner.y));
  let t = fract(corner);
  let top = blend(field[index(x, y)], field[index(x + 1, y)], t.x);
  let bottom = blend(field[index(x, y + 1)], field[index(x + 1, y + 1)], t.x);
  return blend(top, bottom, t.y);
}

fn shade(species: u32, speed: f32, density: f32) -> vec3<f32> {
  switch params.mode {
    case 1u: {
      return turbo(speed);
    }
    case 2u: {
      return turbo(density / 8.0);
    }
    default: {
      let hue = palette_iq(
        f32(species) / f32(SPECIES),
        vec3<f32>(0.5),
        vec3<f32>(0.5),
        vec3<f32>(1.0),
        vec3<f32>(0.0, 0.33, 0.67),
      );
      return hue * (0.35 + speed);
    }
  }
}

@compute @workgroup_size(256)
fn cs_main(
  @builtin(global_invocation_id) gid: vec3<u32>,
  @builtin(num_workgroups) groups: vec3<u32>,
) {
  let i = gid.y * groups.x * 256u + gid.x;
  if (i >= COUNT) {
    return;
  }
  let species = i % SPECIES;
  let size = vec2<f32>(f32(params.columns), f32(ROWS));

  // The first frame finds a new buffer all zeros and scatters the particles
  // across the canvas with small random velocities.
  var particle = swarm[i];
  if (all(particle.pos == vec2<f32>(0.0)) && all(particle.vel == vec2<f32>(0.0))) {
    particle = Particle(random(i, 0xfffffffeu), (random(i, 0xffffffffu) - 0.5) * 4.0);
  }
  let pos = particle.pos * size;
  // Sampling a little off the particle keeps crowds from locking onto the grid.
  let here = sample(pos + random(i, params.frame ^ 0x9e3779b9u) - 0.5);
  // Cohesion fades as a crowd packs in, and pushes it apart past PACKED.
  let packed = here.crowd.z / PACKED;
  let follow = attraction(species) + COHESION * (1.0 - packed * packed);
  let slope = vec2<f32>(dot(follow, here.slope_x), dot(follow, here.slope_y));
  var accel = FORCE * (slope - PRESSURE * here.crowd.xy);

  let pull = params.pull * params.grip;
  if (pull != 0.0) {
    var toward = (params.mouse - particle.pos) * size;
    toward -= round(toward / size) * size; // the short way around the wrapped world
    let distance = length(toward);
    accel += pull * toward / max(distance, 1.0) * exp(-distance * distance / (REACH * REACH));
  }

  accel += (random(i, params.frame) - 0.5) * params.heat;

  particle.vel = particle.vel * exp(-FRICTION * DT) + accel * DT;
  let speed = length(particle.vel);
  if (speed > TOP_SPEED) {
    particle.vel *= TOP_SPEED / speed;
  }
  particle.pos = fract((pos + particle.vel * DT) / size);
  swarm[i] = particle;

  // Count into one of the four nearest cells, picked with bilinear odds, so crowds stay round.
  let home = floor(particle.pos * size - 0.5 + random(params.frame, i));
  atomicAdd(&counts[index(i32(home.x), i32(home.y)) * SPECIES + species], 1u);

  let pixel = min(vec2<u32>(particle.pos * vec2<f32>(params.image)), params.image - 1u);
  let light = vec3<u32>(shade(species, min(speed / HOT, 1.0), here.crowd.z) * 255.0);
  let slot = (pixel.y * params.image.x + pixel.x) * 3u;
  atomicAdd(&image[slot], light.r);
  atomicAdd(&image[slot + 1u], light.g);
  atomicAdd(&image[slot + 2u], light.b);
}
`;

// The values main.js sets in setup, in order.
const VALUES = {
  "params.pull": 600,
  "params.heat": 20,
  "params.seed": 16,
  "params.mode": 0,
};

// The live values carry their scalar type. The defaults above need these named.
const UNSIGNED = new Set(["params.seed", "params.mode"]);

// Particles in the swarm buffer.
const COUNT = 1_000_000;

export default {
  id: "life",
  title: "Life",
  summary:
    "Four species follow or flee each other's density and gather into colonies that chase, split, and swirl. The pointer draws them close, and seed reshuffles who follows whom.",
  category: "Compute",
  // A held pointer gathers the swarm into a ball: ./fc capture thumbs life --hold 0.5,0.5 --wait 10
  thumbnail: "/playground/thumbs/life.webp",
  chip: { x: 0.5, y: 0.48, zoom: 3 },
  kind: "compute",
  requires: "webgpu",
  needs: RELEASE,
  poster: "/playground/life.webp",
  library: ["gradient/palette_iq", "gradient/turbo", "hash/pcg2d"],
  values: VALUES,
  files: { "field.wgsl": FIELD, "clear.wgsl": CLEAR, "update.wgsl": UPDATE, "present.wgsl": PRESENT },
  ...swarm({ id: "life", count: COUNT, values: VALUES, unsigned: UNSIGNED }),
};
