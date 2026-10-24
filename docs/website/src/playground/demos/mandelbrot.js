// Infinite zoom into the Mandelbrot set. main.js computes one reference orbit
// with BigInt at full precision, a compute pass iterates every pixel as a small
// f32 difference from it, and a render pass colors the counts. The tutorial at
// docs/tutorials/infinite-zoom walks through the same program.

import { formatValue } from "../codegen.js";

const ITERATE = /* wgsl */ `// Iterates every pixel as a small difference from the reference orbit, which
// the host computes at the precision the deepest view needs. The host puts the
// ORBIT and PIXELS constants ahead of this code, and the present shader reads
// the counts this one writes.

struct View {
  image: vec2<u32>,
  // A pixel is fraction * 2^exponent wide.
  exponent: i32,
  fraction: f32,
  // Iterations the series covers, and the last index of the orbit.
  skip: u32,
  length: u32,
  budget: u32, // 100..5000
  // Series coefficients at skip, each a * 2^a_exponent.
  a: vec2<f32>,
  a_exponent: i32,
  b: vec2<f32>,
  b_exponent: i32,
  c: vec2<f32>,
  c_exponent: i32,
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage, read> orbit: array<vec2<f32>, ORBIT>;
@group(0) @binding(2) var<storage, read_write> counts: array<f32, PIXELS>;

// The squared size past which an orbit always escapes: a radius of 256.
const ESCAPE: f32 = 65536.0;

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// x * 2^k with k capped at 127, or zero once 2^k is too small for an f32.
fn scale(x: vec2<f32>, k: i32) -> vec2<f32> {
  if (k < -126) {
    return vec2<f32>(0.0);
  }
  return ldexp(x, vec2<i32>(min(k, 127)));
}

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= view.image.x || id.y >= view.image.y) {
    return;
  }
  // How far this pixel sits from the reference, in units of 2^exponent.
  let half = vec2<f32>(view.image) * 0.5;
  let d = (vec2<f32>(id.xy) + 0.5 - half) * vec2<f32>(1.0, -1.0) * view.fraction;
  let e = view.exponent;
  let index = id.y * view.image.x + id.x;

  // In a wide view, the main cardioid and the bulb left of it cover many pixels
  // that never escape. A direct test skips their loop. orbit[1] is the target's c.
  if (e > -20) {
    let c = orbit[1] + scale(d, e);
    let q = dot(c - vec2<f32>(0.25, 0.0), c - vec2<f32>(0.25, 0.0));
    let bulb = c + vec2<f32>(1.0, 0.0);
    if (q * (q + c.x - 0.25) < 0.25 * c.y * c.y || dot(bulb, bulb) < 0.0625) {
      counts[index] = -1.0;
      return;
    }
  }

  // The series moves every pixel to iteration skip in one step. From there on,
  // the pixel's difference from the reference is w * 2^s, with w kept near 1.
  let d2 = mul(d, d);
  let ea = view.a_exponent + e;
  let eb = view.b_exponent + 2 * e;
  let ec = view.c_exponent + 3 * e;
  var s = max(ea, max(eb, ec));
  var w = scale(mul(view.a, d), ea - s) + scale(mul(view.b, d2), eb - s) + scale(mul(view.c, mul(d2, d)), ec - s);
  var m = view.skip; // where the pixel is along the reference orbit
  var n = view.skip; // the pixel's own iteration count
  var reference = orbit[m];
  // 2^s and the pixel's offset in units of 2^s change only with s.
  var unit = scale(vec2<f32>(1.0), s).x;
  var offset = scale(d, e - s);
  let limit = view.skip + view.budget;
  var size2 = 0.0;
  while (n < limit) {
    let size = max(abs(w.x), abs(w.y));
    if (size > 65536.0 || size < 1.0 / 65536.0) {
      let k = frexp(size).exp;
      w = scale(w, -k);
      s += k;
      unit = scale(vec2<f32>(1.0), s).x;
      offset = scale(d, e - s);
    }
    if (m == 0u) {
      // The reference is at 0 here, so only the square and the offset remain.
      let t = max(2 * s, e);
      w = scale(mul(w, w), 2 * s - t) + scale(d, e - t);
      s = t;
      unit = scale(vec2<f32>(1.0), s).x;
      offset = scale(d, e - s);
    } else {
      // z becomes 2 Z z + z^2 + d * 2^e, with every term divided by 2^s.
      w = 2.0 * mul(reference, w) + mul(w, w) * unit + offset;
    }
    m += 1u;
    n += 1u;
    reference = orbit[m];
    let z = w * unit;
    let full = reference + z;
    size2 = dot(full, full);
    if (size2 > ESCAPE) {
      break;
    }
    // Rebase when the pixel is closer to 0 than to the reference, or reaches the end of the orbit.
    if (size2 < dot(z, z) || m >= view.length) {
      let k = frexp(max(abs(full.x), abs(full.y))).exp;
      w = scale(full, -k);
      s = k;
      m = 0u;
      reference = vec2<f32>(0.0);
      unit = scale(vec2<f32>(1.0), s).x;
      offset = scale(d, e - s);
    }
  }
  // A smooth count: the distance past ESCAPE sets the fraction.
  var count = -1.0;
  if (n < limit) {
    count = f32(n) + 1.0 - log2(log2(size2) / log2(ESCAPE));
  }
  counts[index] = count;
}
`;

const PRESENT = /* wgsl */ `// Colors the counts that the iterate shader writes, and prints the depth with
// print_depth from the label shader.

struct Look {
  resolution: vec2<f32>,
  image: vec2<u32>,
  // The view is about 10^-depth tall.
  depth: f32,
  density: f32, // 0.002..0.1
  phase: f32, // 0..1
  inside_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> look: Look;
@group(0) @binding(1) var<storage, read> counts: array<f32, PIXELS>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let cell = min(vec2<u32>(position.xy / look.resolution * vec2<f32>(look.image)), look.image - 1u);
  let count = counts[cell.y * look.image.x + cell.x];
  var color = look.inside_color;
  if (count >= 0.0) {
    color = palette_iq(
      count * look.density + look.phase,
      vec3<f32>(0.5),
      vec3<f32>(0.5),
      vec3<f32>(1.0),
      vec3<f32>(0.0, 0.1, 0.2),
    );
  }
  return vec4<f32>(print_depth(color, position.xy, look.resolution, look.depth), 1.0);
}

// One triangle larger than the target covers every pixel; the GPU clips the rest.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  var corners = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
  );
  return vec4<f32>(corners[i], 0.0, 1.0);
}
`;

const LABEL = /* wgsl */ `// Prints "1e-<depth>" in the bottom left corner: white with a dark shadow.
fn print_depth(color: vec3<f32>, position: vec2<f32>, resolution: vec2<f32>, depth: f32) -> vec3<f32> {
  let unit = max(floor(resolution.y / 160.0), 2.0);
  let shadow = mix(color, vec3<f32>(0.0), label(position - unit, resolution, depth));
  return mix(shadow, vec3<f32>(1.0), label(position, resolution, depth));
}

// 1 where the label covers this pixel. Each glyph is 3 by 5 cells, one bit per
// cell, top row first: 0 to 9, then "e" and "-".
fn label(position: vec2<f32>, resolution: vec2<f32>, depth: f32) -> f32 {
  var glyphs = array<u32, 12>(
    0x7B6Fu, 0x2C97u, 0x73E7u, 0x73CFu, 0x5BC9u, 0x79CFu,
    0x79EFu, 0x7249u, 0x7BEFu, 0x7BCFu, 0x2BE3u, 0x01C0u,
  );
  let unit = max(floor(resolution.y / 160.0), 2.0);
  let origin = vec2<f32>(unit * 4.0, resolution.y - unit * 9.0);
  let cell = vec2<i32>(floor((position - origin) / unit));
  if (cell.x < 0 || cell.y < 0 || cell.y > 4 || cell.x % 4 == 3) {
    return 0.0;
  }
  let exponent = u32(round(max(depth, 0.0)));
  var digits = 1u;
  var power = 1u;
  while (power * 10u <= exponent) {
    power *= 10u;
    digits += 1u;
  }
  // "1", "e", then "-" before any exponent above 0, then the digits.
  let sign = select(0u, 1u, exponent > 0u);
  let slot = u32(cell.x / 4);
  var glyph = 0u;
  if (slot == 0u) {
    glyph = 1u;
  } else if (slot == 1u) {
    glyph = 10u;
  } else if (slot == 2u && sign == 1u) {
    glyph = 11u;
  } else if (slot < 2u + sign + digits) {
    var divisor = 1u;
    for (var i = slot; i < 1u + sign + digits; i++) {
      divisor *= 10u;
    }
    glyph = exponent / divisor % 10u;
  } else {
    return 0.0;
  }
  let bit = 14u - u32(cell.y * 3 + cell.x % 4);
  return f32((glyphs[glyph] >> bit) & 1u);
}
`;

