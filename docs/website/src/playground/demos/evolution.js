// Evolution: a swarm like Life's whose creatures live, eat, die, and split in
// two. Four groups hunt one another in a circle, and every creature carries
// genes for which groups it follows or flees and how much armor it wears.
// Children mix their genes with a mate's and mutate, so lineages drift and
// new kinds of creatures appear. The host program and code tabs are Life's.

import { CLEAR, PRESENT, RELEASE, swarm } from "./swarm.js";

const FIELD = /* wgsl */ `// Blurs the creatures counted per grid cell into densities and their slopes.
// main.js puts the sizes (COUNT, ROWS, COLUMNS, CELLS, PIXELS) ahead of every
// file. update.wgsl reads this buffer and declares Field the same way.

// At one cell: each group's density slope along x and along y, the crowd of all
// groups with its slope in xy and its density in z, and each group's density.
struct Field {
  slope_x: vec4<f32>,
  slope_y: vec4<f32>,
  crowd: vec4<f32>,
  density: vec4<f32>,
}

@group(0) @binding(0) var<uniform> columns: u32;
@group(0) @binding(1) var<storage, read> counts: array<vec4<u32>, CELLS>;
@group(0) @binding(2) var<storage, read_write> field: array<Field, CELLS>;

const RADIUS: i32 = 4; // kernel radius in cells
const SPREAD: f32 = 2.0; // how far creatures sense each other, in cells
const CROWD: f32 = 0.8; // how far crowding pushes, in cells

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  if (gid.x >= columns || gid.y >= ROWS) {
    return;
  }
  // Densities relative to a full swarm's mean keep forces alike at any slot count.
  let scale = f32(columns * ROWS) / f32(COUNT);

  // Gaussian-weighted slopes, normalized so a linear ramp reads as its own slope.
  var slope_x = vec4<f32>(0.0);
  var slope_y = vec4<f32>(0.0);
  var crowd = vec3<f32>(0.0);
  var near = vec4<f32>(0.0);
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
      near += density * push;
      sense_norm += sense * offset.x * offset.x;
      crowd_norm += vec2<f32>(push * offset.x * offset.x, push);
    }
  }
  field[gid.y * COLUMNS + gid.x] = Field(
    slope_x / sense_norm,
    slope_y / sense_norm,
    vec4<f32>(crowd.xy / crowd_norm.x, crowd.z / crowd_norm.y, 0.0),
    near / crowd_norm.y,
  );
}
`;

