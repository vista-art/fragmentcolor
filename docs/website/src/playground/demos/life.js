// Particle life: four species follow or flee each other's density over a grid.
// Four passes run every frame and share their storage buffers by name.

import { formatValue } from "../codegen.js";

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

const CLEAR = /* wgsl */ `// Empties the counts and the image before update.wgsl fills them again.

@group(0) @binding(0) var<storage, read_write> counts: array<vec4<u32>, CELLS>;
@group(0) @binding(1) var<storage, read_write> image: array<u32, PIXELS * 3u>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i < CELLS) {
    counts[i] = vec4<u32>(0u);
  }
  if (i < PIXELS) {
    image[i * 3u] = 0u;
    image[i * 3u + 1u] = 0u;
    image[i * 3u + 2u] = 0u;
  }
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
  let here = sample(pos);
  let follow = attraction(species) + COHESION;
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

const PRESENT = /* wgsl */ `// Tone-maps the image that update.wgsl adds up and draws it onto the canvas without stretching.

@group(0) @binding(0) var<uniform> image_size: vec2<u32>;
@group(0) @binding(1) var<storage, read> image: array<u32, PIXELS * 3u>;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

// Fullscreen triangle: three vertices that cover the canvas, with uv.y down.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  var corners = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  var uv = array<vec2<f32>, 3>(vec2<f32>(0.0, 1.0), vec2<f32>(2.0, 1.0), vec2<f32>(0.0, -1.0));
  return VertexOutput(vec4<f32>(corners[i], 0.0, 1.0), uv[i]);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let pixel = min(vec2<u32>(in.uv * vec2<f32>(image_size)), image_size - 1u);
  let slot = (pixel.y * image_size.x + pixel.x) * 3u;
  let light = vec3<f32>(f32(image[slot]), f32(image[slot + 1u]), f32(image[slot + 2u])) / 255.0;
  // Exposure for the mean particles per pixel keeps any count looking alike.
  let exposure = 1.5 / (1.0 + f32(COUNT) / f32(image_size.x * image_size.y));
  // Tone-map the brightest channel and scale the others with it, so crowds keep their hue.
  let peak = max(max(light.r, light.g), max(light.b, 1e-6));
  return vec4<f32>(light * (1.0 - exp(-peak * exposure)) / peak, 1.0);
}
`;

const PROGRAM = /* js */ `import { Pass, Shader } from "fragmentcolor";
import FIELD from "./field.wgsl?raw";
import CLEAR from "./clear.wgsl?raw";
import UPDATE from "./update.wgsl?raw";
import PRESENT from "./present.wgsl?raw";

const LIBRARY = ["gradient/palette_iq", "gradient/turbo", "hash/pcg2d"];

// Every shader sizes the shared buffers with these.
const COUNT = 1_000_000;
const ROWS = 96;
// Grid columns for the widest canvas, 4:1. A narrower one uses fewer.
const COLUMNS = ROWS * 4;
// Image pixels. A larger canvas renders the image at a lower resolution.
const PIXELS = 1 << 21;