const PROGRAM = /* js */ `import { Pass, Shader } from "fragmentcolor";
import ITERATE from "./iterate.wgsl?raw";
import PRESENT from "./present.wgsl?raw";
import LABEL from "./label.wgsl?raw";

const LIBRARY = ["gradient/palette_iq"];

// A Misiurewicz point in Seahorse Valley: the orbit of 0 lands on a cycle of
// period 2 after 24 steps. Newton's method refines these digits to the precision DEPTH needs.
const TARGET = { re: "-0.7432918908524302", im: "0.1312405523087976", preperiod: 24, period: 2 };
// The zoom goes in until the view is 10^-DEPTH tall, then back out.
const DEPTH = 1000;
// Zoom speed, in doublings per second.
const SPEED = 1.5;
// Height of the first view.
const START = 3;
// Room for the reference orbit, and for the pixels the compute pass iterates.
// A larger canvas renders at a lower resolution.
const ORBIT = 1 << 16;
const PIXELS = 1 << 21;

export async function setup(renderer, target) {
  const header = \`const ORBIT: u32 = \${ORBIT}u;\\nconst PIXELS: u32 = \${PIXELS}u;\`;

  const iterate = await prepare("iterate", [header, ITERATE]);
  iterate.shader.set("view.budget", 1000);

  // The library brings the palette that present.wgsl calls.
  const present = await prepare("present", [...LIBRARY, header, LABEL, PRESENT]);
  present.shader.set("look.density", 0.02);
  present.shader.set("look.phase", 0.6);
  present.shader.set("look.inside_color", [0.02, 0.02, 0.05]);

  // Compute one orbit at the precision the deepest view needs and send it to the GPU once.
  const reference = referenceFor(TARGET, DEPTH);
  iterate.shader.set("orbit", new Uint8Array(reference.orbit.buffer));

  const stages = [iterate, present];
  return {
    iterate: iterate.shader,
    present: present.shader,
    compute: iterate.pass,
    stages,
    passes: stages.map((stage) => stage.pass),
    reference,
    zoom: 0,
    direction: 1,
    last: null,
    pointer: null,
    moved: -Infinity,
  };
}

export function frame(state, renderer, target, time, size, pointer) {
  // This program draws, so later builds may reuse its shaders.
  for (const stage of state.stages) kept.set(stage.key, stage);
  const zoom = steer(state, time, size, pointer);

  // The compute pass fills at most PIXELS: the canvas, or the canvas with each side divided by a whole number.
  let shrink = 1;
  while (Math.floor(size[0] / shrink) * Math.floor(size[1] / shrink) > PIXELS) shrink++;
  const image = [Math.max(Math.floor(size[0] / shrink), 1), Math.max(Math.floor(size[1] / shrink), 1)];

  // A pixel is 2^pixel wide. The series covers every pixel within 2^radius of the target.
  const pixel = Math.log2(START / image[1]) - zoom;
  const exponent = Math.floor(pixel);
  const radius = pixel + Math.log2(Math.hypot(image[0], image[1]) / 2);
  const skip = skipFor(state.reference, radius);
  const [a, b, c] = state.reference.series[skip].map(exact);

  state.iterate.set("view.image", image);
  state.iterate.set("view.exponent", exponent);
  state.iterate.set("view.fraction", 2 ** (pixel - exponent));
  state.iterate.set("view.skip", skip);
  state.iterate.set("view.length", state.reference.length);
  state.iterate.set("view.a", [a[0], a[1]]);
  state.iterate.set("view.a_exponent", a[2]);
  state.iterate.set("view.b", [b[0], b[1]]);
  state.iterate.set("view.b_exponent", b[2]);
  state.iterate.set("view.c", [c[0], c[1]]);
  state.iterate.set("view.c_exponent", c[2]);
  // 8 matches @workgroup_size in iterate.wgsl.
  state.compute.setComputeDispatch(Math.ceil(image[0] / 8), Math.ceil(image[1] / 8), 1);

  state.present.set("look.resolution", size);
  state.present.set("look.image", image);
  state.present.set("look.depth", zoom * Math.log10(2) - Math.log10(START));
  renderer.render(state.passes, target);
}

// The zoom runs from the whole set down to 10^-DEPTH and back on its own. While
// the pointer moves, and for two seconds after, the pointer's vertical position
// picks the depth, deepest at the top. Returns the zoom in doublings.
function steer(state, time, size, [x, y]) {
  // A clock that starts over, as when a paused page resumes, forgets the pointer.
  if (state.last !== null && time < state.last) state.moved = -Infinity;
  const delta = state.last === null ? 0 : Math.min(Math.max(time - state.last, 0), 0.1);
  state.last = time;
  const deepest = DEPTH * Math.log2(10) + Math.log2(START);
  if (state.pointer && (x !== state.pointer[0] || y !== state.pointer[1])) state.moved = time;
  state.pointer = [x, y];
  if (time - state.moved < 2) {
    const goal = (1 - y / size[1]) * deepest;
    state.zoom += (goal - state.zoom) * Math.min(1, delta * 4);
  } else {
    state.zoom += state.direction * SPEED * delta;
    if (state.zoom >= deepest) state.direction = -1;
    if (state.zoom <= 0) state.direction = 1;
  }
  state.zoom = Math.min(Math.max(state.zoom, 0), deepest);
  return state.zoom;
}

// The reference takes a moment at 1000 digits, so builds share it while TARGET
// and DEPTH stay the same.
const orbits = (globalThis.mandelbrotOrbits ??= new Map());

function referenceFor(target, depth) {
  const key = JSON.stringify([target, depth]);
  if (!orbits.has(key)) {
    orbits.clear();
    orbits.set(key, computeReference(target, depth));
  }
  return orbits.get(key);
}

// The orbit of 0 under z^2 + c at full precision, rounded to f32 for the GPU,
// with Martin's series coefficients A, B and C at every iteration. The reach array holds
// the log2 of the widest radius the series still covers at each iteration.
function computeReference(target, depth) {
  const bits = Math.ceil(depth * Math.log2(10)) + 80;
  const shift = BigInt(bits);
  const [cr, ci] = refine(target, bits);
  const orbit = new Float32Array(2 * ORBIT);
  const series = [];
  const reach = new Float64Array(ORBIT).fill(Infinity);
  let [zr, zi] = [0n, 0n];
  let [a, b, c, d] = [ZERO, ZERO, ZERO, ZERO];
  for (let n = 0; ; n++) {
    const [re, im] = [toFloat(zr, bits), toFloat(zi, bits)];
    orbit[2 * n] = re;
    orbit[2 * n + 1] = im;
    series.push([a, b, c]);
    // The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
    if (n > 0) reach[n] = Math.min(reach[n - 1], (Math.log2(TOLERANCE) + log2Size(a) - log2Size(d)) / 3);
    // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
    if (n === ORBIT - 1 || re * re + im * im > 4 || log2Size(a) > bits - 40) {
      return { orbit, series, reach, length: n };
    }
    // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
    d = add(times(d, 2 * re, 2 * im), add(times(multiply(a, c), 2, 0), multiply(b, b)));
    c = add(times(c, 2 * re, 2 * im), times(multiply(a, b), 2, 0));
    b = add(times(b, 2 * re, 2 * im), multiply(a, a));
    a = add(times(a, 2 * re, 2 * im), ONE);
    const next = ((zr * zr - zi * zi) >> shift) + cr;
    zi = ((2n * zr * zi) >> shift) + ci;
    zr = next;
  }
}

// The largest the first dropped term of the series may grow, relative to the first term.
const TOLERANCE = 1e-7;

// The last iteration the series covers for a view of this radius (log2).
function skipFor(reference, radius) {
  let [low, high] = [0, reference.length];
  while (low < high) {
    const middle = (low + high + 1) >> 1;
    if (reference.reach[middle] > radius) low = middle;
    else high = middle - 1;
  }
  return low;
}

// A BigInt n stands for the real number n / 2^bits.
function parse(text, bits) {
  const [whole, fraction = ""] = text.replace("-", "").split(".");
  const value = (BigInt(whole + fraction) << BigInt(bits)) / 10n ** BigInt(fraction.length);
  return text.startsWith("-") ? -value : value;
}

function toFloat(value, bits) {
  return Number(value >> BigInt(bits - 60)) / 2 ** 60;
}

// Newton's method on z(preperiod + period) - z(preperiod) moves c until the orbit
// of 0 enters its cycle, to the working precision. Each round doubles the correct digits.
function refine(target, bits) {
  const shift = BigInt(bits);
  const one = 1n << shift;
  let re = parse(target.re, bits);
  let im = parse(target.im, bits);
  for (let round = 0; round < 64; round++) {
    let [zr, zi, dr, di] = [0n, 0n, 0n, 0n];
    let [kr, ki, kdr, kdi] = [0n, 0n, 0n, 0n];
    for (let n = 0; n < target.preperiod + target.period; n++) {
      if (n === target.preperiod) [kr, ki, kdr, kdi] = [zr, zi, dr, di];
      // dz/dc becomes 2 z dz/dc + 1, then z becomes z^2 + c.
      [dr, di] = [((2n * (zr * dr - zi * di)) >> shift) + one, (2n * (zr * di + zi * dr)) >> shift];
      [zr, zi] = [((zr * zr - zi * zi) >> shift) + re, ((2n * zr * zi) >> shift) + im];
    }
    const [fr, fi, gr, gi] = [zr - kr, zi - ki, dr - kdr, di - kdi];
    const size = (gr * gr + gi * gi) >> shift;
    if (size === 0n) break;
    const sr = (((fr * gr + fi * gi) >> shift) << shift) / size;
    const si = (((fi * gr - fr * gi) >> shift) << shift) / size;
    re -= sr;
    im -= si;
    if ((sr < 0n ? -sr : sr) + (si < 0n ? -si : si) < 256n) break;
  }
  return [re, im];
}

// Complex numbers beyond the range of a double: [re, im, exponent] = (re + i im) * 2^exponent.
const ZERO = [0, 0, 0];
const ONE = [1, 0, 0];

function normalize(re, im, exponent) {
  const size = Math.max(Math.abs(re), Math.abs(im));
  if (size === 0) return ZERO;
  if (size > 2 ** -64 && size < 2 ** 64) return [re, im, exponent];
  const k = Math.round(Math.log2(size));
  return [re * 2 ** -k, im * 2 ** -k, exponent + k];
}

// The same number with re and im below 1, as the shader expects.
function exact([re, im, exponent]) {
  const size = Math.max(Math.abs(re), Math.abs(im));
  if (size === 0) return ZERO;
  const k = Math.floor(Math.log2(size)) + 1;
  return [re * 2 ** -k, im * 2 ** -k, exponent + k];
}

function add(a, b) {
  const exponent = Math.max(a[2], b[2]);
  const [sa, sb] = [2 ** (a[2] - exponent), 2 ** (b[2] - exponent)];
  return normalize(a[0] * sa + b[0] * sb, a[1] * sa + b[1] * sb, exponent);
}

function multiply(a, b) {
  return normalize(a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0], a[2] + b[2]);
}

// a * (re + i im), for a plain complex number.
function times(a, re, im) {
  return normalize(a[0] * re - a[1] * im, a[0] * im + a[1] * re, a[2]);
}

function log2Size(a) {
  return Math.log2(Math.hypot(a[0], a[1])) + a[2];
}

// Each shader holds a copy of every buffer it declares, 8 MB for counts. A build
// reuses the shader and pass that an earlier program made from the same sources,
// if that one drew.
const kept = (globalThis.mandelbrotShaders ??= new Map());

async function prepare(name, sources) {
  const key = JSON.stringify([name, ...sources]);
  if (kept.has(key)) return kept.get(key);
  const shader = await Shader.fetch(sources);
  return { key, shader, pass: Pass.fromShader(name, shader) };
}
`;