const UPDATE = /* wgsl */ `// Lives one step of every creature: it moves along the density slopes its genes
// follow, eats the group it hunts, is bitten by the group that hunts it, and
// splits in two once it has enough energy. Each child mixes its genes with a
// mate's and mutates a little. field.wgsl declares Field the same way, and
// clear.wgsl and present.wgsl read counts and image as plain u32.

// A creature's place, motion, energy, and genes.
struct Creature {
  pos: vec2<f32>, // in [0, 1) across the canvas
  vel: vec2<f32>, // in cells per second
  follow: vec4<f32>, // how it follows (+) or flees (-) each group: its senses
  energy: f32,
  armor: f32, // 0..0.9: blunts bites, and costs energy to carry
  group: u32, // 0..3: hunts group + 1, and group + 3 hunts it
  hue: f32, // lineage color, which drifts a little with every birth
  age: f32, // seconds since birth
}

struct Field {
  slope_x: vec4<f32>,
  slope_y: vec4<f32>,
  crowd: vec4<f32>,
  density: vec4<f32>,
}

struct Params {
  mouse: vec2<f32>, // pointer in [0, 1) across the canvas
  grip: f32, // 1 while the pointer moves, fading to 0 once it rests
  feed: f32, // 0..40, energy per second the pointer gives the creatures under it
  heat: f32, // 0..200, strength of random kicks
  mutation: f32, // 0..1, how far a child's genes stray from its parents'
  mode: u32, // 0..2, colors by group, lineage, or energy
  columns: u32, // grid columns in use
  frame: u32,
  image: vec2<u32>, // image size in pixels
}

const GROUPS: u32 = 4u;
// A slot holds no creature, a living one, or one born this frame that lives from the next.
const EMPTY: u32 = 0u;
const ALIVE: u32 = 1u;
const NEWBORN: u32 = 2u;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> swarm: array<Creature, COUNT>;
@group(0) @binding(2) var<storage, read_write> slots: array<atomic<u32>, COUNT>;
@group(0) @binding(3) var<storage, read> field: array<Field, CELLS>;
@group(0) @binding(4) var<storage, read_write> counts: array<atomic<u32>, CELLS * GROUPS>;
@group(0) @binding(5) var<storage, read_write> image: array<atomic<u32>, PIXELS * 3u>;

const DT: f32 = 1.0 / 60.0;
const FORCE: f32 = 60.0; // acceleration per unit of density slope
const COHESION: f32 = 0.5; // added to every gene, which gathers crowds into colonies
const PRESSURE: f32 = 0.3; // how hard crowds push back, relative to FORCE
const PACKED: f32 = 12.0; // crowd density, in multiples of the mean, where cohesion turns into a push
const FRICTION: f32 = 4.0; // velocity decay per second
const TOP_SPEED: f32 = 40.0; // cells per second
const REACH: f32 = 16.0; // pointer reach in cells
const PULL: f32 = 200.0; // how hard a moving pointer draws creatures in
const START: f32 = 0.5; // share of the slots alive on the first frame

const GRAZE: f32 = 0.8; // energy per second a creature finds where grazing is best
const FORAGE: f32 = 4.0; // crowd density, in multiples of the mean, that leaves nothing to graze
const EAT: f32 = 2.0; // energy per second from prey at the mean density
const FULL_MEAL: f32 = 2.0; // prey density past which eating gets no faster
const BITE: f32 = 0.8; // energy per second lost to hunters at the mean density
const UPKEEP: f32 = 0.2; // energy per second it costs to live
const ARMOR_UPKEEP: f32 = 0.4; // extra upkeep per unit of armor
const FULL: f32 = 4.0; // the most energy a creature can store
const SPLIT: f32 = 2.0; // energy at which a creature splits in two
const LITTER: f32 = 0.45; // share of the energy that each of the two keeps after a split
const LIFESPAN: f32 = 40.0; // seconds a creature lives at most, give or take a quarter
const TRIES: u32 = 4u; // empty slots a creature tries per frame to place its child in
const SWITCH: f32 = 0.01; // chance per birth, at full mutation, that a child joins another group
const ANCESTORS: u32 = 16u; // picks the genes each group starts from, as Life's seed does

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

fn blend(a: Field, b: Field, t: f32) -> Field {
  return Field(
    mix(a.slope_x, b.slope_x, t),
    mix(a.slope_y, b.slope_y, t),
    mix(a.crowd, b.crowd, t),
    mix(a.density, b.density, t),
  );
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

// The genes a group starts from: how it follows (+) or flees (-) each of the four.
fn ancestor(group: u32) -> vec4<f32> {
  let row = vec4<f32>(random(ANCESTORS, group * 2u), random(ANCESTORS, group * 2u + 1u));
  return row * 2.0 - 1.0;
}

// A creature somewhere on the canvas, with its group's genes and a little of its own.
// Its age is random, so the first creatures don't all die at once.
fn spawn(i: u32) -> Creature {
  let group = i % GROUPS;
  let place = random(i, 0xfffffffeu);
  let own = vec4<f32>(random(i, 0xfffffffdu), random(i, 0xfffffffcu)) - 0.5;
  let traits = random(i, 0xfffffffbu);
  let hue = f32(group) / f32(GROUPS) + traits.y * 0.05;
  return Creature(place, vec2<f32>(0.0), ancestor(group) + own * 0.2, 1.0, traits.x * 0.2, group, hue, traits.y * LIFESPAN);
}

// A child of \`parent\` and \`mate\`: each gene comes from one of them, then strays a little.
fn child(parent: Creature, mate: Creature, seed: vec2<u32>) -> Creature {
  let coins = vec4<f32>(random(seed.x, seed.y), random(seed.y, seed.x)) < vec4<f32>(0.5);
  let noise = vec4<f32>(random(seed.x ^ 0x5bd1e995u, seed.y), random(seed.y ^ 0x5bd1e995u, seed.x)) - 0.5;
  let more = random(seed.x ^ 0x27d4eb2fu, seed.y) - 0.5;
  let stray = params.mutation;
  var kid = parent;
  kid.follow = clamp(select(parent.follow, mate.follow, coins) + noise * stray, vec4<f32>(-1.5), vec4<f32>(1.5));
  kid.armor = clamp(select(parent.armor, mate.armor, coins.x) + more.x * 0.2 * stray, 0.0, 0.9);
  kid.hue = fract(parent.hue + more.y * 0.08 * stray);
  kid.energy = parent.energy * LITTER;
  kid.age = 0.0;
  kid.vel = parent.vel + noise.xy * 2.0;
  if (random(seed.y, seed.x ^ 0x68e31da4u).x < SWITCH * stray) {
    kid.group = (parent.group + 1u + u32(more.x * 2.0 + 1.0)) % GROUPS;
  }
  return kid;
}

fn shade(c: Creature, speed: f32) -> vec3<f32> {
  let glow = 0.35 + 0.65 * min(c.energy / SPLIT, 1.0) + speed;
  switch params.mode {
    case 1u: {
      return palette_iq(c.hue, vec3<f32>(0.5), vec3<f32>(0.5), vec3<f32>(1.0), vec3<f32>(0.0, 0.33, 0.67)) * glow;
    }
    case 2u: {
      return turbo(min(c.energy / SPLIT, 1.0));
    }
    default: {
      let hue = palette_iq(
        f32(c.group) / f32(GROUPS),
        vec3<f32>(0.5),
        vec3<f32>(0.5),
        vec3<f32>(1.0),
        vec3<f32>(0.0, 0.33, 0.67),
      );
      return hue * glow;
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

  // The first frame fills part of the slots with creatures of all four groups.
  let state = atomicLoad(&slots[i]);
  if (state == EMPTY) {
    if (params.frame == 0u && random(i, 0xfffffff0u).x < START) {
      swarm[i] = spawn(i);
      atomicStore(&slots[i], ALIVE);
    }
    return;
  }
  // A parent may still be writing a newborn, so it moves from the next frame on.
  if (state == NEWBORN) {
    atomicStore(&slots[i], ALIVE);
    return;
  }

  var c = swarm[i];
  let size = vec2<f32>(f32(params.columns), f32(ROWS));
  let pos = c.pos * size;
  // Sampling a little off the creature keeps crowds from locking onto the grid.
  let here = sample(pos + random(i, params.frame ^ 0x9e3779b9u) - 0.5);
  // Cohesion fades as a crowd packs in, and pushes it apart past PACKED.
  let packed = here.crowd.z / PACKED;
  let follow = c.follow + COHESION * (1.0 - packed * packed);
  let slope = vec2<f32>(dot(follow, here.slope_x), dot(follow, here.slope_y));
  var accel = FORCE * (slope - PRESSURE * here.crowd.xy);

  // The pointer draws creatures in while it moves, and feeds the ones it reaches.
  var toward = (params.mouse - c.pos) * size;
  toward -= round(toward / size) * size; // the short way around the wrapped world
  let distance = length(toward);
  let near = exp(-distance * distance / (REACH * REACH)) * params.grip;
  accel += PULL * toward / max(distance, 1.0) * near;

  accel += (random(i, params.frame) - 0.5) * params.heat;
  c.vel = c.vel * exp(-FRICTION * DT) + accel * DT;
  let speed = length(c.vel);
  if (speed > TOP_SPEED) {
    c.vel *= TOP_SPEED / speed;
  }
  c.pos = fract((pos + c.vel * DT) / size);

  // It eats the group after its own and feeds the group before it. Armor blunts
  // bites but costs upkeep, and running costs energy too.
  let prey = min(here.density[(c.group + 1u) % GROUPS], FULL_MEAL);
  let hunters = here.density[(c.group + GROUPS - 1u) % GROUPS];
  let effort = speed / TOP_SPEED;
  // Grazing takes company: it pays most halfway to FORAGE, and nothing alone or in a packed crowd.
  let share = clamp(here.crowd.z / FORAGE, 0.0, 1.0);
  let graze = GRAZE * 4.0 * share * (1.0 - share);
  let upkeep = UPKEEP + ARMOR_UPKEEP * c.armor + effort * effort;
  c.energy += (graze + EAT * prey - BITE * hunters * (1.0 - c.armor) - upkeep) * DT;
  c.energy = min(c.energy + params.feed * near * DT, FULL);
  c.age += DT;
  let lifespan = LIFESPAN * (0.75 + 0.5 * random(i, 0xfffffffau).x);
  if (c.energy <= 0.0 || c.age > lifespan) {
    atomicStore(&slots[i], EMPTY);
    return;
  }

  // With enough energy it splits, if its crowd has room and one of a few random slots is empty.
  if (c.energy >= SPLIT && here.crowd.z < PACKED && params.frame > 0u) {
    let seed = pcg2d(vec2<u32>(i, params.frame));
    // A random creature of its own group mates with it. With none, it mates with itself.
    var partner = swarm[seed.x % COUNT];
    if (partner.group != c.group || atomicLoad(&slots[seed.x % COUNT]) != ALIVE) {
      partner = c;
    }
    for (var t = 0u; t < TRIES; t++) {
      let j = pcg2d(vec2<u32>(seed.y, t)).x % COUNT;
      if (atomicCompareExchangeWeak(&slots[j], EMPTY, NEWBORN).exchanged) {
        swarm[j] = child(c, partner, vec2<u32>(seed.y, t));
        c.energy *= LITTER;
        break;
      }
    }
  }
  swarm[i] = c;

  // Count into one of the four nearest cells, picked with bilinear odds, so crowds stay round.
  let home = floor(c.pos * size - 0.5 + random(params.frame, i));
  atomicAdd(&counts[index(i32(home.x), i32(home.y)) * GROUPS + c.group], 1u);

  let pixel = min(vec2<u32>(c.pos * vec2<f32>(params.image)), params.image - 1u);
  let light = vec3<u32>(shade(c, min(speed / 10.0, 1.0)) * 255.0);
  let slot = (pixel.y * params.image.x + pixel.x) * 3u;
  atomicAdd(&image[slot], light.r);
  atomicAdd(&image[slot + 1u], light.g);
  atomicAdd(&image[slot + 2u], light.b);
}
`;

// The values main.js sets in setup, in order.
const VALUES = {
  "params.feed": 6,
  "params.heat": 20,
  "params.mutation": 0.1,
  "params.mode": 0,
};

// Slots in the swarm buffer. Creatures fill part of them and leave the rest for children.
const COUNT = 1 << 18;

export default {
  id: "evolution",
  title: "Evolution",
  summary:
    "Creatures eat, die, and split in two. Four groups hunt each other in a circle, and each creature's genes decide which groups it follows or flees and how much armor it wears. Children mix their parents' genes and mutate, so lineages drift. The pointer feeds the creatures under it.",
  category: "Compute",
  thumbnail: "/playground/thumbs/evolution.webp",
  chip: { x: 0.82, y: 0.47, zoom: 4 },
  kind: "compute",
  requires: "webgpu",
  needs: RELEASE,
  poster: "/playground/thumbs/evolution.webp",
  library: ["gradient/palette_iq", "gradient/turbo", "hash/pcg2d"],
  values: VALUES,
  files: { "field.wgsl": FIELD, "clear.wgsl": CLEAR, "update.wgsl": UPDATE, "present.wgsl": PRESENT },
  ...swarm({ count: COUNT, values: VALUES, unsigned: new Set(["params.mode"]) }),
};