export async function setup(renderer, target) {
  const header = [
    \`const COUNT: u32 = \${COUNT}u;\`,
    \`const ROWS: u32 = \${ROWS}u;\`,
    \`const COLUMNS: u32 = \${COLUMNS}u;\`,
    "const CELLS: u32 = ROWS * COLUMNS;",
    \`const PIXELS: u32 = \${PIXELS}u;\`,
  ].join("\\n");

  const field = await prepare("field", [header, FIELD]);
  // 8 matches @workgroup_size in field.wgsl.
  field.pass.setComputeDispatch(COLUMNS / 8, ROWS / 8, 1);

  const clear = await prepare("clear", [header, CLEAR]);

  // The library brings the color ramps and the hash that update.wgsl calls.
  const update = await prepare("update", [...LIBRARY, header, UPDATE]);
  update.shader.set("params.pull", 600);
  update.shader.set("params.heat", 20);
  update.shader.set("params.seed", 16);
  update.shader.set("params.mode", 0);
  // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.
  const groups = Math.ceil(COUNT / 256);
  update.pass.setComputeDispatch(Math.min(groups, 65535), Math.ceil(groups / 65535), 1);

  const present = await prepare("present", [header, PRESENT]);

  const stages = [field, clear, update, present];
  return {
    field: field.shader,
    update: update.shader,
    present: present.shader,
    clear: clear.pass,
    stages,
    passes: stages.map((stage) => stage.pass),
    size: null,
    frame: 0,
    pointer: null,
    moved: -Infinity,
  };
}

export function frame(state, renderer, target, time, size, pointer) {
  // This program draws, so later builds may reuse its shaders.
  for (const stage of state.stages) kept.set(stage.key, stage);
  const resized = size[0] !== state.size?.[0] || size[1] !== state.size?.[1];
  if (resized) fit(state, size);

  // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.
  // A resize can shift its pixel position while the pointer stays put.
  const [x, y] = pointer;
  const moved = state.pointer && (x !== state.pointer[0] || y !== state.pointer[1]);
  if (moved && !resized) state.moved = time;
  state.pointer = [x, y];

  state.update.set("params.mouse", [x / size[0], y / size[1]]);
  state.update.set("params.grip", Math.max(0, 1 - (time - state.moved) / 1.5));
  state.update.set("params.frame", state.frame++);
  renderer.render(state.passes, target);
}

// Fits the grid and the image to the canvas, so nothing stretches.
function fit(state, [width, height]) {
  const columns = Math.min(Math.max(Math.floor((ROWS * width) / height), 16), COLUMNS);
  let scale = 1;
  while (Math.floor(width / scale) * Math.floor(height / scale) > PIXELS) scale++;
  const image = [Math.max(Math.floor(width / scale), 1), Math.max(Math.floor(height / scale), 1)];

  state.field.set("columns", columns);
  state.update.set("params.columns", columns);
  state.update.set("params.image", image);
  state.present.set("image_size", image);
  // 256 matches @workgroup_size in clear.wgsl.
  const threads = Math.max(image[0] * image[1], ROWS * COLUMNS);
  state.clear.setComputeDispatch(Math.ceil(threads / 256), 1, 1);
  state.size = [width, height];
}

// Each shader holds a copy of every buffer it declares, about 40 MB for
// update.wgsl. A build reuses the shader and pass that an earlier program
// made from the same sources, if that one drew.
const kept = (globalThis.lifeShaders ??= new Map());

async function prepare(name, sources) {
  const key = JSON.stringify([name, ...sources]);
  if (kept.has(key)) return kept.get(key);
  const shader = await Shader.fetch(sources);
  return { key, shader, pass: Pass.fromShader(name, shader) };
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

// FragmentColor 0.12.5 rejects atomic<u32> storage buffers and shares uniforms
// across passes in one render call. The playground runs this demo from 0.12.6 on,
// and every generated program names that release.
const RELEASE = "0.12.6";
const NEEDS = `Needs FragmentColor ${RELEASE} or later.`;

function quote(text) {
  return JSON.stringify(String(text));
}

function kotlinQuote(text) {
  return quote(text).replace(/\$/g, "\\$");
}

// Keys under params. belong to update.wgsl. Any other key is taken as present.wgsl's.
function describe(model) {
  const values = (model.values ?? Object.entries(VALUES)).map(([key, value, scalar]) => [
    key,
    value,
    scalar ?? (UNSIGNED.has(key) ? "uint" : "float"),
  ]);
  const pick = (update) => values.filter(([key]) => key.startsWith("params.") === update);
  return { library: model.library ?? [], update: pick(true), present: pick(false) };
}

// `head(a, b)tail` on one line when it fits in `width`, otherwise one part per line.
function wrap(indent, head, parts, tail, width) {
  const line = `${indent}${head}${parts.join(", ")}${tail}`;
  if (line.length <= width) return [line];
  return [`${indent}${head}`, ...parts.map((part) => `${indent}    ${part},`), `${indent}${tail}`];
}

// One `set` call per value, spelled the way `lang` writes it.
function setCalls(lang, name, values, { before = "", after = "", text = quote } = {}) {
  return values.map(
    ([key, value, scalar]) => `    ${before}${name}.set(${text(key)}, ${formatValue(lang, value, scalar)})${after}`,
  );
}

function writeRust({ library, update, present }) {
  const sets = (name, values) => setCalls("rust", name, values, { after: "?;" });
  return [
    `// ${NEEDS}`,
    "",
    "use std::error::Error;",
    "",
    "use fragmentcolor::{Pass, Renderer, Shader, Target};",
    "",
    'const FIELD_SOURCE: &str = include_str!("field.wgsl");',
    'const CLEAR_SOURCE: &str = include_str!("clear.wgsl");',
    'const UPDATE_SOURCE: &str = include_str!("update.wgsl");',
    'const PRESENT_SOURCE: &str = include_str!("present.wgsl");',
    "",
    "// Every shader sizes the shared buffers with these.",
    "const COUNT: u32 = 1_000_000;",
    "const ROWS: u32 = 96;",
    "// Grid columns for the widest window, 4:1. A narrower one uses fewer.",
    "const COLUMNS: u32 = ROWS * 4;",
    "// Image pixels. A larger window renders the image at a lower resolution.",
    "const PIXELS: u32 = 1 << 21;",
    "",
    "struct State {",
    "    field: Shader,",
    "    update: Shader,",
    "    present: Shader,",
    "    clear: Pass,",
    "    passes: Vec<Pass>,",
    "    size: Option<[f32; 2]>,",
    "    frame: u32,",
    "    pointer: Option<[f32; 2]>,",
    "    moved: f32,",
    "}",
    "",
    "// Call setup once, then frame on every redraw of your window.",
    "fn setup() -> Result<State, Box<dyn Error>> {",
    "    let header = [",
    '        format!("const COUNT: u32 = {COUNT}u;"),',
    '        format!("const ROWS: u32 = {ROWS}u;"),',
    '        format!("const COLUMNS: u32 = {COLUMNS}u;"),',
    '        "const CELLS: u32 = ROWS * COLUMNS;".to_string(),',
    '        format!("const PIXELS: u32 = {PIXELS}u;"),',
    "    ]",
    '    .join("\\n");',
    "",
    "    let field = Shader::new([header.as_str(), FIELD_SOURCE])?;",
    "    let clear = Shader::new([header.as_str(), CLEAR_SOURCE])?;",
    "    // The library brings the color ramps and the hash that update.wgsl calls.",
    ...wrap("    ", "let update = Shader::new([", [...library.map(quote), "header.as_str()", "UPDATE_SOURCE"], "])?;", 100),
    ...sets("update", update),
    "    let present = Shader::new([header.as_str(), PRESENT_SOURCE])?;",
    ...sets("present", present),
    "",
    "    // 8 matches @workgroup_size in field.wgsl.",
    '    let field_pass = Pass::from_shader("field", &field);',
    "    field_pass.set_compute_dispatch(COLUMNS / 8, ROWS / 8, 1);",
    '    let clear_pass = Pass::from_shader("clear", &clear);',
    "    // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    let update_pass = Pass::from_shader("update", &update);',
    "    let groups = COUNT.div_ceil(256);",
    "    update_pass.set_compute_dispatch(groups.min(65535), groups.div_ceil(65535), 1);",
    '    let present_pass = Pass::from_shader("present", &present);',
    "",
    "    Ok(State {",
    "        passes: vec![field_pass, clear_pass.clone(), update_pass, present_pass],",
    "        field,",
    "        update,",
    "        present,",
    "        clear: clear_pass,",
    "        size: None,",
    "        frame: 0,",
    "        pointer: None,",
    "        moved: f32::NEG_INFINITY,",
    "    })",
    "}",
    "",
    "fn frame(",
    "    state: &mut State,",
    "    renderer: &Renderer,",
    "    target: &impl Target,",
    "    time: f32,",
    "    size: [f32; 2],",
    "    pointer: [f32; 2],",
    ") -> Result<(), Box<dyn Error>> {",
    "    let resized = state.size != Some(size);",
    "    if resized {",
    "        fit(state, size)?;",
    "    }",
    "",
    "    // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    // A resize can shift its pixel position while the pointer stays put.",
    "    if !resized && state.pointer.is_some_and(|last| last != pointer) {",
    "        state.moved = time;",
    "    }",
    "    state.pointer = Some(pointer);",
    "",
    "    let mouse = [pointer[0] / size[0], pointer[1] / size[1]];",
    '    state.update.set("params.mouse", mouse)?;',
    "    let grip = (1.0 - (time - state.moved) / 1.5).max(0.0);",
    '    state.update.set("params.grip", grip)?;',
    '    state.update.set("params.frame", state.frame)?;',
    "    state.frame += 1;",
    "    renderer.render(&state.passes, target)?;",
    "    Ok(())",
    "}",
    "",
    "// Fits the grid and the image to the window, so nothing stretches.",
    "fn fit(state: &mut State, size: [f32; 2]) -> Result<(), Box<dyn Error>> {",
    "    let width = (size[0] as u32).max(1);",
    "    let height = (size[1] as u32).max(1);",
    "    let columns = (ROWS * width / height).clamp(16, COLUMNS);",
    "    let scale = (1..)",
    "        .find(|scale| (width / scale) * (height / scale) <= PIXELS)",
    "        .unwrap_or(1);",
    "    let image = [(width / scale).max(1), (height / scale).max(1)];",
    "",
    '    state.field.set("columns", columns)?;',
    '    state.update.set("params.columns", columns)?;',
    '    state.update.set("params.image", image)?;',
    '    state.present.set("image_size", image)?;',
    "    // 256 matches @workgroup_size in clear.wgsl.",
    "    let threads = (image[0] * image[1]).max(ROWS * COLUMNS);",
    "    let groups = threads.div_ceil(256);",
    "    state.clear.set_compute_dispatch(groups, 1, 1);",
    "    state.size = Some(size);",
    "    Ok(())",
    "}",
    "",
  ].join("\n");
}