// The values main.js sets in setup, in order. Keys under look. belong to the present shader.
const VALUES = {
  "view.budget": 1000,
  "look.density": 0.02,
  "look.phase": 0.6,
  "look.inside_color": [0.02, 0.02, 0.05],
};

const RELEASE = "0.12.6";

// The live values carry their scalar type. The defaults above need these named.
const UNSIGNED = new Set(["view.budget"]);

// Splits the live values between the two shaders by their uniform root.
function describe(model) {
  const values = (model.values ?? Object.entries(VALUES)).map(([key, value, scalar]) => [
    key,
    value,
    scalar ?? (UNSIGNED.has(key) ? "uint" : "float"),
  ]);
  const library = model.library ?? ["gradient/palette_iq"];
  const pick = (root) => values.filter(([key]) => key.startsWith(root));
  return { library, iterate: pick("view."), present: pick("look."), release: RELEASE, formatValue };
}

const RUST_PROGRAM = `// Needs FragmentColor %%RELEASE%% or later, and the num-bigint and num-traits crates.

use std::error::Error;

use fragmentcolor::{Pass, Renderer, Shader, Target};
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive};

type Failure = Box<dyn Error + Send + Sync>;

const ITERATE: &str = include_str!("iterate.wgsl");
const PRESENT: &str = include_str!("present.wgsl");
const LABEL: &str = include_str!("label.wgsl");

/// A Misiurewicz point: the orbit of 0 lands on a cycle of \`period\` after \`preperiod\` steps.
struct Point {
    re: &'static str,
    im: &'static str,
    preperiod: usize,
    period: usize,
}

/// A spiral in Seahorse Valley. Newton's method refines these digits to the precision DEPTH needs.
const TARGET: Point = Point {
    re: "-0.7432918908524302",
    im: "0.1312405523087976",
    preperiod: 24,
    period: 2,
};
/// The zoom goes in until the view is 10^-DEPTH tall, then back out.
const DEPTH: f64 = 1000.0;
/// Zoom speed, in doublings per second.
const SPEED: f64 = 1.5;
/// Height of the first view.
const START: f64 = 3.0;
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger window renders at a lower resolution.
const ORBIT: usize = 1 << 16;
const PIXELS: u32 = 1 << 21;

struct State {
    iterate: Shader,
    present: Shader,
    compute: Pass,
    passes: Vec<Pass>,
    reference: Reference,
    zoom: f64,
    direction: f64,
    last: Option<f32>,
    pointer: Option<[f32; 2]>,
    moved: f32,
}

// Call setup once, then frame on every redraw of your window.
fn setup() -> Result<State, Failure> {
    let header = format!("const ORBIT: u32 = {ORBIT}u;\\nconst PIXELS: u32 = {PIXELS}u;");

    let iterate = Shader::new([header.as_str(), ITERATE])?;
%%ITERATE_SETS%%

    // The library brings the palette that present.wgsl calls.
    let present = Shader::new([%%LIBRARY_ITEMS%%header.as_str(), LABEL, PRESENT])?;
%%PRESENT_SETS%%

    // Compute one orbit at the precision the deepest view needs and send it to the GPU once.
    let reference = Reference::new(&TARGET, DEPTH)?;
    let bytes: Vec<u8> = reference.orbit.iter().flat_map(|value| value.to_le_bytes()).collect();
    iterate.set("orbit", bytes.as_slice())?;

    let compute = Pass::from_shader("iterate", &iterate);
    let draw = Pass::from_shader("present", &present);
    Ok(State {
        passes: vec![compute.clone(), draw],
        iterate,
        present,
        compute,
        reference,
        zoom: 0.0,
        direction: 1.0,
        last: None,
        pointer: None,
        moved: f32::NEG_INFINITY,
    })
}

fn frame(
    state: &mut State,
    renderer: &Renderer,
    target: &impl Target,
    time: f32,
    size: [f32; 2],
    pointer: [f32; 2],
) -> Result<(), Failure> {
    let zoom = steer(state, time, size, pointer);

    // The compute pass fills at most PIXELS: the window, or the window with each side divided by a whole number.
    let [width, height] = [size[0].max(1.0) as u32, size[1].max(1.0) as u32];
    let shrink = (1..).find(|k| (width / k) * (height / k) <= PIXELS).unwrap_or(1);
    let image = [(width / shrink).max(1), (height / shrink).max(1)];

    // A pixel is 2^pixel wide. The series covers every pixel within 2^radius of the target.
    let pixel = (START / f64::from(image[1])).log2() - zoom;
    let exponent = pixel.floor();
    let radius = pixel + (f64::from(image[0]).hypot(f64::from(image[1])) / 2.0).log2();
    let skip = state.reference.skip(radius);
    let [a, b, c] = state.reference.series[skip].map(Wide::exact);

    state.iterate.set("view.image", image)?;
    state.iterate.set("view.exponent", exponent as i32)?;
    state.iterate.set("view.fraction", (pixel - exponent).exp2() as f32)?;
    state.iterate.set("view.skip", skip as u32)?;
    state.iterate.set("view.length", state.reference.length as u32)?;
    state.iterate.set("view.a", [a.re as f32, a.im as f32])?;
    state.iterate.set("view.a_exponent", a.exponent)?;
    state.iterate.set("view.b", [b.re as f32, b.im as f32])?;
    state.iterate.set("view.b_exponent", b.exponent)?;
    state.iterate.set("view.c", [c.re as f32, c.im as f32])?;
    state.iterate.set("view.c_exponent", c.exponent)?;
    // 8 matches @workgroup_size in iterate.wgsl.
    state.compute.set_compute_dispatch(image[0].div_ceil(8), image[1].div_ceil(8), 1);

    state.present.set("look.resolution", size)?;
    state.present.set("look.image", image)?;
    let depth = zoom * 2_f64.log10() - START.log10();
    state.present.set("look.depth", depth as f32)?;
    renderer.render(&state.passes, target)?;
    Ok(())
}

/// The zoom runs from the whole set down to 10^-DEPTH and back on its own. While
/// the pointer moves, and for two seconds after, its vertical position picks the
/// depth, deepest at the top. Returns the zoom in doublings.
fn steer(state: &mut State, time: f32, size: [f32; 2], pointer: [f32; 2]) -> f64 {
    let delta = state.last.map_or(0.0, |last| f64::from((time - last).clamp(0.0, 0.1)));
    state.last = Some(time);
    let deepest = DEPTH * 10_f64.log2() + START.log2();
    if state.pointer.is_some_and(|last| last != pointer) {
        state.moved = time;
    }
    state.pointer = Some(pointer);
    if time - state.moved < 2.0 {
        let goal = (1.0 - f64::from(pointer[1] / size[1].max(1.0))) * deepest;
        state.zoom += (goal - state.zoom) * (delta * 4.0).min(1.0);
    } else {
        state.zoom += state.direction * SPEED * delta;
        if state.zoom >= deepest {
            state.direction = -1.0;
        }
        if state.zoom <= 0.0 {
            state.direction = 1.0;
        }
    }
    state.zoom = state.zoom.clamp(0.0, deepest);
    state.zoom
}

/// The orbit of 0 under \`z^2 + c\` at full precision, rounded to \`f32\` for the GPU,
/// with Martin's series coefficients A, B and C at every iteration. \`reach\` holds
/// the log2 of the widest radius the series still covers at each iteration.
struct Reference {
    orbit: Vec<f32>,
    series: Vec<[Wide; 3]>,
    reach: Vec<f64>,
    length: usize,
}

/// The largest the first dropped term of the series may grow, relative to the first term.
const TOLERANCE: f64 = 1e-7;

impl Reference {
    fn new(target: &Point, depth: f64) -> Result<Reference, Failure> {
        let bits = (depth * 10_f64.log2()).ceil() as usize + 80;
        let (cr, ci) = refine(target, bits)?;
        let mut orbit = vec![0.0_f32; 2 * ORBIT];
        let mut series = Vec::new();
        let mut reach = vec![f64::INFINITY; ORBIT];
        let (mut zr, mut zi) = (BigInt::ZERO, BigInt::ZERO);
        let [mut a, mut b, mut c, mut d] = [Wide::ZERO; 4];
        let mut length = ORBIT - 1;
        for n in 0..ORBIT {
            let (re, im) = (to_float(&zr, bits), to_float(&zi, bits));
            orbit[2 * n] = re as f32;
            orbit[2 * n + 1] = im as f32;
            series.push([a, b, c]);
            // The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
            if n > 0 {
                let limit = (TOLERANCE.log2() + a.log2_size() - d.log2_size()) / 3.0;
                reach[n] = reach[n - 1].min(limit);
            }
            // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
            if n == ORBIT - 1 || re * re + im * im > 4.0 || a.log2_size() > bits as f64 - 40.0 {
                length = n;
                break;
            }
            // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
            let twice = (2.0 * re, 2.0 * im);
            d = d
                .times(twice)
                .add(a.multiply(c).times((2.0, 0.0)).add(b.multiply(b)));
            c = c.times(twice).add(a.multiply(b).times((2.0, 0.0)));
            b = b.times(twice).add(a.multiply(a));
            a = a.times(twice).add(Wide::ONE);
            let next = ((&zr * &zr - &zi * &zi) >> bits) + &cr;
            zi = ((2 * &zr * &zi) >> bits) + &ci;
            zr = next;
        }
        Ok(Reference {
            orbit,
            series,
            reach,
            length,
        })
    }

    /// The last iteration the series covers for a view of this radius (log2).
    fn skip(&self, radius: f64) -> usize {
        let covered = self.reach[..=self.length].partition_point(|&reach| reach > radius);
        covered.saturating_sub(1)
    }
}

/// A \`BigInt\` \`n\` stands for the real number \`n / 2^bits\`.
fn parse(text: &str, bits: usize) -> Result<BigInt, Failure> {
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text),
    };
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    let value = BigInt::parse_bytes(format!("{whole}{fraction}").as_bytes(), 10)
        .ok_or_else(|| format!("{text} is not a decimal number"))?;
    let scale = BigInt::from(10).pow(fraction.len() as u32);
    Ok(sign * ((value << bits) / scale))
}

fn to_float(value: &BigInt, bits: usize) -> f64 {
    (value >> (bits - 60)).to_f64().unwrap_or(f64::NAN) / 2_f64.powi(60)
}

/// Newton's method on z(preperiod + period) - z(preperiod) moves \`c\` until the orbit
/// of 0 enters its cycle, to the working precision. Each round doubles the correct digits.
fn refine(target: &Point, bits: usize) -> Result<(BigInt, BigInt), Failure> {
    let one = BigInt::from(1) << bits;
    let mut re = parse(target.re, bits)?;
    let mut im = parse(target.im, bits)?;
    for _ in 0..64 {
        let [mut zr, mut zi, mut dr, mut di] = [(); 4].map(|_| BigInt::ZERO);
        let mut kept = [(); 4].map(|_| BigInt::ZERO);
        for n in 0..target.preperiod + target.period {
            if n == target.preperiod {
                kept = [zr.clone(), zi.clone(), dr.clone(), di.clone()];
            }
            // dz/dc becomes 2 z dz/dc + 1, then z becomes z^2 + c.
            let next_dr = ((2 * (&zr * &dr - &zi * &di)) >> bits) + &one;
            di = (2 * (&zr * &di + &zi * &dr)) >> bits;
            dr = next_dr;
            let next_zr = ((&zr * &zr - &zi * &zi) >> bits) + &re;
            zi = ((2 * &zr * &zi) >> bits) + &im;
            zr = next_zr;
        }
        let [kr, ki, kdr, kdi] = kept;
        let (fr, fi, gr, gi) = (zr - kr, zi - ki, dr - kdr, di - kdi);
        let size = (&gr * &gr + &gi * &gi) >> bits;
        if size == BigInt::ZERO {
            break;
        }
        let sr = (((&fr * &gr + &fi * &gi) >> bits) << bits) / &size;
        let si = (((&fi * &gr - &fr * &gi) >> bits) << bits) / &size;
        re -= &sr;
        im -= &si;
        if sr.abs() + si.abs() < BigInt::from(256) {
            break;
        }
    }
    Ok((re, im))
}

/// A complex number beyond the range of an \`f64\`: \`(re + i im) * 2^exponent\`.
#[derive(Clone, Copy)]
struct Wide {
    re: f64,
    im: f64,
    exponent: i32,
}

impl Wide {
    const ZERO: Wide = Wide {
        re: 0.0,
        im: 0.0,
        exponent: 0,
    };
    const ONE: Wide = Wide {
        re: 1.0,
        im: 0.0,
        exponent: 0,
    };

    fn new(re: f64, im: f64, exponent: i32) -> Wide {
        let size = re.abs().max(im.abs());
        if size == 0.0 {
            return Wide::ZERO;
        }
        if size > 2_f64.powi(-64) && size < 2_f64.powi(64) {
            return Wide { re, im, exponent };
        }
        let k = size.log2().round() as i32;
        let scale = 2_f64.powi(-k);
        Wide {
            re: re * scale,
            im: im * scale,
            exponent: exponent + k,
        }
    }

    /// The same number with re and im below 1, as the shader expects.
    fn exact(self) -> Wide {
        let size = self.re.abs().max(self.im.abs());
        if size == 0.0 {
            return Wide::ZERO;
        }
        let k = size.log2().floor() as i32 + 1;
        let scale = 2_f64.powi(-k);
        Wide {
            re: self.re * scale,
            im: self.im * scale,
            exponent: self.exponent + k,
        }
    }

    fn add(self, other: Wide) -> Wide {
        let exponent = self.exponent.max(other.exponent);
        let a = 2_f64.powi(self.exponent - exponent);
        let b = 2_f64.powi(other.exponent - exponent);
        Wide::new(
            self.re * a + other.re * b,
            self.im * a + other.im * b,
            exponent,
        )
    }

    fn multiply(self, other: Wide) -> Wide {
        Wide::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
            self.exponent + other.exponent,
        )
    }

    /// self * (re + i im), for a plain complex number.
    fn times(self, (re, im): (f64, f64)) -> Wide {
        Wide::new(
            self.re * re - self.im * im,
            self.re * im + self.im * re,
            self.exponent,
        )
    }

    fn log2_size(self) -> f64 {
        self.re.hypot(self.im).log2() + f64::from(self.exponent)
    }
}
`;

