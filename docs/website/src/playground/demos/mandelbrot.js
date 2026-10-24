// Infinite zoom into the Mandelbrot set. main.js computes one reference orbit
// with BigInt at full precision, a compute pass iterates every pixel as a small
// f32 difference from it, and a render pass colors the counts. The tutorial at
// docs/tutorials/infinite-zoom walks through the same program.

import { formatValue } from "../codegen.js";

const ITERATE = /* wgsl */ `// Iterates every pixel as a small difference from the reference orbit that
// main.js computes at full precision. main.js puts ORBIT and PIXELS ahead of
// this file, and present.wgsl reads the counts it writes.

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

// An orbit whose squared size passes this never comes back.
const ESCAPE: f32 = 65536.0;

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// x * 2^k, or zero once 2^k is too small for an f32.
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
  // that never escape. A direct test spares them the whole budget.
  if (e > -20) {
    let c = orbit[1] + scale(d, e);
    let q = dot(c - vec2<f32>(0.25, 0.0), c - vec2<f32>(0.25, 0.0));
    let bulb = c + vec2<f32>(1.0, 0.0);
    if (q * (q + c.x - 0.25) < 0.25 * c.y * c.y || dot(bulb, bulb) < 0.0625) {
      counts[index] = -1.0;
      return;
    }
  }

  // The series moves every pixel to iteration skip in one step. Its difference
  // from the reference is w * 2^s, with w kept near 1.
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
      // z becomes 2 Z z + z^2 + d, with every term divided by 2^s.
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
    // Rebase when the pixel is closer to 0 than to the reference, or the orbit runs out.
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
  // A smooth count: how far past ESCAPE the last step went adds the fraction.
  var count = -1.0;
  if (n < limit) {
    count = f32(n) + 1.0 - log2(log2(size2) / log2(ESCAPE));
  }
  counts[index] = count;
}
`;

const PRESENT = /* wgsl */ `// Colors the counts that iterate.wgsl writes. label.wgsl prints the depth.

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

// Fullscreen triangle: three vertices that cover the canvas.
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
  let exponent = u32(max(depth, 0.0));
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
// period 2 after 24 steps. Newton's method sharpens these digits to any precision.
const TARGET = { re: "-0.7432918908524302", im: "0.1312405523087976", preperiod: 24, period: 2 };
// The zoom dives until the view is 10^-DEPTH tall, then climbs back.
const DEPTH = 1000;
// Zoom speed, in doublings per second.
const SPEED = 1.5;
// Height of the first view.
const START = 3;
// Room for the reference orbit, and for the pixels the compute pass iterates.
// A larger canvas renders at a lower resolution.
const ORBIT = 1 << 16;
const PIXELS = 1 << 21;
// Iterations the orbit keeps past the deepest view, for the budget control.
const BUDGET = 5000;

export async function setup(renderer, target) {
  const header = \`const ORBIT: u32 = \${ORBIT}u;\\nconst PIXELS: u32 = \${PIXELS}u;\`;

  const iterate = await prepare("iterate", [header, ITERATE]);
  iterate.shader.set("view.budget", 1000);

  // The library brings the palette that present.wgsl calls.
  const present = await prepare("present", [...LIBRARY, header, LABEL, PRESENT]);
  present.shader.set("look.density", 0.02);
  present.shader.set("look.phase", 0.6);
  present.shader.set("look.inside_color", [0.02, 0.02, 0.05]);

  // One orbit at the precision the deepest view needs, sent to the GPU once.
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

  // The compute pass fills at most PIXELS: the canvas, or a whole fraction of it.
  let shrink = 1;
  while (Math.floor(size[0] / shrink) * Math.floor(size[1] / shrink) > PIXELS) shrink++;
  const image = [Math.max(Math.floor(size[0] / shrink), 1), Math.max(Math.floor(size[1] / shrink), 1)];

  // A pixel is 2^pixel wide. The series covers every pixel within radius of the target.
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

// The zoom runs from the whole set down to 10^-DEPTH and back on its own. Moving
// the pointer takes over for two seconds: its height picks the depth, deepest at
// the top. Returns the zoom in doublings.
function steer(state, time, size, [x, y]) {
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
// with Martin's series coefficients A, B and C at every iteration. reach holds
// the log2 of the widest radius the series still covers at each iteration.
function computeReference(target, depth) {
  const deepest = -depth * Math.log2(10) - 1;
  const bits = Math.ceil(depth * Math.log2(10)) + 80;
  const shift = BigInt(bits);
  const [cr, ci] = refine(target, bits);
  const orbit = new Float32Array(2 * ORBIT);
  const series = [];
  const reach = new Float64Array(ORBIT).fill(Infinity);
  let [zr, zi] = [0n, 0n];
  let [a, b, c, d] = [ZERO, ZERO, ZERO, ZERO];
  let end = ORBIT - 1;
  for (let n = 0; ; n++) {
    const [re, im] = [toFloat(zr, bits), toFloat(zi, bits)];
    orbit[2 * n] = re;
    orbit[2 * n + 1] = im;
    series.push([a, b, c]);
    // The series holds while the first dropped term, D d^4, stays below TOLERANCE * A d.
    if (n > 0) reach[n] = Math.min(reach[n - 1], (Math.log2(TOLERANCE) + log2Size(a) - log2Size(d)) / 3);
    if (reach[n] < deepest) end = Math.min(end, n + BUDGET);
    if (n === end || re * re + im * im > 4) return { orbit, series, reach, length: n };
    // Each coefficient follows from the old ones, so they update in this order.
    d = add(times(d, 2 * re, 2 * im), add(times(multiply(a, c), 2, 0), multiply(b, b)));
    c = add(times(c, 2 * re, 2 * im), times(multiply(a, b), 2, 0));
    b = add(times(b, 2 * re, 2 * im), multiply(a, a));
    a = add(times(a, 2 * re, 2 * im), ONE);
    const next = ((zr * zr - zi * zi) >> shift) + cr;
    zi = ((2n * zr * zi) >> shift) + ci;
    zr = next;
  }
}

// How far the dropped terms of the series may drift, relative to the first one.
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
// of 0 lands exactly on its cycle. Each round doubles the correct digits.
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

// Complex numbers beyond a double's range: [re, im, exponent] = (re + i im) * 2^exponent.
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

// Splits the live values between the two shaders by their uniform root, spelled
// for each language.
function describe(model) {
  const values = model.values ?? Object.entries(VALUES);
  const library = model.library ?? ["gradient/palette_iq"];
  const pick = (root) => values.filter(([key]) => key.startsWith(root));
  return { library, iterate: pick("view."), present: pick("look."), release: RELEASE, formatValue };
}

function writeRust() {
  return undefined;
}

function writePython() {
  return undefined;
}

function writeSwift() {
  return undefined;
}

function writeKotlin() {
  return undefined;
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