function writePython({ library, update, present }) {
  const sets = (name, values) => setCalls("python", name, values);
  return [
    `# ${NEEDS}`,
    "",
    "import math",
    "from pathlib import Path",
    "from types import SimpleNamespace",
    "",
    "from fragmentcolor import Pass, Shader",
    "",
    ...wrap("", "LIBRARY = [", library.map(quote), "]", 88),
    'FIELD_SOURCE = Path(__file__).with_name("field.wgsl").read_text()',
    'CLEAR_SOURCE = Path(__file__).with_name("clear.wgsl").read_text()',
    'UPDATE_SOURCE = Path(__file__).with_name("update.wgsl").read_text()',
    'PRESENT_SOURCE = Path(__file__).with_name("present.wgsl").read_text()',
    "",
    "# Every shader sizes the shared buffers with these.",
    "COUNT = 1_000_000",
    "ROWS = 96",
    "# Grid columns for the widest canvas, 4:1. A narrower one uses fewer.",
    "COLUMNS = ROWS * 4",
    "# Image pixels. A larger canvas renders the image at a lower resolution.",
    "PIXELS = 1 << 21",
    "",
    "",
    "# Call setup() once, then frame() on every redraw of your canvas.",
    "def setup():",
    '    header = "\\n".join(',
    "        [",
    '            f"const COUNT: u32 = {COUNT}u;",',
    '            f"const ROWS: u32 = {ROWS}u;",',
    '            f"const COLUMNS: u32 = {COLUMNS}u;",',
    '            "const CELLS: u32 = ROWS * COLUMNS;",',
    '            f"const PIXELS: u32 = {PIXELS}u;",',
    "        ]",
    "    )",
    "",
    "    field = Shader([header, FIELD_SOURCE])",
    "    clear = Shader([header, CLEAR_SOURCE])",
    "    # The library brings the color ramps and the hash that update.wgsl calls.",
    "    update = Shader([*LIBRARY, header, UPDATE_SOURCE])",
    ...sets("update", update),
    "    present = Shader([header, PRESENT_SOURCE])",
    ...sets("present", present),
    "",
    "    # 8 matches @workgroup_size in field.wgsl.",
    '    field_pass = Pass.from_shader("field", field)',
    "    field_pass.set_compute_dispatch(COLUMNS // 8, ROWS // 8, 1)",
    '    clear_pass = Pass.from_shader("clear", clear)',
    "    # 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    update_pass = Pass.from_shader("update", update)',
    "    groups = math.ceil(COUNT / 256)",
    "    update_pass.set_compute_dispatch(min(groups, 65535), math.ceil(groups / 65535), 1)",
    '    present_pass = Pass.from_shader("present", present)',
    "",
    "    return SimpleNamespace(",
    "        field=field,",
    "        update=update,",
    "        present=present,",
    "        clear=clear_pass,",
    "        passes=[field_pass, clear_pass, update_pass, present_pass],",
    "        size=None,",
    "        frame=0,",
    "        pointer=None,",
    "        moved=-math.inf,",
    "    )",
    "",
    "",
    "def frame(state, renderer, target, time, size, pointer):",
    "    resized = state.size != list(size)",
    "    if resized:",
    "        fit(state, size)",
    "",
    "    # The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    # A resize can shift its pixel position while the pointer stays put.",
    "    if not resized and state.pointer is not None and list(pointer) != state.pointer:",
    "        state.moved = time",
    "    state.pointer = list(pointer)",
    "",
    '    state.update.set("params.mouse", [pointer[0] / size[0], pointer[1] / size[1]])',
    '    state.update.set("params.grip", max(0.0, 1 - (time - state.moved) / 1.5))',
    '    state.update.set("params.frame", state.frame)',
    "    state.frame += 1",
    "    renderer.render(state.passes, target)",
    "",
    "",
    "# Fits the grid and the image to the canvas, so nothing stretches.",
    "def fit(state, size):",
    "    width = max(int(size[0]), 1)",
    "    height = max(int(size[1]), 1)",
    "    columns = min(max(ROWS * width // height, 16), COLUMNS)",
    "    scale = 1",
    "    while (width // scale) * (height // scale) > PIXELS:",
    "        scale += 1",
    "    image = [max(width // scale, 1), max(height // scale, 1)]",
    "",
    '    state.field.set("columns", columns)',
    '    state.update.set("params.columns", columns)',
    '    state.update.set("params.image", image)',
    '    state.present.set("image_size", image)',
    "    # 256 matches @workgroup_size in clear.wgsl.",
    "    threads = max(image[0] * image[1], ROWS * COLUMNS)",
    "    state.clear.set_compute_dispatch(math.ceil(threads / 256), 1, 1)",
    "    state.size = list(size)",
    "",
  ].join("\n");
}