const PYTHON_PROGRAM = `# Needs FragmentColor %%RELEASE%% or later.

import math
from array import array
from pathlib import Path
from types import SimpleNamespace

from fragmentcolor import Pass, Shader

%%LIBRARY_LINE%%
ITERATE = Path(__file__).with_name("iterate.wgsl").read_text()
PRESENT = Path(__file__).with_name("present.wgsl").read_text()
LABEL = Path(__file__).with_name("label.wgsl").read_text()

# A Misiurewicz point in Seahorse Valley: the orbit of 0 lands on a cycle of
# period 2 after 24 steps. Newton's method refines these digits to the precision DEPTH needs.
TARGET = SimpleNamespace(re="-0.7432918908524302", im="0.1312405523087976", preperiod=24, period=2)
# The zoom goes in until the view is 10^-DEPTH tall, then back out.
DEPTH = 1000
# Zoom speed, in doublings per second.
SPEED = 1.5
# Height of the first view.
START = 3.0
# Room for the reference orbit, and for the pixels the compute pass iterates.
# A larger canvas renders at a lower resolution.
ORBIT = 1 << 16
PIXELS = 1 << 21


# Call setup() once, then frame() on every redraw of your canvas.
def setup():
    header = f"const ORBIT: u32 = {ORBIT}u;\\nconst PIXELS: u32 = {PIXELS}u;"

    iterate = Shader([header, ITERATE])
%%ITERATE_SETS%%

    # The library brings the palette that present.wgsl calls.
    present = Shader([*LIBRARY, header, LABEL, PRESENT])
%%PRESENT_SETS%%

    # Compute one orbit at the precision the deepest view needs and send it to the GPU once.
    reference = compute_reference(TARGET, DEPTH)
    iterate.set("orbit", reference.orbit.tobytes())

    compute = Pass.from_shader("iterate", iterate)
    draw = Pass.from_shader("present", present)
    return SimpleNamespace(
        iterate=iterate,
        present=present,
        compute=compute,
        passes=[compute, draw],
        reference=reference,
        zoom=0.0,
        direction=1.0,
        last=None,
        pointer=None,
        moved=-math.inf,
    )


def frame(state, renderer, target, time, size, pointer):
    zoom = steer(state, time, size, pointer)

    # The compute pass fills at most PIXELS: the canvas, or the canvas with each side divided by a whole number.
    width, height = max(int(size[0]), 1), max(int(size[1]), 1)
    shrink = 1
    while (width // shrink) * (height // shrink) > PIXELS:
        shrink += 1
    image = [max(width // shrink, 1), max(height // shrink, 1)]

    # A pixel is 2^pixel wide. The series covers every pixel within 2^radius of the target.
    pixel = math.log2(START / image[1]) - zoom
    exponent = math.floor(pixel)
    radius = pixel + math.log2(math.hypot(image[0], image[1]) / 2)
    skip = skip_for(state.reference, radius)
    a, b, c = (exact(value) for value in state.reference.series[skip])

    state.iterate.set("view.image", image)
    state.iterate.set("view.exponent", exponent)
    state.iterate.set("view.fraction", 2 ** (pixel - exponent))
    state.iterate.set("view.skip", skip)
    state.iterate.set("view.length", state.reference.length)
    state.iterate.set("view.a", [a[0], a[1]])
    state.iterate.set("view.a_exponent", a[2])
    state.iterate.set("view.b", [b[0], b[1]])
    state.iterate.set("view.b_exponent", b[2])
    state.iterate.set("view.c", [c[0], c[1]])
    state.iterate.set("view.c_exponent", c[2])
    # 8 matches @workgroup_size in iterate.wgsl.
    state.compute.set_compute_dispatch(math.ceil(image[0] / 8), math.ceil(image[1] / 8), 1)

    state.present.set("look.resolution", size)
    state.present.set("look.image", image)
    state.present.set("look.depth", zoom * math.log10(2) - math.log10(START))
    renderer.render(state.passes, target)


# The zoom runs from the whole set down to 10^-DEPTH and back on its own. While
# the pointer moves, and for two seconds after, its vertical position picks the
# depth, deepest at the top. Returns the zoom in doublings.
def steer(state, time, size, pointer):
    delta = 0.0 if state.last is None else min(max(time - state.last, 0.0), 0.1)
    state.last = time
    deepest = DEPTH * math.log2(10) + math.log2(START)
    if state.pointer is not None and list(pointer) != state.pointer:
        state.moved = time
    state.pointer = list(pointer)
    if time - state.moved < 2:
        goal = (1 - pointer[1] / size[1]) * deepest
        state.zoom += (goal - state.zoom) * min(1.0, delta * 4)
    else:
        state.zoom += state.direction * SPEED * delta
        if state.zoom >= deepest:
            state.direction = -1.0
        if state.zoom <= 0:
            state.direction = 1.0
    state.zoom = min(max(state.zoom, 0.0), deepest)
    return state.zoom


# The orbit of 0 under z^2 + c at full precision, rounded to f32 for the GPU,
# with Martin's series coefficients A, B and C at every iteration. The reach list holds
# the log2 of the widest radius the series still covers at each iteration.
def compute_reference(target, depth):
    bits = math.ceil(depth * math.log2(10)) + 80
    cr, ci = refine(target, bits)
    orbit = array("f", [0.0]) * (2 * ORBIT)
    series = []
    reach = [math.inf] * ORBIT
    zr, zi = 0, 0
    a = b = c = d = ZERO
    n = 0
    while True:
        re, im = to_float(zr, bits), to_float(zi, bits)
        orbit[2 * n], orbit[2 * n + 1] = re, im
        series.append((a, b, c))
        # The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
        if n > 0:
            reach[n] = min(reach[n - 1], (math.log2(TOLERANCE) + log2_size(a) - log2_size(d)) / 3)
        # Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
        if n == ORBIT - 1 or re * re + im * im > 4 or log2_size(a) > bits - 40:
            return SimpleNamespace(orbit=orbit, series=series, reach=reach, length=n)
        # Each new coefficient reads the old values of the ones before it, so D updates first and A last.
        twice = (2 * re, 2 * im)
        d = add(times(d, *twice), add(times(multiply(a, c), 2, 0), multiply(b, b)))
        c = add(times(c, *twice), times(multiply(a, b), 2, 0))
        b = add(times(b, *twice), multiply(a, a))
        a = add(times(a, *twice), ONE)
        zr, zi = ((zr * zr - zi * zi) >> bits) + cr, ((2 * zr * zi) >> bits) + ci
        n += 1


# The largest the first dropped term of the series may grow, relative to the first term.
TOLERANCE = 1e-7


# The last iteration the series covers for a view of this radius (log2).
def skip_for(reference, radius):
    low, high = 0, reference.length
    while low < high:
        middle = (low + high + 1) // 2
        if reference.reach[middle] > radius:
            low = middle
        else:
            high = middle - 1
    return low


# An int n stands for the real number n / 2^bits.
def parse(text, bits):
    whole, _, fraction = text.lstrip("-").partition(".")
    value = (int(whole + fraction) << bits) // 10 ** len(fraction)
    return -value if text.startswith("-") else value


def to_float(value, bits):
    return (value >> (bits - 60)) / 2**60


# Newton's method on z(preperiod + period) - z(preperiod) moves c until the orbit
# of 0 enters its cycle, to the working precision. Each round doubles the correct digits.
def refine(target, bits):
    one = 1 << bits
    re, im = parse(target.re, bits), parse(target.im, bits)
    for _ in range(64):
        zr = zi = dr = di = 0
        kr = ki = kdr = kdi = 0
        for n in range(target.preperiod + target.period):
            if n == target.preperiod:
                kr, ki, kdr, kdi = zr, zi, dr, di
            # dz/dc becomes 2 z dz/dc + 1, then z becomes z^2 + c.
            dr, di = ((2 * (zr * dr - zi * di)) >> bits) + one, (2 * (zr * di + zi * dr)) >> bits
            zr, zi = ((zr * zr - zi * zi) >> bits) + re, ((2 * zr * zi) >> bits) + im
        fr, fi, gr, gi = zr - kr, zi - ki, dr - kdr, di - kdi
        size = (gr * gr + gi * gi) >> bits
        if size == 0:
            break
        sr = (((fr * gr + fi * gi) >> bits) << bits) // size
        si = (((fi * gr - fr * gi) >> bits) << bits) // size
        re, im = re - sr, im - si
        if abs(sr) + abs(si) < 256:
            break
    return re, im


# Complex numbers beyond the range of a float: (re, im, exponent) = (re + i im) * 2^exponent.
ZERO = (0.0, 0.0, 0)
ONE = (1.0, 0.0, 0)


def normalize(re, im, exponent):
    size = max(abs(re), abs(im))
    if size == 0:
        return ZERO
    if 2.0**-64 < size < 2.0**64:
        return (re, im, exponent)
    k = round(math.log2(size))
    return (re * 2.0**-k, im * 2.0**-k, exponent + k)


# The same number with re and im below 1, as the shader expects.
def exact(value):
    re, im, exponent = value
    size = max(abs(re), abs(im))
    if size == 0:
        return ZERO
    k = math.floor(math.log2(size)) + 1
    return (re * 2.0**-k, im * 2.0**-k, exponent + k)


def add(a, b):
    exponent = max(a[2], b[2])
    sa, sb = 2.0 ** (a[2] - exponent), 2.0 ** (b[2] - exponent)
    return normalize(a[0] * sa + b[0] * sb, a[1] * sa + b[1] * sb, exponent)


def multiply(a, b):
    return normalize(a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0], a[2] + b[2])


# a * (re + i im), for a plain complex number.
def times(a, re, im):
    return normalize(a[0] * re - a[1] * im, a[0] * im + a[1] * re, a[2])


def log2_size(a):
    size = math.hypot(a[0], a[1])
    return math.log2(size) + a[2] if size else -math.inf
`;

const SWIFT_PROGRAM = `// Needs FragmentColor %%RELEASE%% or later, and the BigInt package
// (https://github.com/attaswift/BigInt).

import BigInt
import Foundation
import FragmentColor

%%LIBRARY_LINE%%

/// A Misiurewicz point: the orbit of 0 lands on a cycle of \`period\` after \`preperiod\` steps.
struct Point {
    let re: String
    let im: String
    let preperiod: Int
    let period: Int
}

/// A spiral in Seahorse Valley. Newton's method refines these digits to the precision DEPTH needs.
let target = Point(re: "-0.7432918908524302", im: "0.1312405523087976", preperiod: 24, period: 2)
/// The zoom goes in until the view is 10^-depth tall, then back out.
let depth = 1000.0
/// Zoom speed, in doublings per second.
let speed = 1.5
/// Height of the first view.
let start = 3.0
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger view renders at a lower resolution.
let orbitSize = 1 << 16
let pixels: UInt32 = 1 << 21

struct State {
    let iterate: Shader
    let present: Shader
    let compute: Pass
    let passes: [Pass]
    let reference: Reference
    var zoom = 0.0
    var direction = 1.0
    var last: Float?
    var pointer: [Float]?
    var moved = -Float.infinity
}

func source(_ name: String) throws -> String {
    let url = Bundle.main.url(forResource: name, withExtension: "wgsl")!
    return try String(contentsOf: url, encoding: .utf8)
}

// Call setup once, then frame from your MTKView or CADisplayLink callback.
func setup() throws -> State {
    let header = "const ORBIT: u32 = \\(orbitSize)u;\\nconst PIXELS: u32 = \\(pixels)u;"

    let iterate = try Shader([header, try source("iterate")])
%%ITERATE_SETS%%

    // The library brings the palette that present.wgsl calls.
    let present = try Shader(library + [header, try source("label"), try source("present")])
%%PRESENT_SETS%%

    // Compute one orbit at the precision the deepest view needs and send it to the GPU once.
    let reference = Reference(target, depth: depth)
    try iterate.set("orbit", reference.orbit.withUnsafeBytes { Data($0) })

    let compute = Pass.fromShader("iterate", iterate)
    let draw = Pass.fromShader("present", present)
    return State(iterate: iterate, present: present, compute: compute, passes: [compute, draw], reference: reference)
}

func frame(
    state: inout State,
    renderer: Renderer,
    target: WindowTarget,
    time: Float,
    size: [Float],
    pointer: [Float]
) throws {
    let zoom = steer(&state, time: time, size: size, pointer: pointer)

    // The compute pass fills at most pixels: the view, or the view with each side divided by a whole number.
    let width = max(UInt32(size[0]), 1)
    let height = max(UInt32(size[1]), 1)
    var shrink: UInt32 = 1
    while (width / shrink) * (height / shrink) > pixels {
        shrink += 1
    }
    let image = [max(width / shrink, 1), max(height / shrink, 1)]

    // A pixel is 2^pixel wide. The series covers every pixel within 2^radius of the target.
    let pixel = log2(start / Double(image[1])) - zoom
    let exponent = pixel.rounded(.down)
    let radius = pixel + log2(hypot(Double(image[0]), Double(image[1])) / 2)
    let skip = state.reference.skip(radius)
    let coefficients = state.reference.series[skip].map(\\.exact)
    let (a, b, c) = (coefficients[0], coefficients[1], coefficients[2])

    try state.iterate.set("view.image", image)
    try state.iterate.set("view.exponent", Int32(exponent))
    try state.iterate.set("view.fraction", Float(exp2(pixel - exponent)))
    try state.iterate.set("view.skip", UInt32(skip))
    try state.iterate.set("view.length", UInt32(state.reference.length))
    try state.iterate.set("view.a", [Float(a.re), Float(a.im)])
    try state.iterate.set("view.a_exponent", Int32(a.exponent))
    try state.iterate.set("view.b", [Float(b.re), Float(b.im)])
    try state.iterate.set("view.b_exponent", Int32(b.exponent))
    try state.iterate.set("view.c", [Float(c.re), Float(c.im)])
    try state.iterate.set("view.c_exponent", Int32(c.exponent))
    // 8 matches @workgroup_size in iterate.wgsl.
    state.compute.setComputeDispatch((image[0] + 7) / 8, (image[1] + 7) / 8, 1)

    try state.present.set("look.resolution", size)
    try state.present.set("look.image", image)
    try state.present.set("look.depth", Float(zoom * log10(2) - log10(start)))
    try renderer.render(state.passes, target)
}

/// The zoom runs from the whole set down to 10^-depth and back on its own. While
/// the pointer moves, and for two seconds after, its vertical position picks the
/// depth, deepest at the top. Returns the zoom in doublings.
func steer(_ state: inout State, time: Float, size: [Float], pointer: [Float]) -> Double {
    let delta = state.last.map { Double(min(max(time - $0, 0), 0.1)) } ?? 0
    state.last = time
    let deepest = depth * log2(10) + log2(start)
    if let last = state.pointer, last != pointer {
        state.moved = time
    }
    state.pointer = pointer
    if time - state.moved < 2 {
        let goal = (1 - Double(pointer[1] / size[1])) * deepest
        state.zoom += (goal - state.zoom) * min(1, delta * 4)
    } else {
        state.zoom += state.direction * speed * delta
        if state.zoom >= deepest {
            state.direction = -1
        }
        if state.zoom <= 0 {
            state.direction = 1
        }
    }
    state.zoom = min(max(state.zoom, 0), deepest)
    return state.zoom
}

/// The orbit of 0 under z^2 + c at full precision, rounded to Float for the GPU,
/// with Martin's series coefficients A, B and C at every iteration. \`reach\` holds
/// the log2 of the widest radius the series still covers at each iteration.
struct Reference {
    var orbit: [Float]
    var series: [[Wide]] = []
    var reach: [Double]
    var length = orbitSize - 1

    init(_ target: Point, depth: Double) {
        let bits = Int((depth * log2(10)).rounded(.up)) + 80
        let (cr, ci) = refine(target, bits: bits)
        orbit = [Float](repeating: 0, count: 2 * orbitSize)
        reach = [Double](repeating: .infinity, count: orbitSize)
        var (zr, zi) = (BigInt(0), BigInt(0))
        var (a, b, c, d) = (Wide.zero, Wide.zero, Wide.zero, Wide.zero)
        for n in 0..<orbitSize {
            let (re, im) = (toDouble(zr, bits: bits), toDouble(zi, bits: bits))
            orbit[2 * n] = Float(re)
            orbit[2 * n + 1] = Float(im)
            series.append([a, b, c])
            // The series holds while the first dropped term, D * delta^4, stays below tolerance * A * delta.
            if n > 0 {
                reach[n] = min(reach[n - 1], (log2(tolerance) + a.log2Size - d.log2Size) / 3)
            }
            // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
            if n == orbitSize - 1 || re * re + im * im > 4 || a.log2Size > Double(bits - 40) {
                length = n
                break
            }
            // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
            let (twiceRe, twiceIm) = (2 * re, 2 * im)
            d = d.times(twiceRe, twiceIm).adding(a.multiplied(by: c).times(2, 0).adding(b.multiplied(by: b)))
            c = c.times(twiceRe, twiceIm).adding(a.multiplied(by: b).times(2, 0))
            b = b.times(twiceRe, twiceIm).adding(a.multiplied(by: a))
            a = a.times(twiceRe, twiceIm).adding(.one)
            (zr, zi) = (((zr * zr - zi * zi) >> bits) + cr, ((2 * zr * zi) >> bits) + ci)
        }
    }

    /// The last iteration the series covers for a view of this radius (log2).
    func skip(_ radius: Double) -> Int {
        var (low, high) = (0, length)
        while low < high {
            let middle = (low + high + 1) / 2
            if reach[middle] > radius {
                low = middle
            } else {
                high = middle - 1
            }
        }
        return low
    }
}

/// The largest the first dropped term of the series may grow, relative to the first term.
let tolerance = 1e-7

/// A \`BigInt\` \`n\` stands for the real number \`n / 2^bits\`.
func parse(_ text: String, bits: Int) -> BigInt {
    let negative = text.hasPrefix("-")
    let parts = text.drop { $0 == "-" }.split(separator: ".", omittingEmptySubsequences: false)
    let fraction = parts.count > 1 ? String(parts[1]) : ""
    let value = (BigInt(String(parts[0]) + fraction)! << bits) / BigInt(10).power(fraction.count)
    return negative ? -value : value
}

func toDouble(_ value: BigInt, bits: Int) -> Double {
    Double(value >> (bits - 60)) / 0x1p60
}

/// Newton's method on z(preperiod + period) - z(preperiod) moves \`c\` until the orbit
/// of 0 enters its cycle, to the working precision. Each round doubles the correct digits.
func refine(_ target: Point, bits: Int) -> (BigInt, BigInt) {
    let one = BigInt(1) << bits
    var re = parse(target.re, bits: bits)
    var im = parse(target.im, bits: bits)
    for _ in 0..<64 {
        var (zr, zi, dr, di) = (BigInt(0), BigInt(0), BigInt(0), BigInt(0))
        var kept = (BigInt(0), BigInt(0), BigInt(0), BigInt(0))
        for n in 0..<(target.preperiod + target.period) {
            if n == target.preperiod {
                kept = (zr, zi, dr, di)
            }
            // dz/dc becomes 2 z dz/dc + 1, then z becomes z^2 + c.
            (dr, di) = (((2 * (zr * dr - zi * di)) >> bits) + one, (2 * (zr * di + zi * dr)) >> bits)
            (zr, zi) = (((zr * zr - zi * zi) >> bits) + re, ((2 * zr * zi) >> bits) + im)
        }
        let (fr, fi, gr, gi) = (zr - kept.0, zi - kept.1, dr - kept.2, di - kept.3)
        let size = (gr * gr + gi * gi) >> bits
        if size == 0 {
            break
        }
        let sr = (((fr * gr + fi * gi) >> bits) << bits) / size
        let si = (((fi * gr - fr * gi) >> bits) << bits) / size
        re -= sr
        im -= si
        if abs(sr) + abs(si) < 256 {
            break
        }
    }
    return (re, im)
}

/// A complex number beyond a Double's range: (re + i im) * 2^exponent.
struct Wide {
    var re: Double
    var im: Double
    var exponent: Int

    static let zero = Wide(re: 0, im: 0, exponent: 0)
    static let one = Wide(re: 1, im: 0, exponent: 0)

    static func make(_ re: Double, _ im: Double, _ exponent: Int) -> Wide {
        let size = max(abs(re), abs(im))
        if size == 0 {
            return .zero
        }
        if size > 0x1p-64 && size < 0x1p64 {
            return Wide(re: re, im: im, exponent: exponent)
        }
        let k = Int(log2(size).rounded())
        return Wide(re: scalbn(re, -k), im: scalbn(im, -k), exponent: exponent + k)
    }

    /// The same number with re and im below 1, as the shader expects.
    var exact: Wide {
        let size = max(abs(re), abs(im))
        if size == 0 {
            return .zero
        }
        let k = Int(log2(size).rounded(.down)) + 1
        return Wide(re: scalbn(re, -k), im: scalbn(im, -k), exponent: exponent + k)
    }

    func adding(_ other: Wide) -> Wide {
        let shared = max(exponent, other.exponent)
        let (a, b) = (scalbn(1.0, exponent - shared), scalbn(1.0, other.exponent - shared))
        return .make(re * a + other.re * b, im * a + other.im * b, shared)
    }

    func multiplied(by other: Wide) -> Wide {
        .make(re * other.re - im * other.im, re * other.im + im * other.re, exponent + other.exponent)
    }

    /// self * (re + i im), for a plain complex number.
    func times(_ otherRe: Double, _ otherIm: Double) -> Wide {
        .make(re * otherRe - im * otherIm, re * otherIm + im * otherRe, exponent)
    }

    var log2Size: Double {
        re == 0 && im == 0 ? -.infinity : log2(hypot(re, im)) + Double(exponent)
    }
}
`;