function writeSwift({ library, update, present }) {
  const sets = (name, values) => setCalls("swift", name, values, { before: "try " });
  const libraryLine = library.length
    ? wrap("", "let library = [", library.map(quote), "]", 100)
    : ["let library: [String] = []"];
  return [
    `// ${NEEDS}`,
    "",
    "import Foundation",
    "import FragmentColor",
    "",
    ...libraryLine,
    "",
    "// Every shader sizes the shared buffers with these.",
    "let particleCount: UInt32 = 1_000_000",
    "let gridRows: UInt32 = 96",
    "// Grid columns for the widest window, 4:1. A narrower one uses fewer.",
    "let gridColumns: UInt32 = gridRows * 4",
    "// Image pixels. A larger window renders the image at a lower resolution.",
    "let imagePixels: UInt32 = 1 << 21",
    "",
    "struct State {",
    "    let field: Shader",
    "    let update: Shader",
    "    let present: Shader",
    "    let clear: Pass",
    "    let passes: [Pass]",
    "    var size: [Float]?",
    "    var frame: UInt32 = 0",
    "    var pointer: [Float]?",
    "    var moved = -Float.infinity",
    "}",
    "",
    "// Reads a WGSL file bundled with the app.",
    "func readSource(_ name: String) throws -> String {",
    '    let url = Bundle.main.url(forResource: name, withExtension: "wgsl")!',
    "    return try String(contentsOf: url, encoding: .utf8)",
    "}",
    "",
    "// Call setup once, then frame from your MTKView or CADisplayLink callback.",
    "func setup() throws -> State {",
    '    let header = """',
    "        const COUNT: u32 = \\(particleCount)u;",
    "        const ROWS: u32 = \\(gridRows)u;",
    "        const COLUMNS: u32 = \\(gridColumns)u;",
    "        const CELLS: u32 = ROWS * COLUMNS;",
    "        const PIXELS: u32 = \\(imagePixels)u;",
    '        """',
    "",
    '    let field = try Shader([header, readSource("field")])',
    '    let clear = try Shader([header, readSource("clear")])',
    "    // The library brings the color ramps and the hash that update.wgsl calls.",
    '    let update = try Shader(library + [header, readSource("update")])',
    ...sets("update", update),
    '    let present = try Shader([header, readSource("present")])',
    ...sets("present", present),
    "",
    "    // 8 matches @workgroup_size in field.wgsl.",
    '    let fieldPass = Pass.fromShader("field", field)',
    "    fieldPass.setComputeDispatch(gridColumns / 8, gridRows / 8, 1)",
    '    let clearPass = Pass.fromShader("clear", clear)',
    "    // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    let updatePass = Pass.fromShader("update", update)',
    "    let groups = (particleCount + 255) / 256",
    "    updatePass.setComputeDispatch(min(groups, 65535), (groups + 65534) / 65535, 1)",
    '    let presentPass = Pass.fromShader("present", present)',
    "",
    "    return State(",
    "        field: field,",
    "        update: update,",
    "        present: present,",
    "        clear: clearPass,",
    "        passes: [fieldPass, clearPass, updatePass, presentPass]",
    "    )",
    "}",
    "",
    "func frame(",
    "    state: inout State,",
    "    renderer: Renderer,",
    "    target: WindowTarget,",
    "    time: Float,",
    "    size: [Float],",
    "    pointer: [Float]",
    ") throws {",
    "    let resized = state.size != size",
    "    if resized {",
    "        try fit(&state, size)",
    "    }",
    "",
    "    // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    // A resize can shift its pixel position while the pointer stays put.",
    "    if !resized, let last = state.pointer, last != pointer {",
    "        state.moved = time",
    "    }",
    "    state.pointer = pointer",
    "",
    '    try state.update.set("params.mouse", [pointer[0] / size[0], pointer[1] / size[1]])',
    '    try state.update.set("params.grip", max(0, 1 - (time - state.moved) / 1.5))',
    '    try state.update.set("params.frame", state.frame)',
    "    state.frame += 1",
    "    try renderer.render(state.passes, target)",
    "}",
    "",
    "// Fits the grid and the image to the window, so nothing stretches.",
    "func fit(_ state: inout State, _ size: [Float]) throws {",
    "    let width = max(UInt32(size[0]), 1)",
    "    let height = max(UInt32(size[1]), 1)",
    "    let columns = min(max(gridRows * width / height, 16), gridColumns)",
    "    var scale: UInt32 = 1",
    "    while (width / scale) * (height / scale) > imagePixels {",
    "        scale += 1",
    "    }",
    "    let image = [max(width / scale, 1), max(height / scale, 1)]",
    "",
    '    try state.field.set("columns", columns)',
    '    try state.update.set("params.columns", columns)',
    '    try state.update.set("params.image", image)',
    '    try state.present.set("image_size", image)',
    "    // 256 matches @workgroup_size in clear.wgsl.",
    "    let threads = max(image[0] * image[1], gridRows * gridColumns)",
    "    state.clear.setComputeDispatch((threads + 255) / 256, 1, 1)",
    "    state.size = size",
    "}",
    "",
  ].join("\n");
}