const KOTLIN_PROGRAM = `// Needs FragmentColor %%RELEASE%% or later.

import android.content.res.AssetManager
import java.math.BigInteger
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.hypot
import kotlin.math.log10
import kotlin.math.log2
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.roundToInt
import org.fragmentcolor.*

%%LIBRARY_LINE%%

/** A Misiurewicz point: the orbit of 0 lands on a cycle of [period] after [preperiod] steps. */
class Point(val re: String, val im: String, val preperiod: Int, val period: Int)

/** A spiral in Seahorse Valley. Newton's method refines these digits to the precision DEPTH needs. */
private val TARGET = Point("-0.7432918908524302", "0.1312405523087976", preperiod = 24, period = 2)
/** The zoom goes in until the view is 10^-DEPTH tall, then back out. */
private const val DEPTH = 1000.0
/** Zoom speed, in doublings per second. */
private const val SPEED = 1.5
/** Height of the first view. */
private const val START = 3.0
/** Room for the reference orbit, and for the pixels the compute pass iterates. */
/** A larger view renders at a lower resolution. */
private const val ORBIT = 1 shl 16
private val PIXELS = 1u shl 21

class State(val iterate: Shader, val present: Shader, val compute: Pass, val passes: List<Pass>, val reference: Reference) {
    var zoom = 0.0
    var direction = 1.0
    var last: Float? = null
    var pointer: List<Float>? = null
    var moved = Float.NEGATIVE_INFINITY
}

// Call setup once, then frame from your Choreographer or render thread callback.
fun setup(assets: AssetManager): State {
    fun source(name: String) = assets.open(name).bufferedReader().use { it.readText() }
    val header = "const ORBIT: u32 = \${ORBIT}u;\\nconst PIXELS: u32 = \${PIXELS}u;"

    val iterate = Shader(listOf(header, source("iterate.wgsl")))
%%ITERATE_SETS%%

    // The library brings the palette that present.wgsl calls.
    val present = Shader(LIBRARY + listOf(header, source("label.wgsl"), source("present.wgsl")))
%%PRESENT_SETS%%

    // Compute one orbit at the precision the deepest view needs and send it to the GPU once.
    val reference = Reference(TARGET, DEPTH)
    val bytes = ByteBuffer.allocate(4 * reference.orbit.size).order(ByteOrder.LITTLE_ENDIAN)
    reference.orbit.forEach { bytes.putFloat(it) }
    iterate.set("orbit", bytes.array())

    val compute = Pass.fromShader("iterate", iterate)
    val draw = Pass.fromShader("present", present)
    return State(iterate, present, compute, listOf(compute, draw), reference)
}

fun frame(
    state: State,
    renderer: Renderer,
    target: WindowTarget,
    time: Float,
    size: List<Float>,
    pointer: List<Float>,
) {
    val zoom = steer(state, time, size, pointer)

    // The compute pass fills at most PIXELS: the view, or the view with each side divided by a whole number.
    val width = maxOf(size[0].toUInt(), 1u)
    val height = maxOf(size[1].toUInt(), 1u)
    var shrink = 1u
    while ((width / shrink) * (height / shrink) > PIXELS) shrink++
    val image = listOf(maxOf(width / shrink, 1u), maxOf(height / shrink, 1u))

    // A pixel is 2^pixel wide. The series covers every pixel within 2^radius of the target.
    val pixel = log2(START / image[1].toDouble()) - zoom
    val exponent = floor(pixel)
    val radius = pixel + log2(hypot(image[0].toDouble(), image[1].toDouble()) / 2)
    val skip = state.reference.skip(radius)
    val (a, b, c) = state.reference.series[skip].map { it.exact() }

    state.iterate.set("view.image", image)
    state.iterate.set("view.exponent", exponent.toInt())
    state.iterate.set("view.fraction", 2.0.pow(pixel - exponent).toFloat())
    state.iterate.set("view.skip", skip.toUInt())
    state.iterate.set("view.length", state.reference.length.toUInt())
    state.iterate.set("view.a", listOf(a.re.toFloat(), a.im.toFloat()))
    state.iterate.set("view.a_exponent", a.exponent)
    state.iterate.set("view.b", listOf(b.re.toFloat(), b.im.toFloat()))
    state.iterate.set("view.b_exponent", b.exponent)
    state.iterate.set("view.c", listOf(c.re.toFloat(), c.im.toFloat()))
    state.iterate.set("view.c_exponent", c.exponent)
    // 8 matches @workgroup_size in iterate.wgsl.
    state.compute.setComputeDispatch((image[0] + 7u) / 8u, (image[1] + 7u) / 8u, 1u)

    state.present.set("look.resolution", size)
    state.present.set("look.image", image)
    state.present.set("look.depth", (zoom * log10(2.0) - log10(START)).toFloat())
    renderer.render(state.passes, target)
}

/**
 * The zoom runs from the whole set down to 10^-DEPTH and back on its own. While
 * the pointer moves, and for two seconds after, its vertical position picks the
 * depth, deepest at the top. Returns the zoom in doublings.
 */
fun steer(state: State, time: Float, size: List<Float>, pointer: List<Float>): Double {
    val delta = state.last?.let { min(max(time - it, 0f), 0.1f).toDouble() } ?: 0.0
    state.last = time
    val deepest = DEPTH * log2(10.0) + log2(START)
    if (state.pointer != null && state.pointer != pointer) state.moved = time
    state.pointer = pointer
    if (time - state.moved < 2f) {
        val goal = (1 - (pointer[1] / size[1]).toDouble()) * deepest
        state.zoom += (goal - state.zoom) * min(1.0, delta * 4)
    } else {
        state.zoom += state.direction * SPEED * delta
        if (state.zoom >= deepest) state.direction = -1.0
        if (state.zoom <= 0) state.direction = 1.0
    }
    state.zoom = state.zoom.coerceIn(0.0, deepest)
    return state.zoom
}

/**
 * The orbit of 0 under z^2 + c at full precision, rounded to Float for the GPU,
 * with Martin's series coefficients A, B and C at every iteration. [reach] holds
 * the log2 of the widest radius the series still covers at each iteration.
 */
class Reference(target: Point, depth: Double) {
    val orbit = FloatArray(2 * ORBIT)
    val series = mutableListOf<List<Wide>>()
    val reach = DoubleArray(ORBIT) { Double.POSITIVE_INFINITY }
    var length = ORBIT - 1

    init {
        val bits = ceil(depth * log2(10.0)).toInt() + 80
        val (cr, ci) = refine(target, bits)
        var zr = BigInteger.ZERO
        var zi = BigInteger.ZERO
        var a = Wide.ZERO
        var b = Wide.ZERO
        var c = Wide.ZERO
        var d = Wide.ZERO
        for (n in 0 until ORBIT) {
            val re = toDouble(zr, bits)
            val im = toDouble(zi, bits)
            orbit[2 * n] = re.toFloat()
            orbit[2 * n + 1] = im.toFloat()
            series.add(listOf(a, b, c))
            // The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
            if (n > 0) reach[n] = min(reach[n - 1], (log2(TOLERANCE) + a.log2Size() - d.log2Size()) / 3)
            // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
            if (n == ORBIT - 1 || re * re + im * im > 4 || a.log2Size() > bits - 40) {
                length = n
                break
            }
            // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
            d = d.times(2 * re, 2 * im) + ((a * c).times(2.0, 0.0) + b * b)
            c = c.times(2 * re, 2 * im) + (a * b).times(2.0, 0.0)
            b = b.times(2 * re, 2 * im) + a * a
            a = a.times(2 * re, 2 * im) + Wide.ONE
            val next = zr.multiply(zr).subtract(zi.multiply(zi)).shiftRight(bits).add(cr)
            zi = zr.multiply(zi).shiftLeft(1).shiftRight(bits).add(ci)
            zr = next
        }
    }

    /** The last iteration the series covers for a view of this radius (log2). */
    fun skip(radius: Double): Int {
        var low = 0
        var high = length
        while (low < high) {
            val middle = (low + high + 1) / 2
            if (reach[middle] > radius) low = middle else high = middle - 1
        }
        return low
    }
}

/** The largest the first dropped term of the series may grow, relative to the first term. */
private const val TOLERANCE = 1e-7

/** A BigInteger n stands for the real number n / 2^bits. */
fun parse(text: String, bits: Int): BigInteger {
    val parts = text.removePrefix("-").split(".")
    val fraction = parts.getOrElse(1) { "" }
    val value = BigInteger(parts[0] + fraction).shiftLeft(bits).divide(BigInteger.TEN.pow(fraction.length))
    return if (text.startsWith("-")) value.negate() else value
}

fun toDouble(value: BigInteger, bits: Int): Double = value.shiftRight(bits - 60).toDouble() / 2.0.pow(60)

/**
 * Newton's method on z(preperiod + period) - z(preperiod) moves c until the orbit
 * of 0 enters its cycle, to the working precision. Each round doubles the correct digits.
 */
fun refine(target: Point, bits: Int): Pair<BigInteger, BigInteger> {
    val one = BigInteger.ONE.shiftLeft(bits)
    var re = parse(target.re, bits)
    var im = parse(target.im, bits)
    repeat(64) {
        var zr = BigInteger.ZERO
        var zi = BigInteger.ZERO
        var dr = BigInteger.ZERO
        var di = BigInteger.ZERO
        var kept = listOf(zr, zi, dr, di)
        for (n in 0 until target.preperiod + target.period) {
            if (n == target.preperiod) kept = listOf(zr, zi, dr, di)
            // dz/dc becomes 2 z dz/dc + 1, then z becomes z^2 + c.
            val nextDr = zr.multiply(dr).subtract(zi.multiply(di)).shiftLeft(1).shiftRight(bits).add(one)
            di = zr.multiply(di).add(zi.multiply(dr)).shiftLeft(1).shiftRight(bits)
            dr = nextDr
            val nextZr = zr.multiply(zr).subtract(zi.multiply(zi)).shiftRight(bits).add(re)
            zi = zr.multiply(zi).shiftLeft(1).shiftRight(bits).add(im)
            zr = nextZr
        }
        val fr = zr - kept[0]
        val fi = zi - kept[1]
        val gr = dr - kept[2]
        val gi = di - kept[3]
        val size = gr.multiply(gr).add(gi.multiply(gi)).shiftRight(bits)
        if (size.signum() == 0) return Pair(re, im)
        val sr = fr.multiply(gr).add(fi.multiply(gi)).shiftRight(bits).shiftLeft(bits).divide(size)
        val si = fi.multiply(gr).subtract(fr.multiply(gi)).shiftRight(bits).shiftLeft(bits).divide(size)
        re -= sr
        im -= si
        if (sr.abs() + si.abs() < BigInteger.valueOf(256)) return Pair(re, im)
    }
    return Pair(re, im)
}

/** A complex number beyond a Double's range: (re + i im) * 2^exponent. */
data class Wide(val re: Double, val im: Double, val exponent: Int) {
    companion object {
        val ZERO = Wide(0.0, 0.0, 0)
        val ONE = Wide(1.0, 0.0, 0)

        fun of(re: Double, im: Double, exponent: Int): Wide {
            val size = max(abs(re), abs(im))
            if (size == 0.0) return ZERO
            if (size > 2.0.pow(-64) && size < 2.0.pow(64)) return Wide(re, im, exponent)
            val k = log2(size).roundToInt()
            return Wide(re * 2.0.pow(-k), im * 2.0.pow(-k), exponent + k)
        }
    }

    /** The same number with re and im below 1, as the shader expects. */
    fun exact(): Wide {
        val size = max(abs(re), abs(im))
        if (size == 0.0) return ZERO
        val k = floor(log2(size)).toInt() + 1
        return Wide(re * 2.0.pow(-k), im * 2.0.pow(-k), exponent + k)
    }

    operator fun plus(other: Wide): Wide {
        val shared = max(exponent, other.exponent)
        val a = 2.0.pow(exponent - shared)
        val b = 2.0.pow(other.exponent - shared)
        return of(re * a + other.re * b, im * a + other.im * b, shared)
    }

    operator fun times(other: Wide) =
        of(re * other.re - im * other.im, re * other.im + im * other.re, exponent + other.exponent)

    /** this * (re + i im), for a plain complex number. */
    fun times(otherRe: Double, otherIm: Double) = of(re * otherRe - im * otherIm, re * otherIm + im * otherRe, exponent)

    fun log2Size(): Double = if (re == 0.0 && im == 0.0) Double.NEGATIVE_INFINITY else log2(hypot(re, im)) + exponent
}
`;

// Each port program carries %% placeholders for the parts the live state decides.
function fillProgram(text, parts) {
  return text.replace(/%%(\w+)%%/g, (_, key) => parts[key] ?? "");
}

function quote(text) {
  return JSON.stringify(String(text));
}

function kotlinQuote(text) {
  return quote(text).replace(/\$/g, "\\$");
}

// One `set` call per value, spelled the way `lang` writes it.
function setCalls(lang, name, values, { before = "", after = "", text = quote, formatValue }) {
  return values
    .map(([key, value, scalar]) => `    ${before}${name}.set(${text(key)}, ${formatValue(lang, value, scalar)})${after}`)
    .join("\n");
}

function writeRust({ library, iterate, present, release, formatValue }) {
  const sets = (name, values) => setCalls("rust", name, values, { after: "?;", formatValue });
  return fillProgram(RUST_PROGRAM, {
    RELEASE: release,
    LIBRARY_ITEMS: library.map((slug) => `${quote(slug)}, `).join(""),
    ITERATE_SETS: sets("iterate", iterate),
    PRESENT_SETS: sets("present", present),
  });
}

function writePython({ library, iterate, present, release, formatValue }) {
  const sets = (name, values) => setCalls("python", name, values, { formatValue });
  return fillProgram(PYTHON_PROGRAM, {
    RELEASE: release,
    LIBRARY_LINE: `LIBRARY = [${library.map(quote).join(", ")}]`,
    ITERATE_SETS: sets("iterate", iterate),
    PRESENT_SETS: sets("present", present),
  });
}