function writeKotlin({ library, update, present }) {
  const sets = (name, values) => setCalls("kotlin", name, values, { text: kotlinQuote });
  const libraryLine = library.length
    ? wrap("", "private val LIBRARY = listOf(", library.map(kotlinQuote), ")", 100)
    : ["private val LIBRARY = emptyList<String>()"];
  return [
    `// ${NEEDS}`,
    "",
    "import android.content.res.AssetManager",
    "import org.fragmentcolor.*",
    "",
    ...libraryLine,
    "",
    "// Every shader sizes the shared buffers with these.",
    "private const val COUNT = 1_000_000u",
    "private const val ROWS = 96u",
    "",
    "// Grid columns for the widest window, 4:1. A narrower one uses fewer.",
    "private val COLUMNS = ROWS * 4u",
    "",
    "// Image pixels. A larger window renders the image at a lower resolution.",
    "private val PIXELS = 1u shl 21",
    "",
    "class State(",
    "    val field: Shader,",
    "    val update: Shader,",
    "    val present: Shader,",
    "    val clear: Pass,",
    "    val passes: List<Pass>,",
    ") {",
    "    var size: List<Float>? = null",
    "    var frame = 0u",
    "    var pointer: List<Float>? = null",
    "    var moved = Float.NEGATIVE_INFINITY",
    "}",
    "",
    "// Call setup once, then frame from your Choreographer or render thread callback.",
    "fun setup(assets: AssetManager): State {",
    "    fun read(name: String) = assets.open(name).bufferedReader().use { it.readText() }",
    "    val header =",
    "        listOf(",
    '            "const COUNT: u32 = ${COUNT}u;",',
    '            "const ROWS: u32 = ${ROWS}u;",',
    '            "const COLUMNS: u32 = ${COLUMNS}u;",',
    '            "const CELLS: u32 = ROWS * COLUMNS;",',
    '            "const PIXELS: u32 = ${PIXELS}u;",',
    '        ).joinToString("\\n")',
    "",
    '    val field = Shader(listOf(header, read("field.wgsl")))',
    '    val clear = Shader(listOf(header, read("clear.wgsl")))',
    "    // The library brings the color ramps and the hash that update.wgsl calls.",
    '    val update = Shader(LIBRARY + listOf(header, read("update.wgsl")))',
    ...sets("update", update),
    '    val present = Shader(listOf(header, read("present.wgsl")))',
    ...sets("present", present),
    "",
    "    // 8 matches @workgroup_size in field.wgsl.",
    '    val fieldPass = Pass.fromShader("field", field)',
    "    fieldPass.setComputeDispatch(COLUMNS / 8u, ROWS / 8u, 1u)",
    '    val clearPass = Pass.fromShader("clear", clear)',
    "    // 256 matches @workgroup_size in update.wgsl. A row holds at most 65535 groups.",
    '    val updatePass = Pass.fromShader("update", update)',
    "    val groups = (COUNT + 255u) / 256u",
    "    updatePass.setComputeDispatch(minOf(groups, 65535u), (groups + 65534u) / 65535u, 1u)",
    '    val presentPass = Pass.fromShader("present", present)',
    "",
    "    val passes = listOf(fieldPass, clearPass, updatePass, presentPass)",
    "    return State(field, update, present, clearPass, passes)",
    "}",
    "",
    "fun frame(",
    "    state: State,",
    "    renderer: Renderer,",
    "    target: WindowTarget,",
    "    time: Float,",
    "    size: List<Float>,",
    "    pointer: List<Float>,",
    ") {",
    "    val resized = state.size != size",
    "    if (resized) fit(state, size)",
    "",
    "    // The pointer pulls while it moves and lets go over 1.5 seconds once it rests.",
    "    // A resize can shift its pixel position while the pointer stays put.",
    "    if (!resized && state.pointer != null && state.pointer != pointer) state.moved = time",
    "    state.pointer = pointer",
    "",
    '    state.update.set("params.mouse", listOf(pointer[0] / size[0], pointer[1] / size[1]))',
    '    state.update.set("params.grip", maxOf(0f, 1f - (time - state.moved) / 1.5f))',
    '    state.update.set("params.frame", state.frame)',
    "    state.frame++",
    "    renderer.render(state.passes, target)",
    "}",
    "",
    "// Fits the grid and the image to the window, so nothing stretches.",
    "fun fit(",
    "    state: State,",
    "    size: List<Float>,",
    ") {",
    "    val width = maxOf(size[0].toUInt(), 1u)",
    "    val height = maxOf(size[1].toUInt(), 1u)",
    "    val columns = (ROWS * width / height).coerceIn(16u, COLUMNS)",
    "    var scale = 1u",
    "    while ((width / scale) * (height / scale) > PIXELS) scale++",
    "    val image = listOf(maxOf(width / scale, 1u), maxOf(height / scale, 1u))",
    "",
    '    state.field.set("columns", columns)',
    '    state.update.set("params.columns", columns)',
    '    state.update.set("params.image", image)',
    '    state.present.set("image_size", image)',
    "    // 256 matches @workgroup_size in clear.wgsl.",
    "    val threads = maxOf(image[0] * image[1], ROWS * COLUMNS)",
    "    state.clear.setComputeDispatch((threads + 255u) / 256u, 1u, 1u)",
    "    state.size = size",
    "}",
    "",
  ].join("\n");
}

export default {
  id: "life",
  title: "Life",
  summary:
    "Four species follow or flee each other's density and gather into colonies that chase, split, and swirl. The pointer draws them close, and seed reshuffles who follows whom.",
  category: "Compute",
  // A held pointer gathers the swarm into a ball: ./fc capture thumbs life --hold 0.5,0.5 --wait 10
  thumbnail: "/playground/thumbs/life.webp",
  chip: { x: 0.49, y: 0.49, zoom: 6 },
  kind: "compute",
  requires: "webgpu",
  needs: RELEASE,
  poster: "/playground/life.webp",
  library: ["gradient/palette_iq", "gradient/turbo", "hash/pcg2d"],
  values: VALUES,
  files: { "field.wgsl": FIELD, "clear.wgsl": CLEAR, "update.wgsl": UPDATE, "present.wgsl": PRESENT },
  js: PROGRAM,
  code(model) {
    const spec = describe(model);
    return {
      rust: writeRust(spec),
      python: writePython(spec),
      swift: writeSwift(spec),
      kotlin: writeKotlin(spec),
    };
  },
};