function writeSwift({ library, iterate, present, release, formatValue }) {
  const sets = (name, values) => setCalls("swift", name, values, { before: "try ", formatValue });
  return fillProgram(SWIFT_PROGRAM, {
    RELEASE: release,
    LIBRARY_LINE: library.length ? `let library = [${library.map(quote).join(", ")}]` : "let library: [String] = []",
    ITERATE_SETS: sets("iterate", iterate),
    PRESENT_SETS: sets("present", present),
  });
}

function writeKotlin({ library, iterate, present, release, formatValue }) {
  const sets = (name, values) => setCalls("kotlin", name, values, { text: kotlinQuote, formatValue });
  return fillProgram(KOTLIN_PROGRAM, {
    RELEASE: release,
    LIBRARY_LINE: library.length
      ? `private val LIBRARY = listOf(${library.map(kotlinQuote).join(", ")})`
      : "private val LIBRARY = emptyList<String>()",
    ITERATE_SETS: sets("iterate", iterate),
    PRESENT_SETS: sets("present", present),
  });
}

export default {
  id: "mandelbrot",
  title: "Mandelbrot",
  summary:
    "A dive into a spiral of the Mandelbrot set, down to 1e-1000 and back. A BigInt reference orbit guides a compute pass that iterates every pixel in f32. The pointer's height picks the depth.",
  kind: "compute",
  requires: "webgpu",
  needs: RELEASE,
  poster: "/playground/mandelbrot.webp",
  library: ["gradient/palette_iq"],
  values: VALUES,
  files: { "iterate.wgsl": ITERATE, "present.wgsl": PRESENT, "label.wgsl": LABEL },
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
