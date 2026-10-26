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
  // Where the view's center sits relative to the reference, in pixels.
  center: vec2<f32>,
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
  let d = ((vec2<f32>(id.xy) + 0.5 - half) * vec2<f32>(1.0, -1.0) + view.center) * view.fraction;
  let e = view.exponent;
  let index = id.y * view.image.x + id.x;

  // In a wide view, the main cardioid and the bulb left of it cover many pixels
  // that never escape. A direct test skips their loop. orbit[1] is the reference's c.
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
// Until the first scroll, pinch or drag, the zoom goes in until the view is
// 10^-DEPTH tall, then back out.
const DEPTH = 1000;
// Zoom speed, in doublings per second.
const SPEED = 1.5;
// Height of the first view.
const START = 3;
// Room for the reference orbit, and for the pixels the compute pass iterates.
// A larger canvas renders at a lower resolution.
const ORBIT = 1 << 16;
const PIXELS = 1 << 21;
// Milliseconds per frame spent growing reference orbits, so a new one never stalls the view.
const WORK = 6;

export async function setup(renderer, target) {
  const header = \`const ORBIT: u32 = \${ORBIT}u;\\nconst PIXELS: u32 = \${PIXELS}u;\`;

  const iterate = await prepare("iterate", [header, ITERATE]);
  iterate.shader.set("view.budget", 183);

  // The library brings the palette that present.wgsl calls.
  const present = await prepare("present", [...LIBRARY, header, LABEL, PRESENT]);
  present.shader.set("look.density", 0.023);
  present.shader.set("look.phase", 0.332);
  present.shader.set("look.inside_color", [0.02, 0.02, 0.05]);

  // The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
  const home = homeReference(TARGET, DEPTH);
  const stages = [iterate, present];
  return {
    iterate: iterate.shader,
    present: present.shader,
    compute: iterate.pass,
    stages,
    passes: stages.map((stage) => stage.pass),
    reference: home,
    next: null,
    center: [home.cr, home.ci],
    bits: home.bits,
    zoom: 0,
    direction: 1,
    last: null,
    exploring: false,
    sent: null,
    drawn: null,
  };
}

export function frame(state, renderer, target, time, size, pointer, gesture = { zoom: 0, pan: [0, 0] }) {
  // This program draws, so later builds may reuse its shaders.
  for (const stage of state.stages) kept.set(stage.key, stage);
  const delta = state.last === null ? 0 : Math.min(Math.max(time - state.last, 0), 0.1);
  state.last = time;

  // The compute pass fills at most PIXELS: the canvas, or the canvas with each side divided by a whole number.
  let shrink = 1;
  while (Math.floor(size[0] / shrink) * Math.floor(size[1] / shrink) > PIXELS) shrink++;
  const image = [Math.max(Math.floor(size[0] / shrink), 1), Math.max(Math.floor(size[1] / shrink), 1)];
  const budget = state.iterate.get("view.budget");

  // The first scroll, pinch or drag ends the dive. From then on the view is the reader's.
  if (gesture.zoom || gesture.pan[0] || gesture.pan[1]) state.exploring = true;
  const deadline = performance.now() + WORK;
  if (state.exploring) explore(state, size, image, pointer, gesture, budget, deadline);
  else dive(state, image, delta, budget, deadline);

  // A pixel is 2^pixel wide, and the view's center sits away from the reference by \`center\` pixels.
  const reference = state.reference;
  const pixel = Math.log2(START / image[1]) - state.zoom;
  const exponent = Math.floor(pixel);
  const center = offset(state, reference, pixel);
  // The series covers every pixel within 2^radius of the reference.
  const radius = pixel + Math.log2(Math.hypot(image[0], image[1]) / 2 + Math.hypot(center[0], center[1]));
  const skip = skipFor(reference, radius);
  const [a, b, c] = reference.series[skip].map(exact);

  // The GPU gets the orbit again whenever it grew or another reference took over.
  if (state.sent?.reference !== reference || state.sent.length !== reference.length) {
    state.iterate.set("orbit", new Uint8Array(reference.orbit.buffer));
    state.sent = { reference, length: reference.length };
  }
  state.iterate.set("view.image", image);
  state.iterate.set("view.exponent", exponent);
  state.iterate.set("view.fraction", 2 ** (pixel - exponent));
  state.iterate.set("view.center", center);
  state.iterate.set("view.skip", skip);
  state.iterate.set("view.length", reference.length);
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
  state.present.set("look.depth", state.zoom * Math.log10(2) - Math.log10(START));

  // When nothing moved, the counts from the last frame still hold, so only the colors redraw.
  const view = JSON.stringify([image, pixel, center, skip, reference.length, budget]);
  renderer.render(view === state.drawn ? state.passes.slice(1) : state.passes, target);
  state.drawn = view;
}

// The zoom runs from the whole set down to 10^-DEPTH and back on its own, never
// deeper than the reference orbit computed so far covers.
function dive(state, image, delta, budget, deadline) {
  grow(state.reference, deadline);
  const deepest = DEPTH * Math.log2(10) + Math.log2(START);
  const zoom = Math.min(Math.max(state.zoom + state.direction * SPEED * delta, 0), deepest);
  if (covers(state.reference, image, zoom, [0, 0], budget)) state.zoom = zoom;
  if (state.zoom >= deepest) state.direction = -1;
  if (state.zoom <= 0) state.direction = 1;
}

// The reader's view: the wheel or a pinch zooms around the pointer, and a drag pans.
// When the view drifts away from the reference, or zooms past its digits, a new
// reference starts where the view is heading and takes over once it covers the view.
function explore(state, size, image, pointer, gesture, budget, deadline) {
  const pixel = Math.log2(START / image[1]) - state.zoom;
  // The pointer and the drag, in image pixels with y pointing up.
  const scale = image[1] / size[1];
  const at = [(pointer[0] - size[0] / 2) * scale, (size[1] / 2 - pointer[1]) * scale];
  state.center = pointAt(state, -gesture.pan[0] * scale, gesture.pan[1] * scale, pixel);

  const deepest = DEPTH * Math.log2(10) + Math.log2(START);
  const zoom = Math.min(Math.max(state.zoom + gesture.zoom, 0), deepest);
  const after = Math.log2(START / image[1]) - zoom;
  // The center needs a few more digits than the deepest pixel it has seen.
  if (state.bits < Math.ceil(-after) + 64) {
    state.center = state.center.map((value) => value << 128n);
    state.bits += 128;
  }
  // Zooming around the pointer keeps the point under it in place: the center moves
  // toward it by \`stay\` of the way, and the reference's offset moves with it.
  const stay = 1 - 2 ** (state.zoom - zoom);
  const reference = state.reference;
  const drift = offset(state, reference, after).map((value, i) => value + at[i] * stay * 2 ** (pixel - after));
  // Zooming in waits while the reference lacks the digits, is more than two views
  // from where the zoom lands, or hasn't grown far enough yet.
  const near = Math.hypot(...drift) <= 2 * image[1];
  const deep = reference.bits >= Math.ceil(-after) + 40;
  const far = zoom > state.zoom && !(near && deep);
  if (zoom < state.zoom || (!far && covers(reference, image, zoom, drift, budget))) {
    state.center = pointAt(state, at[0] * stay, at[1] * stay, pixel);
    state.zoom = zoom;
  }

  // A zoom in heads for the point under the pointer; anything else stays at the center.
  const now = Math.log2(START / image[1]) - state.zoom;
  const aim = gesture.zoom > 0 ? at : [0, 0];
  // The offset is the center minus the reference, so a reference at the aim sits at -aim.
  const away = (other, limit) => {
    const [x, y] = offset(state, other, now);
    return Math.hypot(x + aim[0], y + aim[1]) > limit;
  };
  // A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
  const finest = far ? after : now;
  if (far || away(reference, image[1] / 4) || reference.bits < Math.ceil(-now) + 120) {
    if (!state.next || away(state.next, image[1] / 2) || state.next.bits < Math.ceil(-finest) + 40) {
      const bits = Math.ceil(-finest) + 200;
      state.next = startReference(alignTo(pointAt(state, aim[0], aim[1], now), state.bits, bits), bits);
    }
  }
  if (state.next) {
    grow(state.next, deadline);
    if (covers(state.next, image, state.zoom, offset(state, state.next, now), budget)) {
      state.reference = state.next;
      state.next = null;
    }
  }
  grow(state.reference, deadline);
}

// Whether this reference serves a view at this zoom: its orbit is complete, or long
// enough for the series skip plus every pixel's budget.
function covers(reference, image, zoom, center, budget) {
  if (reference.done) return true;
  const pixel = Math.log2(START / image[1]) - zoom;
  const radius = pixel + Math.log2(Math.hypot(image[0], image[1]) / 2 + Math.hypot(center[0], center[1]));
  return skipFor(reference, radius) + budget + 200 <= reference.length;
}

// The point (x, y) pixels from the view's center, as fixed point numbers at state.bits.
function pointAt(state, x, y, pixel) {
  const scale = pixel + state.bits;
  const lift = Math.max(0, Math.floor(scale) - 40);
  const step = (value) => BigInt(Math.round(value * 2 ** (scale - lift))) << BigInt(lift);
  return [state.center[0] + step(x), state.center[1] + step(y)];
}

// The view's center minus the reference's c, in pixels.
function offset(state, reference, pixel) {
  const bits = Math.max(state.bits, reference.bits);
  const [cr, ci] = alignTo(state.center, state.bits, bits);
  const [rr, ri] = alignTo([reference.cr, reference.ci], reference.bits, bits);
  return [scaled(cr - rr, -bits - pixel), scaled(ci - ri, -bits - pixel)];
}

// value * 2^power as a double, for a BigInt of any size.
function scaled(value, power) {
  const size = (value < 0n ? -value : value).toString(2).length;
  const drop = Math.max(0, size - 60);
  return Number(value >> BigInt(drop)) * 2 ** (power + drop);
}

// The same fixed point numbers at another precision.
function alignTo(values, from, to) {
  return values.map((value) => (to >= from ? value << BigInt(to - from) : value >> BigInt(from - to)));
}

// The dive's reference takes a moment to grow at 1000 digits, so builds share it
// while TARGET and DEPTH stay the same.
const homes = (globalThis.mandelbrotHomes ??= new Map());

function homeReference(target, depth) {
  const key = JSON.stringify([target, depth]);
  if (!homes.has(key)) {
    homes.clear();
    const bits = Math.ceil(depth * Math.log2(10)) + 80;
    homes.set(key, startReference(refine(target, bits), bits));
  }
  return homes.get(key);
}

// The orbit of 0 under z^2 + c at \`bits\` of precision, rounded to f32 for the GPU,
// with Martin's series coefficients A, B and C at every iteration. The reach array
// holds the log2 of the widest radius the series still covers at each iteration.
// grow adds to it until a deadline; length is the last index computed so far.
function startReference([cr, ci], bits) {
  return {
    cr,
    ci,
    bits,
    orbit: new Float32Array(2 * ORBIT),
    series: [],
    reach: new Float64Array(ORBIT).fill(Infinity),
    length: -1,
    done: false,
    z: [0n, 0n],
    coefficients: [ZERO, ZERO, ZERO, ZERO],
    saved: null,
    checkpoint: 1,
  };
}

function grow(reference, deadline) {
  const shift = BigInt(reference.bits);
  let [zr, zi] = reference.z;
  let [a, b, c, d] = reference.coefficients;
  while (!reference.done && (reference.length < 0 || performance.now() < deadline)) {
    for (let k = 0; k < 64 && !reference.done; k++) {
      const n = reference.length + 1;
      const [re, im] = [toFloat(zr, reference.bits), toFloat(zi, reference.bits)];
      reference.orbit[2 * n] = re;
      reference.orbit[2 * n + 1] = im;
      reference.series.push([a, b, c]);
      // The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
      if (n > 0) reference.reach[n] = Math.min(reference.reach[n - 1], (Math.log2(TOLERANCE) + log2Size(a) - log2Size(d)) / 3);
      reference.length = n;
      // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
      // An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
      // and nothing after adds anything. Brent's method compares it with powers of two.
      const saved = reference.saved;
      const settled = saved && Math.hypot(re - saved[0], im - saved[1]) < 1e-12 && Math.abs(log2Size(a) - saved[2]) < 1e-9;
      if (n === ORBIT - 1 || re * re + im * im > 4 || log2Size(a) > reference.bits - 40 || settled) {
        reference.done = true;
        break;
      }
      if (n === reference.checkpoint) {
        reference.saved = [re, im, log2Size(a)];
        reference.checkpoint *= 2;
      }
      // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
      d = add(times(d, 2 * re, 2 * im), add(times(multiply(a, c), 2, 0), multiply(b, b)));
      c = add(times(c, 2 * re, 2 * im), times(multiply(a, b), 2, 0));
      b = add(times(b, 2 * re, 2 * im), multiply(a, a));
      a = add(times(a, 2 * re, 2 * im), ONE);
      const next = ((zr * zr - zi * zi) >> shift) + reference.cr;
      zi = ((2n * zr * zi) >> shift) + reference.ci;
      zr = next;
    }
  }
  reference.z = [zr, zi];
  reference.coefficients = [a, b, c, d];
}

// The largest the first dropped term of the series may grow, relative to the first term.
const TOLERANCE = 1e-7;

// The last iteration the series covers for a view of this radius (log2).
function skipFor(reference, radius) {
  // The last step reads one entry past skip, so skip stops one short of the end.
  let [low, high] = [0, Math.max(reference.length - 1, 0)];
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
  "view.budget": 183,
  "look.density": 0.023,
  "look.phase": 0.332,
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
use std::time::{Duration, Instant};

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
/// Until the first scroll, pinch or drag, the zoom goes in until the view is
/// 10^-DEPTH tall, then back out.
const DEPTH: f64 = 1000.0;
/// Zoom speed, in doublings per second.
const SPEED: f64 = 1.5;
/// Height of the first view.
const START: f64 = 3.0;
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger window renders at a lower resolution.
const ORBIT: usize = 1 << 16;
const PIXELS: u32 = 1 << 21;
/// Time per frame spent growing reference orbits, so a new one never stalls the view.
const WORK: Duration = Duration::from_millis(6);

/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings
/// (positive zooms in) and pan in window pixels.
#[derive(Clone, Copy, Default)]
struct Gesture {
    zoom: f64,
    pan: [f64; 2],
}

struct State {
    iterate: Shader,
    present: Shader,
    compute: Pass,
    passes: Vec<Pass>,
    reference: Reference,
    next: Option<Reference>,
    center: [BigInt; 2],
    bits: usize,
    zoom: f64,
    direction: f64,
    last: Option<f32>,
    exploring: bool,
    sent: Option<(u64, usize)>,
    drawn: Option<[f64; 9]>,
}

// Call setup once, then frame on every redraw of your window.
fn setup() -> Result<State, Failure> {
    let header = format!("const ORBIT: u32 = {ORBIT}u;\\nconst PIXELS: u32 = {PIXELS}u;");

    let iterate = Shader::new([header.as_str(), ITERATE])?;
%%ITERATE_SETS%%

    // The library brings the palette that present.wgsl calls.
    let present = Shader::new([%%LIBRARY_ITEMS%%header.as_str(), LABEL, PRESENT])?;
%%PRESENT_SETS%%

    // The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
    let bits = (DEPTH * 10_f64.log2()).ceil() as usize + 80;
    let (cr, ci) = refine(&TARGET, bits)?;
    let home = Reference::start([cr, ci], bits);

    let compute = Pass::from_shader("iterate", &iterate);
    let draw = Pass::from_shader("present", &present);
    Ok(State {
        passes: vec![compute.clone(), draw],
        iterate,
        present,
        compute,
        center: [home.cr.clone(), home.ci.clone()],
        bits,
        reference: home,
        next: None,
        zoom: 0.0,
        direction: 1.0,
        last: None,
        exploring: false,
        sent: None,
        drawn: None,
    })
}

fn frame(
    state: &mut State,
    renderer: &Renderer,
    target: &impl Target,
    time: f32,
    size: [f32; 2],
    pointer: [f32; 2],
    gesture: Gesture,
) -> Result<(), Failure> {
    let delta = state
        .last
        .map_or(0.0, |last| f64::from((time - last).clamp(0.0, 0.1)));
    state.last = Some(time);

    // The compute pass fills at most PIXELS: the window, or the window with each side divided by a whole number.
    let [width, height] = [size[0].max(1.0) as u32, size[1].max(1.0) as u32];
    let shrink = (1..).find(|k| (width / k) * (height / k) <= PIXELS).unwrap_or(1);
    let image = [(width / shrink).max(1), (height / shrink).max(1)];
    let budget: u32 = state.iterate.get("view.budget")?;

    // The first scroll, pinch or drag ends the dive. From then on the view is the reader's.
    if gesture.zoom != 0.0 || gesture.pan != [0.0, 0.0] {
        state.exploring = true;
    }
    let deadline = Instant::now() + WORK;
    if state.exploring {
        explore(state, size, image, pointer, gesture, budget, deadline);
    } else {
        dive(state, image, delta, budget, deadline);
    }

    // A pixel is 2^pixel wide, and the view's center sits away from the reference by \`center\` pixels.
    let reference = &state.reference;
    let pixel = (START / f64::from(image[1])).log2() - state.zoom;
    let exponent = pixel.floor();
    let center = offset(&state.center, state.bits, reference, pixel);
    // The series covers every pixel within 2^radius of the reference.
    let radius = pixel + (half_diagonal(image) + center[0].hypot(center[1])).log2();
    let skip = reference.skip(radius);
    let [a, b, c] = reference.series[skip].map(Wide::exact);

    // The GPU gets the orbit again whenever it grew or another reference took over.
    if state.sent != Some((reference.id, reference.length)) {
        let bytes: Vec<u8> = reference.orbit.iter().flat_map(|value| value.to_le_bytes()).collect();
        state.iterate.set("orbit", bytes.as_slice())?;
        state.sent = Some((reference.id, reference.length));
    }
    state.iterate.set("view.image", image)?;
    state.iterate.set("view.exponent", exponent as i32)?;
    state.iterate.set("view.fraction", (pixel - exponent).exp2() as f32)?;
    state.iterate.set("view.center", [center[0] as f32, center[1] as f32])?;
    state.iterate.set("view.skip", skip as u32)?;
    state.iterate.set("view.length", reference.length as u32)?;
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
    let depth = state.zoom * 2_f64.log10() - START.log10();
    state.present.set("look.depth", depth as f32)?;

    // When nothing moved, the counts from the last frame still hold, so only the colors redraw.
    let view = [
        f64::from(image[0]),
        f64::from(image[1]),
        pixel,
        center[0],
        center[1],
        skip as f64,
        reference.length as f64,
        reference.id as f64,
        f64::from(budget),
    ];
    if state.drawn == Some(view) {
        renderer.render(&state.passes[1], target)?;
    } else {
        renderer.render(&state.passes, target)?;
    }
    state.drawn = Some(view);
    Ok(())
}

/// The zoom runs from the whole set down to 10^-DEPTH and back on its own, never
/// deeper than the reference orbit computed so far covers.
fn dive(state: &mut State, image: [u32; 2], delta: f64, budget: u32, deadline: Instant) {
    state.reference.grow(deadline);
    let deepest = DEPTH * 10_f64.log2() + START.log2();
    let zoom = (state.zoom + state.direction * SPEED * delta).clamp(0.0, deepest);
    if covers(&state.reference, image, zoom, [0.0, 0.0], budget) {
        state.zoom = zoom;
    }
    if state.zoom >= deepest {
        state.direction = -1.0;
    }
    if state.zoom <= 0.0 {
        state.direction = 1.0;
    }
}

/// The reader's view: the wheel or a pinch zooms around the pointer, and a drag pans.
/// When the view drifts away from the reference, or zooms past its digits, a new
/// reference starts where the view is heading and takes over once it covers the view.
fn explore(
    state: &mut State,
    size: [f32; 2],
    image: [u32; 2],
    pointer: [f32; 2],
    gesture: Gesture,
    budget: u32,
    deadline: Instant,
) {
    let pixel = (START / f64::from(image[1])).log2() - state.zoom;
    // The pointer and the drag, in image pixels with y pointing up.
    let scale = f64::from(image[1]) / f64::from(size[1]);
    let at = [
        f64::from(pointer[0] - size[0] / 2.0) * scale,
        f64::from(size[1] / 2.0 - pointer[1]) * scale,
    ];
    state.center = point_at(&state.center, state.bits, [-gesture.pan[0] * scale, gesture.pan[1] * scale], pixel);

    let deepest = DEPTH * 10_f64.log2() + START.log2();
    let zoom = (state.zoom + gesture.zoom).clamp(0.0, deepest);
    let after = (START / f64::from(image[1])).log2() - zoom;
    // The center needs a few more digits than the deepest pixel it has seen.
    if state.bits < (-after).ceil() as usize + 64 {
        state.center = state.center.clone().map(|value| value << 128);
        state.bits += 128;
    }
    // Zooming around the pointer keeps the point under it in place: the center moves
    // toward it by \`stay\` of the way, and the reference's offset moves with it.
    let stay = 1.0 - (state.zoom - zoom).exp2();
    let reference = &state.reference;
    let shift = (pixel - after).exp2() * stay;
    let [x, y] = offset(&state.center, state.bits, reference, after);
    let drift = [x + at[0] * shift, y + at[1] * shift];
    // Zooming in waits while the reference lacks the digits, is more than two views
    // from where the zoom lands, or hasn't grown far enough yet.
    let near = drift[0].hypot(drift[1]) <= 2.0 * f64::from(image[1]);
    let deep = reference.bits >= (-after).ceil() as usize + 40;
    let far = zoom > state.zoom && !(near && deep);
    if zoom < state.zoom || (!far && covers(reference, image, zoom, drift, budget)) {
        state.center = point_at(&state.center, state.bits, [at[0] * stay, at[1] * stay], pixel);
        state.zoom = zoom;
    }

    // A zoom in heads for the point under the pointer; anything else stays at the center.
    let now = (START / f64::from(image[1])).log2() - state.zoom;
    let aim = if gesture.zoom > 0.0 { at } else { [0.0, 0.0] };
    // The offset is the center minus the reference, so a reference at the aim sits at -aim.
    let away = |other: &Reference, limit: f64| {
        let [x, y] = offset(&state.center, state.bits, other, now);
        (x + aim[0]).hypot(y + aim[1]) > limit
    };
    // A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
    let finest = if far { after } else { now };
    let needs = far
        || away(&state.reference, f64::from(image[1]) / 4.0)
        || state.reference.bits < (-now).ceil() as usize + 120;
    let shallow = (-finest).ceil() as usize + 40;
    if needs && state.next.as_ref().is_none_or(|next| away(next, f64::from(image[1]) / 2.0) || next.bits < shallow) {
        let bits = (-finest).ceil() as usize + 200;
        let goal = point_at(&state.center, state.bits, aim, now);
        state.next = Some(Reference::start(align(goal, state.bits, bits), bits));
    }
    if let Some(mut next) = state.next.take() {
        next.grow(deadline);
        let distance = offset(&state.center, state.bits, &next, now);
        if covers(&next, image, state.zoom, distance, budget) {
            state.reference = next;
        } else {
            state.next = Some(next);
        }
    }
    state.reference.grow(deadline);
}

/// Whether this reference serves a view at this zoom: its orbit is complete, or long
/// enough for the series skip plus every pixel's budget.
fn covers(reference: &Reference, image: [u32; 2], zoom: f64, center: [f64; 2], budget: u32) -> bool {
    if reference.done {
        return true;
    }
    let pixel = (START / f64::from(image[1])).log2() - zoom;
    let radius = pixel + (half_diagonal(image) + center[0].hypot(center[1])).log2();
    reference.skip(radius) + budget as usize + 200 <= reference.length
}

fn half_diagonal(image: [u32; 2]) -> f64 {
    f64::from(image[0]).hypot(f64::from(image[1])) / 2.0
}

/// The point (x, y) pixels from \`center\`, as fixed point numbers at \`bits\`.
fn point_at(center: &[BigInt; 2], bits: usize, [x, y]: [f64; 2], pixel: f64) -> [BigInt; 2] {
    let scale = pixel + bits as f64;
    let lift = (scale.floor() - 40.0).max(0.0);
    let step = |value: f64| BigInt::from((value * (scale - lift).exp2()).round() as i64) << (lift as usize);
    [&center[0] + step(x), &center[1] + step(y)]
}

/// \`center\` minus the reference's c, in pixels.
fn offset(center: &[BigInt; 2], bits: usize, reference: &Reference, pixel: f64) -> [f64; 2] {
    let shared = bits.max(reference.bits);
    let [cr, ci] = align(center.clone(), bits, shared);
    let [rr, ri] = align([reference.cr.clone(), reference.ci.clone()], reference.bits, shared);
    let power = -(shared as f64) - pixel;
    [scaled(&(cr - rr), power), scaled(&(ci - ri), power)]
}

/// value * 2^power as a double, for a BigInt of any size.
fn scaled(value: &BigInt, power: f64) -> f64 {
    let drop = value.bits().saturating_sub(60);
    (value >> drop).to_f64().unwrap_or(0.0) * (power + drop as f64).exp2()
}

/// The same fixed point numbers at another precision.
fn align(values: [BigInt; 2], from: usize, to: usize) -> [BigInt; 2] {
    values.map(|value| if to >= from { value << (to - from) } else { value >> (from - to) })
}

/// The orbit of 0 under \`z^2 + c\` at \`bits\` of precision, rounded to \`f32\` for the GPU,
/// with Martin's series coefficients A, B and C at every iteration. \`reach\` holds the
/// log2 of the widest radius the series still covers at each iteration. \`grow\` adds
/// to it until a deadline; \`length\` is the last index computed so far.
struct Reference {
    id: u64,
    cr: BigInt,
    ci: BigInt,
    bits: usize,
    orbit: Vec<f32>,
    series: Vec<[Wide; 3]>,
    reach: Vec<f64>,
    length: usize,
    done: bool,
    z: [BigInt; 2],
    coefficients: [Wide; 4],
    saved: Option<(f64, f64, f64)>,
    checkpoint: usize,
}

/// The largest the first dropped term of the series may grow, relative to the first term.
const TOLERANCE: f64 = 1e-7;

impl Reference {
    fn start([cr, ci]: [BigInt; 2], bits: usize) -> Reference {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let mut reference = Reference {
            id: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            cr,
            ci,
            bits,
            orbit: vec![0.0; 2 * ORBIT],
            series: Vec::new(),
            reach: vec![f64::INFINITY; ORBIT],
            length: 0,
            done: false,
            z: [BigInt::ZERO, BigInt::ZERO],
            coefficients: [Wide::ZERO; 4],
            saved: None,
            checkpoint: 1,
        };
        reference.series.push([Wide::ZERO; 3]);
        reference.z = [reference.cr.clone(), reference.ci.clone()];
        reference.coefficients[0] = Wide::ONE;
        reference
    }

    fn grow(&mut self, deadline: Instant) {
        while !self.done && Instant::now() < deadline {
            for _ in 0..64 {
                if self.done {
                    break;
                }
                self.step();
            }
        }
    }

    /// Adds the next iteration of the orbit and the series.
    fn step(&mut self) {
        let n = self.length + 1;
        let [zr, zi] = &self.z;
        let (re, im) = (to_float(zr, self.bits), to_float(zi, self.bits));
        self.orbit[2 * n] = re as f32;
        self.orbit[2 * n + 1] = im as f32;
        let [mut a, mut b, mut c, mut d] = self.coefficients;
        self.series.push([a, b, c]);
        // The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
        let limit = (TOLERANCE.log2() + a.log2_size() - d.log2_size()) / 3.0;
        self.reach[n] = self.reach[n - 1].min(limit);
        self.length = n;
        // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
        // An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
        // and nothing after adds anything. Brent's method compares it with powers of two.
        let settled = self.saved.is_some_and(|(r, i, size)| {
            (re - r).hypot(im - i) < 1e-12 && (a.log2_size() - size).abs() < 1e-9
        });
        if n == ORBIT - 1 || re * re + im * im > 4.0 || a.log2_size() > self.bits as f64 - 40.0 || settled {
            self.done = true;
            return;
        }
        if n == self.checkpoint {
            self.saved = Some((re, im, a.log2_size()));
            self.checkpoint *= 2;
        }
        // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
        let twice = (2.0 * re, 2.0 * im);
        d = d.times(twice).add(a.multiply(c).times((2.0, 0.0)).add(b.multiply(b)));
        c = c.times(twice).add(a.multiply(b).times((2.0, 0.0)));
        b = b.times(twice).add(a.multiply(a));
        a = a.times(twice).add(Wide::ONE);
        self.coefficients = [a, b, c, d];
        let next = ((zr * zr - zi * zi) >> self.bits) + &self.cr;
        let imaginary = ((2 * zr * zi) >> self.bits) + &self.ci;
        self.z = [next, imaginary];
    }

    /// The last iteration the series covers for a view of this radius (log2).
    fn skip(&self, radius: f64) -> usize {
        // The last step reads one entry past skip, so skip stops one short of the end.
        let covered = self.reach[..self.length.max(1)].partition_point(|&reach| reach > radius);
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
import time as clock
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
# Until the first scroll, pinch or drag, the zoom goes in until the view is
# 10^-DEPTH tall, then back out.
DEPTH = 1000
# Zoom speed, in doublings per second.
SPEED = 1.5
# Height of the first view.
START = 3.0
# Room for the reference orbit, and for the pixels the compute pass iterates.
# A larger canvas renders at a lower resolution.
ORBIT = 1 << 16
PIXELS = 1 << 21
# Seconds per frame spent growing reference orbits, so a new one never stalls the view.
WORK = 0.006


# Call setup() once, then frame() on every redraw of your canvas. frame's gesture
# is what the wheel, a drag and a pinch did since the last frame: zoom in doublings
# (positive zooms in) and pan in canvas pixels.
def setup():
    header = f"const ORBIT: u32 = {ORBIT}u;\\nconst PIXELS: u32 = {PIXELS}u;"

    iterate = Shader([header, ITERATE])
%%ITERATE_SETS%%

    # The library brings the palette that present.wgsl calls.
    present = Shader([*LIBRARY, header, LABEL, PRESENT])
%%PRESENT_SETS%%

    # The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
    bits = math.ceil(DEPTH * math.log2(10)) + 80
    home = start_reference(refine(TARGET, bits), bits)

    compute = Pass.from_shader("iterate", iterate)
    draw = Pass.from_shader("present", present)
    return SimpleNamespace(
        iterate=iterate,
        present=present,
        compute=compute,
        passes=[compute, draw],
        reference=home,
        next=None,
        center=[home.cr, home.ci],
        bits=bits,
        zoom=0.0,
        direction=1.0,
        last=None,
        exploring=False,
        sent=None,
        drawn=None,
    )


def frame(state, renderer, target, time, size, pointer, gesture=None):
    gesture = gesture or SimpleNamespace(zoom=0.0, pan=[0.0, 0.0])
    delta = 0.0 if state.last is None else min(max(time - state.last, 0.0), 0.1)
    state.last = time

    # The compute pass fills at most PIXELS: the canvas, or the canvas with each side divided by a whole number.
    width, height = max(int(size[0]), 1), max(int(size[1]), 1)
    shrink = 1
    while (width // shrink) * (height // shrink) > PIXELS:
        shrink += 1
    image = [max(width // shrink, 1), max(height // shrink, 1)]
    budget = state.iterate.get("view.budget")

    # The first scroll, pinch or drag ends the dive. From then on the view is the reader's.
    if gesture.zoom or gesture.pan[0] or gesture.pan[1]:
        state.exploring = True
    deadline = clock.perf_counter() + WORK
    if state.exploring:
        explore(state, size, image, pointer, gesture, budget, deadline)
    else:
        dive(state, image, delta, budget, deadline)

    # A pixel is 2^pixel wide, and the view's center sits away from the reference by \`center\` pixels.
    reference = state.reference
    pixel = math.log2(START / image[1]) - state.zoom
    exponent = math.floor(pixel)
    center = offset(state, reference, pixel)
    # The series covers every pixel within 2^radius of the reference.
    radius = pixel + math.log2(math.hypot(*image) / 2 + math.hypot(*center))
    skip = skip_for(reference, radius)
    a, b, c = (exact(value) for value in reference.series[skip])

    # The GPU gets the orbit again whenever it grew or another reference took over.
    if state.sent != (id(reference), reference.length):
        state.iterate.set("orbit", reference.orbit.tobytes())
        state.sent = (id(reference), reference.length)
    state.iterate.set("view.image", image)
    state.iterate.set("view.exponent", exponent)
    state.iterate.set("view.fraction", 2 ** (pixel - exponent))
    state.iterate.set("view.center", center)
    state.iterate.set("view.skip", skip)
    state.iterate.set("view.length", reference.length)
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
    state.present.set("look.depth", state.zoom * math.log10(2) - math.log10(START))

    # When nothing moved, the counts from the last frame still hold, so only the colors redraw.
    view = (*image, pixel, *center, skip, reference.length, id(reference), budget)
    renderer.render(state.passes[1] if view == state.drawn else state.passes, target)
    state.drawn = view


# The zoom runs from the whole set down to 10^-DEPTH and back on its own, never
# deeper than the reference orbit computed so far covers.
def dive(state, image, delta, budget, deadline):
    grow(state.reference, deadline)
    deepest = DEPTH * math.log2(10) + math.log2(START)
    zoom = min(max(state.zoom + state.direction * SPEED * delta, 0.0), deepest)
    if covers(state.reference, image, zoom, [0.0, 0.0], budget):
        state.zoom = zoom
    if state.zoom >= deepest:
        state.direction = -1.0
    if state.zoom <= 0:
        state.direction = 1.0


# The reader's view: the wheel or a pinch zooms around the pointer, and a drag pans.
# When the view drifts away from the reference, or zooms past its digits, a new
# reference starts where the view is heading and takes over once it covers the view.
def explore(state, size, image, pointer, gesture, budget, deadline):
    pixel = math.log2(START / image[1]) - state.zoom
    # The pointer and the drag, in image pixels with y pointing up.
    scale = image[1] / size[1]
    at = [(pointer[0] - size[0] / 2) * scale, (size[1] / 2 - pointer[1]) * scale]
    state.center = point_at(state, [-gesture.pan[0] * scale, gesture.pan[1] * scale], pixel)

    deepest = DEPTH * math.log2(10) + math.log2(START)
    zoom = min(max(state.zoom + gesture.zoom, 0.0), deepest)
    after = math.log2(START / image[1]) - zoom
    # The center needs a few more digits than the deepest pixel it has seen.
    if state.bits < math.ceil(-after) + 64:
        state.center = [value << 128 for value in state.center]
        state.bits += 128
    # Zooming around the pointer keeps the point under it in place: the center moves
    # toward it by \`stay\` of the way, and the reference's offset moves with it.
    stay = 1 - 2 ** (state.zoom - zoom)
    reference = state.reference
    shift = 2 ** (pixel - after) * stay
    x, y = offset(state, reference, after)
    drift = [x + at[0] * shift, y + at[1] * shift]
    # Zooming in waits while the reference lacks the digits, is more than two views
    # from where the zoom lands, or hasn't grown far enough yet.
    near = math.hypot(*drift) <= 2 * image[1]
    deep = reference.bits >= math.ceil(-after) + 40
    far = zoom > state.zoom and not (near and deep)
    if zoom < state.zoom or (not far and covers(reference, image, zoom, drift, budget)):
        state.center = point_at(state, [at[0] * stay, at[1] * stay], pixel)
        state.zoom = zoom

    # A zoom in heads for the point under the pointer; anything else stays at the center.
    now = math.log2(START / image[1]) - state.zoom
    aim = at if gesture.zoom > 0 else [0.0, 0.0]

    # The offset is the center minus the reference, so a reference at the aim sits at -aim.
    def away(other, limit):
        x, y = offset(state, other, now)
        return math.hypot(x + aim[0], y + aim[1]) > limit

    # A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
    finest = after if far else now
    if far or away(reference, image[1] / 4) or reference.bits < math.ceil(-now) + 120:
        if state.next is None or away(state.next, image[1] / 2) or state.next.bits < math.ceil(-finest) + 40:
            bits = math.ceil(-finest) + 200
            state.next = start_reference(align(point_at(state, aim, now), state.bits, bits), bits)
    if state.next is not None:
        grow(state.next, deadline)
        if covers(state.next, image, state.zoom, offset(state, state.next, now), budget):
            state.reference, state.next = state.next, None
    grow(state.reference, deadline)


# Whether this reference serves a view at this zoom: its orbit is complete, or long
# enough for the series skip plus every pixel's budget.
def covers(reference, image, zoom, center, budget):
    if reference.done:
        return True
    pixel = math.log2(START / image[1]) - zoom
    radius = pixel + math.log2(math.hypot(*image) / 2 + math.hypot(*center))
    return skip_for(reference, radius) + budget + 200 <= reference.length


# The point (x, y) pixels from the view's center, as fixed point numbers at state.bits.
def point_at(state, point, pixel):
    scale = pixel + state.bits
    lift = max(0, math.floor(scale) - 40)
    return [value + (round(step * 2 ** (scale - lift)) << lift) for value, step in zip(state.center, point)]


# The view's center minus the reference's c, in pixels.
def offset(state, reference, pixel):
    bits = max(state.bits, reference.bits)
    cr, ci = align(state.center, state.bits, bits)
    rr, ri = align([reference.cr, reference.ci], reference.bits, bits)
    return [scaled(cr - rr, -bits - pixel), scaled(ci - ri, -bits - pixel)]


# value * 2^power as a float, for an int of any size.
def scaled(value, power):
    drop = max(0, abs(value).bit_length() - 60)
    return (value >> drop) * 2.0 ** (power + drop)


# The same fixed point numbers at another precision.
def align(values, start, end):
    return [value << (end - start) if end >= start else value >> (start - end) for value in values]


# The orbit of 0 under z^2 + c at \`bits\` of precision, rounded to f32 for the GPU,
# with Martin's series coefficients A, B and C at every iteration. The reach list
# holds the log2 of the widest radius the series still covers at each iteration.
# grow adds to it until a deadline; length is the last index computed so far.
def start_reference(c, bits):
    cr, ci = c
    return SimpleNamespace(
        cr=cr,
        ci=ci,
        bits=bits,
        orbit=array("f", [0.0]) * (2 * ORBIT),
        series=[(ZERO, ZERO, ZERO)],
        reach=[math.inf] * ORBIT,
        length=0,
        done=False,
        z=(cr, ci),
        coefficients=(ONE, ZERO, ZERO, ZERO),
        saved=None,
        checkpoint=1,
    )


def grow(reference, deadline):
    while not reference.done and clock.perf_counter() < deadline:
        for _ in range(64):
            if reference.done:
                break
            step(reference)


# Adds the next iteration of the orbit and the series.
def step(reference):
    n = reference.length + 1
    zr, zi = reference.z
    re, im = to_float(zr, reference.bits), to_float(zi, reference.bits)
    reference.orbit[2 * n], reference.orbit[2 * n + 1] = re, im
    a, b, c, d = reference.coefficients
    reference.series.append((a, b, c))
    # The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
    reference.reach[n] = min(reference.reach[n - 1], (math.log2(TOLERANCE) + log2_size(a) - log2_size(d)) / 3)
    reference.length = n
    # Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
    # An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
    # and nothing after adds anything. Brent's method compares it with powers of two.
    saved = reference.saved
    settled = saved and math.hypot(re - saved[0], im - saved[1]) < 1e-12 and abs(log2_size(a) - saved[2]) < 1e-9
    if n == ORBIT - 1 or re * re + im * im > 4 or log2_size(a) > reference.bits - 40 or settled:
        reference.done = True
        return
    if n == reference.checkpoint:
        reference.saved = (re, im, log2_size(a))
        reference.checkpoint *= 2
    # Each new coefficient reads the old values of the ones before it, so D updates first and A last.
    twice = (2 * re, 2 * im)
    d = add(times(d, *twice), add(times(multiply(a, c), 2, 0), multiply(b, b)))
    c = add(times(c, *twice), times(multiply(a, b), 2, 0))
    b = add(times(b, *twice), multiply(a, a))
    a = add(times(a, *twice), ONE)
    reference.coefficients = (a, b, c, d)
    shift = reference.bits
    reference.z = (((zr * zr - zi * zi) >> shift) + reference.cr, ((2 * zr * zi) >> shift) + reference.ci)


# The largest the first dropped term of the series may grow, relative to the first term.
TOLERANCE = 1e-7


# The last iteration the series covers for a view of this radius (log2).
def skip_for(reference, radius):
    # The last step reads one entry past skip, so skip stops one short of the end.
    low, high = 0, max(reference.length - 1, 0)
    while low < high:
        middle = (low + high + 1) // 2
        if reference.reach[middle] > radius:
            low = middle
        else:
            high = middle - 1
    return low



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

/// A spiral in Seahorse Valley. Newton's method refines these digits to the precision depth needs.
let target = Point(re: "-0.7432918908524302", im: "0.1312405523087976", preperiod: 24, period: 2)
/// Until the first scroll, pinch or drag, the zoom goes in until the view is
/// 10^-depth tall, then back out.
let depth = 1000.0
/// Zoom speed, in doublings per second.
let speed = 1.5
/// Height of the first view.
let start = 3.0
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger view renders at a lower resolution.
let orbitSize = 1 << 16
let pixels: UInt32 = 1 << 21
/// Seconds per frame spent growing reference orbits, so a new one never stalls the view.
let work = 0.006

/// What the wheel, a drag and a pinch did since the last frame: zoom in doublings
/// (positive zooms in) and pan in view pixels.
struct Gesture {
    var zoom = 0.0
    var pan: [Double] = [0, 0]
}

struct State {
    let iterate: Shader
    let present: Shader
    let compute: Pass
    let passes: [Pass]
    var reference: Reference
    var next: Reference?
    var center: [BigInt]
    var bits: Int
    var zoom = 0.0
    var direction = 1.0
    var last: Float?
    var exploring = false
    var sent: (ObjectIdentifier, Int)?
    var drawn: (ObjectIdentifier, [Double])?
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

    // The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
    let bits = Int((depth * log2(10)).rounded(.up)) + 80
    let (cr, ci) = refine(target, bits: bits)
    let home = Reference(c: [cr, ci], bits: bits)

    let compute = Pass.fromShader("iterate", iterate)
    let draw = Pass.fromShader("present", present)
    return State(
        iterate: iterate, present: present, compute: compute, passes: [compute, draw],
        reference: home, next: nil, center: [cr, ci], bits: bits
    )
}

func frame(
    state: inout State,
    renderer: Renderer,
    target: WindowTarget,
    time: Float,
    size: [Float],
    pointer: [Float],
    gesture: Gesture = Gesture()
) throws {
    let delta = state.last.map { Double(min(max(time - $0, 0), 0.1)) } ?? 0
    state.last = time

    // The compute pass fills at most \`pixels\` cells: the view, or the view with each side divided by a whole number.
    let width = max(UInt32(size[0]), 1)
    let height = max(UInt32(size[1]), 1)
    var shrink: UInt32 = 1
    while (width / shrink) * (height / shrink) > pixels {
        shrink += 1
    }
    let image = [max(width / shrink, 1), max(height / shrink, 1)]
    let budget = Int(try state.iterate.get("view.budget").float ?? 0)

    // The first scroll, pinch or drag ends the dive. From then on the view is the reader's.
    if gesture.zoom != 0 || gesture.pan != [0, 0] {
        state.exploring = true
    }
    let deadline = Date().addingTimeInterval(work)
    if state.exploring {
        explore(&state, size, image, pointer, gesture, budget, deadline)
    } else {
        dive(&state, image, delta, budget, deadline)
    }

    // A pixel is 2^pixel wide, and the view's center sits away from the reference by \`center\` pixels.
    let reference = state.reference
    let pixel = log2(start / Double(image[1])) - state.zoom
    let exponent = pixel.rounded(.down)
    let center = offset(state, reference, pixel)
    // The series covers every pixel within 2^radius of the reference.
    let radius = pixel + log2(halfDiagonal(image) + hypot(center[0], center[1]))
    let skip = reference.skip(radius)
    let coefficients = reference.series[skip].map(\\.exact)
    let (a, b, c) = (coefficients[0], coefficients[1], coefficients[2])

    // The GPU gets the orbit again whenever it grew or another reference took over.
    let sent = (ObjectIdentifier(reference), reference.length)
    if state.sent.map({ $0 != sent }) ?? true {
        try state.iterate.set("orbit", reference.orbit.withUnsafeBytes { Data($0) })
        state.sent = sent
    }
    try state.iterate.set("view.image", image)
    try state.iterate.set("view.exponent", Int32(exponent))
    try state.iterate.set("view.fraction", Float(exp2(pixel - exponent)))
    try state.iterate.set("view.center", center.map { Float($0) })
    try state.iterate.set("view.skip", UInt32(skip))
    try state.iterate.set("view.length", UInt32(reference.length))
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
    try state.present.set("look.depth", Float(state.zoom * log10(2) - log10(start)))

    // When nothing moved, the counts from the last frame still hold, so only the colors redraw.
    let numbers = [Double(image[0]), Double(image[1]), pixel, center[0], center[1], Double(skip), Double(reference.length), Double(budget)]
    let view = (ObjectIdentifier(reference), numbers)
    if state.drawn.map({ $0 == view }) ?? false {
        try renderer.render([state.passes[1]], target)
    } else {
        try renderer.render(state.passes, target)
    }
    state.drawn = view
}

/// The zoom runs from the whole set down to 10^-depth and back on its own, never
/// deeper than the reference orbit computed so far covers.
func dive(_ state: inout State, _ image: [UInt32], _ delta: Double, _ budget: Int, _ deadline: Date) {
    state.reference.grow(until: deadline)
    let deepest = depth * log2(10) + log2(start)
    let zoom = min(max(state.zoom + state.direction * speed * delta, 0), deepest)
    if covers(state.reference, image, zoom, [0, 0], budget) {
        state.zoom = zoom
    }
    if state.zoom >= deepest {
        state.direction = -1
    }
    if state.zoom <= 0 {
        state.direction = 1
    }
}

/// The reader's view: the wheel or a pinch zooms around the pointer, and a drag pans.
/// When the view drifts away from the reference, or zooms past its digits, a new
/// reference starts where the view is heading and takes over once it covers the view.
func explore(
    _ state: inout State, _ size: [Float], _ image: [UInt32], _ pointer: [Float],
    _ gesture: Gesture, _ budget: Int, _ deadline: Date
) {
    let pixel = log2(start / Double(image[1])) - state.zoom
    // The pointer and the drag, in image pixels with y pointing up.
    let scale = Double(image[1]) / Double(size[1])
    let at = [Double(pointer[0] - size[0] / 2) * scale, Double(size[1] / 2 - pointer[1]) * scale]
    state.center = pointAt(state, [-gesture.pan[0] * scale, gesture.pan[1] * scale], pixel)

    let deepest = depth * log2(10) + log2(start)
    let zoom = min(max(state.zoom + gesture.zoom, 0), deepest)
    let after = log2(start / Double(image[1])) - zoom
    // The center needs a few more digits than the deepest pixel it has seen.
    if state.bits < Int((-after).rounded(.up)) + 64 {
        state.center = state.center.map { $0 << 128 }
        state.bits += 128
    }
    // Zooming around the pointer keeps the point under it in place: the center moves
    // toward it by \`stay\` of the way, and the reference's offset moves with it.
    let stay = 1 - exp2(state.zoom - zoom)
    let reference = state.reference
    let shift = exp2(pixel - after) * stay
    let offsetAfter = offset(state, reference, after)
    let drift = [offsetAfter[0] + at[0] * shift, offsetAfter[1] + at[1] * shift]
    // Zooming in waits while the reference lacks the digits, is more than two views
    // from where the zoom lands, or hasn't grown far enough yet.
    let near = hypot(drift[0], drift[1]) <= 2 * Double(image[1])
    let deep = reference.bits >= Int((-after).rounded(.up)) + 40
    let far = zoom > state.zoom && !(near && deep)
    if zoom < state.zoom || (!far && covers(reference, image, zoom, drift, budget)) {
        state.center = pointAt(state, [at[0] * stay, at[1] * stay], pixel)
        state.zoom = zoom
    }

    // A zoom in heads for the point under the pointer; anything else stays at the center.
    let now = log2(start / Double(image[1])) - state.zoom
    let aim = gesture.zoom > 0 ? at : [0, 0]
    let current = state
    // The offset is the center minus the reference, so a reference at the aim sits at -aim.
    func away(_ other: Reference, _ limit: Double) -> Bool {
        let point = offset(current, other, now)
        return hypot(point[0] + aim[0], point[1] + aim[1]) > limit
    }
    // A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
    let finest = far ? after : now
    let shallow = Int((-finest).rounded(.up)) + 40
    if far || away(reference, Double(image[1]) / 4) || reference.bits < Int((-now).rounded(.up)) + 120 {
        if state.next.map({ away($0, Double(image[1]) / 2) || $0.bits < shallow }) ?? true {
            let bits = Int((-finest).rounded(.up)) + 200
            state.next = Reference(c: align(pointAt(state, aim, now), from: state.bits, to: bits), bits: bits)
        }
    }
    if let next = state.next {
        next.grow(until: deadline)
        if covers(next, image, state.zoom, offset(state, next, now), budget) {
            state.reference = next
            state.next = nil
        }
    }
    state.reference.grow(until: deadline)
}

/// Whether this reference serves a view at this zoom: its orbit is complete, or long
/// enough for the series skip plus every pixel's budget.
func covers(_ reference: Reference, _ image: [UInt32], _ zoom: Double, _ center: [Double], _ budget: Int) -> Bool {
    if reference.done {
        return true
    }
    let pixel = log2(start / Double(image[1])) - zoom
    let radius = pixel + log2(halfDiagonal(image) + hypot(center[0], center[1]))
    return reference.skip(radius) + budget + 200 <= reference.length
}

func halfDiagonal(_ image: [UInt32]) -> Double {
    hypot(Double(image[0]), Double(image[1])) / 2
}

/// The point (x, y) pixels from the view's center, as fixed point numbers at state.bits.
func pointAt(_ state: State, _ point: [Double], _ pixel: Double) -> [BigInt] {
    let scale = pixel + Double(state.bits)
    let lift = max(0, Int(scale.rounded(.down)) - 40)
    return zip(state.center, point).map { value, step in
        value + (BigInt(Int64((step * exp2(scale - Double(lift))).rounded())) << lift)
    }
}

/// The view's center minus the reference's c, in pixels.
func offset(_ state: State, _ reference: Reference, _ pixel: Double) -> [Double] {
    let bits = max(state.bits, reference.bits)
    let center = align(state.center, from: state.bits, to: bits)
    let c = align(reference.c, from: reference.bits, to: bits)
    let power = -Double(bits) - pixel
    return [scaled(center[0] - c[0], power), scaled(center[1] - c[1], power)]
}

/// value * 2^power as a Double, for a BigInt of any size.
func scaled(_ value: BigInt, _ power: Double) -> Double {
    let drop = max(0, value.magnitude.bitWidth - 60)
    return Double(value >> drop) * exp2(power + Double(drop))
}

/// The same fixed point numbers at another precision.
func align(_ values: [BigInt], from: Int, to: Int) -> [BigInt] {
    values.map { to >= from ? $0 << (to - from) : $0 >> (from - to) }
}

/// The orbit of 0 under \`z^2 + c\` at \`bits\` of precision, rounded to Float for the GPU,
/// with Martin's series coefficients A, B and C at every iteration. \`reach\` holds the
/// log2 of the widest radius the series still covers at each iteration. \`grow\` adds
/// to it until a deadline; \`length\` is the last index computed so far.
final class Reference {
    let c: [BigInt]
    let bits: Int
    var orbit: [Float]
    var series: [[Wide]] = [[.zero, .zero, .zero]]
    var reach: [Double]
    var length = 0
    var done = false
    var z: [BigInt]
    var coefficients: [Wide] = [.one, .zero, .zero, .zero]
    var saved: (Double, Double, Double)?
    var checkpoint = 1

    init(c: [BigInt], bits: Int) {
        self.c = c
        self.bits = bits
        orbit = [Float](repeating: 0, count: 2 * orbitSize)
        reach = [Double](repeating: .infinity, count: orbitSize)
        z = c
    }

    func grow(until deadline: Date) {
        while !done && Date() < deadline {
            for _ in 0..<64 where !done {
                step()
            }
        }
    }

    /// Adds the next iteration of the orbit and the series.
    func step() {
        let n = length + 1
        let (re, im) = (toDouble(z[0], bits: bits), toDouble(z[1], bits: bits))
        orbit[2 * n] = Float(re)
        orbit[2 * n + 1] = Float(im)
        var (a, b, c, d) = (coefficients[0], coefficients[1], coefficients[2], coefficients[3])
        series.append([a, b, c])
        // The series holds while the first dropped term, D * delta^4, stays below tolerance * A * delta.
        reach[n] = min(reach[n - 1], (log2(tolerance) + a.log2Size - d.log2Size) / 3)
        length = n
        // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
        // An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
        // and nothing after adds anything. Brent's method compares it with powers of two.
        let settled = saved.map { hypot(re - $0.0, im - $0.1) < 1e-12 && abs(a.log2Size - $0.2) < 1e-9 } ?? false
        if n == orbitSize - 1 || re * re + im * im > 4 || a.log2Size > Double(bits - 40) || settled {
            done = true
            return
        }
        if n == checkpoint {
            saved = (re, im, a.log2Size)
            checkpoint *= 2
        }
        // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
        let (twiceRe, twiceIm) = (2 * re, 2 * im)
        d = d.times(twiceRe, twiceIm).adding(a.multiplied(by: c).times(2, 0).adding(b.multiplied(by: b)))
        c = c.times(twiceRe, twiceIm).adding(a.multiplied(by: b).times(2, 0))
        b = b.times(twiceRe, twiceIm).adding(a.multiplied(by: a))
        a = a.times(twiceRe, twiceIm).adding(.one)
        coefficients = [a, b, c, d]
        z = [((z[0] * z[0] - z[1] * z[1]) >> bits) + self.c[0], ((2 * z[0] * z[1]) >> bits) + self.c[1]]
    }

    /// The last iteration the series covers for a view of this radius (log2).
    func skip(_ radius: Double) -> Int {
        // The last step reads one entry past skip, so skip stops one short of the end.
        var (low, high) = (0, max(length - 1, 0))
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
import kotlin.math.roundToLong
import org.fragmentcolor.*

%%LIBRARY_LINE%%

/** A Misiurewicz point: the orbit of 0 lands on a cycle of [period] after [preperiod] steps. */
class Point(val re: String, val im: String, val preperiod: Int, val period: Int)

/** A spiral in Seahorse Valley. Newton's method refines these digits to the precision DEPTH needs. */
private val TARGET = Point("-0.7432918908524302", "0.1312405523087976", preperiod = 24, period = 2)
/** Until the first scroll, pinch or drag, the zoom goes in until the view is 10^-DEPTH tall, then back out. */
private const val DEPTH = 1000.0
/** Zoom speed, in doublings per second. */
private const val SPEED = 1.5
/** Height of the first view. */
private const val START = 3.0
/** Room for the reference orbit, and for the pixels the compute pass iterates. */
/** A larger view renders at a lower resolution. */
private const val ORBIT = 1 shl 16
private val PIXELS = 1u shl 21
/** Nanoseconds per frame spent growing reference orbits, so a new one never stalls the view. */
private const val WORK = 6_000_000L

/**
 * What the wheel, a drag and a pinch did since the last frame: zoom in doublings
 * (positive zooms in) and pan in view pixels.
 */
data class Gesture(val zoom: Double = 0.0, val pan: List<Double> = listOf(0.0, 0.0))

class State(
    val iterate: Shader,
    val present: Shader,
    val compute: Pass,
    val passes: List<Pass>,
    var reference: Reference,
    var center: List<BigInteger>,
    var bits: Int,
) {
    var next: Reference? = null
    var zoom = 0.0
    var direction = 1.0
    var last: Float? = null
    var exploring = false
    var sent: Pair<Reference, Int>? = null
    var drawn: Pair<Reference, List<Double>>? = null
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

    // The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
    val bits = ceil(DEPTH * log2(10.0)).toInt() + 80
    val (cr, ci) = refine(TARGET, bits)
    val home = Reference(listOf(cr, ci), bits)

    val compute = Pass.fromShader("iterate", iterate)
    val draw = Pass.fromShader("present", present)
    return State(iterate, present, compute, listOf(compute, draw), home, listOf(cr, ci), bits)
}

fun frame(
    state: State,
    renderer: Renderer,
    target: WindowTarget,
    time: Float,
    size: List<Float>,
    pointer: List<Float>,
    gesture: Gesture = Gesture(),
) {
    val delta = state.last?.let { min(max(time - it, 0f), 0.1f).toDouble() } ?: 0.0
    state.last = time

    // The compute pass fills at most PIXELS: the view, or the view with each side divided by a whole number.
    val width = maxOf(size[0].toUInt(), 1u)
    val height = maxOf(size[1].toUInt(), 1u)
    var shrink = 1u
    while ((width / shrink) * (height / shrink) > PIXELS) shrink++
    val image = listOf(maxOf(width / shrink, 1u), maxOf(height / shrink, 1u))
    val budget = (state.iterate.get("view.budget").float ?: 0f).toInt()

    // The first scroll, pinch or drag ends the dive. From then on the view is the reader's.
    if (gesture.zoom != 0.0 || gesture.pan != listOf(0.0, 0.0)) state.exploring = true
    val deadline = System.nanoTime() + WORK
    if (state.exploring) {
        explore(state, size, image, pointer, gesture, budget, deadline)
    } else {
        dive(state, image, delta, budget, deadline)
    }

    // A pixel is 2^pixel wide, and the view's center sits away from the reference by \`center\` pixels.
    val reference = state.reference
    val pixel = log2(START / image[1].toDouble()) - state.zoom
    val exponent = floor(pixel)
    val center = offset(state, reference, pixel)
    // The series covers every pixel within 2^radius of the reference.
    val radius = pixel + log2(halfDiagonal(image) + hypot(center[0], center[1]))
    val skip = reference.skip(radius)
    val (a, b, c) = reference.series[skip].map { it.exact() }

    // The GPU gets the orbit again whenever it grew or another reference took over.
    val sent = Pair(reference, reference.length)
    if (state.sent != sent) {
        val bytes = ByteBuffer.allocate(4 * reference.orbit.size).order(ByteOrder.LITTLE_ENDIAN)
        reference.orbit.forEach { bytes.putFloat(it) }
        state.iterate.set("orbit", bytes.array())
        state.sent = sent
    }
    state.iterate.set("view.image", image)
    state.iterate.set("view.exponent", exponent.toInt())
    state.iterate.set("view.fraction", 2.0.pow(pixel - exponent).toFloat())
    state.iterate.set("view.center", center.map { it.toFloat() })
    state.iterate.set("view.skip", skip.toUInt())
    state.iterate.set("view.length", reference.length.toUInt())
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
    state.present.set("look.depth", (state.zoom * log10(2.0) - log10(START)).toFloat())

    // When nothing moved, the counts from the last frame still hold, so only the colors redraw.
    val numbers = listOf(image[0].toDouble(), image[1].toDouble(), pixel, center[0], center[1])
    val view = Pair(reference, numbers + listOf(skip.toDouble(), reference.length.toDouble(), budget.toDouble()))
    if (state.drawn == view) {
        renderer.render(state.passes[1], target)
    } else {
        renderer.render(state.passes, target)
    }
    state.drawn = view
}

/**
 * The zoom runs from the whole set down to 10^-DEPTH and back on its own, never
 * deeper than the reference orbit computed so far covers.
 */
fun dive(state: State, image: List<UInt>, delta: Double, budget: Int, deadline: Long) {
    state.reference.grow(deadline)
    val deepest = DEPTH * log2(10.0) + log2(START)
    val zoom = (state.zoom + state.direction * SPEED * delta).coerceIn(0.0, deepest)
    if (covers(state.reference, image, zoom, listOf(0.0, 0.0), budget)) state.zoom = zoom
    if (state.zoom >= deepest) state.direction = -1.0
    if (state.zoom <= 0) state.direction = 1.0
}

/**
 * The reader's view: the wheel or a pinch zooms around the pointer, and a drag pans.
 * When the view drifts away from the reference, or zooms past its digits, a new
 * reference starts where the view is heading and takes over once it covers the view.
 */
fun explore(
    state: State,
    size: List<Float>,
    image: List<UInt>,
    pointer: List<Float>,
    gesture: Gesture,
    budget: Int,
    deadline: Long,
) {
    val pixel = log2(START / image[1].toDouble()) - state.zoom
    // The pointer and the drag, in image pixels with y pointing up.
    val scale = image[1].toDouble() / size[1]
    val at = listOf((pointer[0] - size[0] / 2) * scale, (size[1] / 2 - pointer[1]) * scale)
    state.center = pointAt(state, listOf(-gesture.pan[0] * scale, gesture.pan[1] * scale), pixel)

    val deepest = DEPTH * log2(10.0) + log2(START)
    val zoom = (state.zoom + gesture.zoom).coerceIn(0.0, deepest)
    val after = log2(START / image[1].toDouble()) - zoom
    // The center needs a few more digits than the deepest pixel it has seen.
    if (state.bits < ceil(-after).toInt() + 64) {
        state.center = state.center.map { it.shiftLeft(128) }
        state.bits += 128
    }
    // Zooming around the pointer keeps the point under it in place: the center moves
    // toward it by \`stay\` of the way, and the reference's offset moves with it.
    val stay = 1 - 2.0.pow(state.zoom - zoom)
    val reference = state.reference
    val shift = 2.0.pow(pixel - after) * stay
    val (x, y) = offset(state, reference, after)
    val drift = listOf(x + at[0] * shift, y + at[1] * shift)
    // Zooming in waits while the reference lacks the digits, is more than two views
    // from where the zoom lands, or hasn't grown far enough yet.
    val near = hypot(drift[0], drift[1]) <= 2 * image[1].toDouble()
    val deep = reference.bits >= ceil(-after).toInt() + 40
    val far = zoom > state.zoom && !(near && deep)
    if (zoom < state.zoom || (!far && covers(reference, image, zoom, drift, budget))) {
        state.center = pointAt(state, listOf(at[0] * stay, at[1] * stay), pixel)
        state.zoom = zoom
    }

    // A zoom in heads for the point under the pointer; anything else stays at the center.
    val now = log2(START / image[1].toDouble()) - state.zoom
    val aim = if (gesture.zoom > 0) at else listOf(0.0, 0.0)
    // The offset is the center minus the reference, so a reference at the aim sits at -aim.
    fun away(other: Reference, limit: Double): Boolean {
        val point = offset(state, other, now)
        return hypot(point[0] + aim[0], point[1] + aim[1]) > limit
    }
    // A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
    val finest = if (far) after else now
    if (far || away(reference, image[1].toDouble() / 4) || reference.bits < ceil(-now).toInt() + 120) {
        if (state.next?.let { away(it, image[1].toDouble() / 2) || it.bits < ceil(-finest).toInt() + 40 } != false) {
            val bits = ceil(-finest).toInt() + 200
            state.next = Reference(align(pointAt(state, aim, now), state.bits, bits), bits)
        }
    }
    state.next?.let { next ->
        next.grow(deadline)
        if (covers(next, image, state.zoom, offset(state, next, now), budget)) {
            state.reference = next
            state.next = null
        }
    }
    state.reference.grow(deadline)
}

/**
 * Whether this reference serves a view at this zoom: its orbit is complete, or long
 * enough for the series skip plus every pixel's budget.
 */
fun covers(reference: Reference, image: List<UInt>, zoom: Double, center: List<Double>, budget: Int): Boolean {
    if (reference.done) return true
    val pixel = log2(START / image[1].toDouble()) - zoom
    val radius = pixel + log2(halfDiagonal(image) + hypot(center[0], center[1]))
    return reference.skip(radius) + budget + 200 <= reference.length
}

fun halfDiagonal(image: List<UInt>): Double = hypot(image[0].toDouble(), image[1].toDouble()) / 2

/** The point (x, y) pixels from the view's center, as fixed point numbers at state.bits. */
fun pointAt(state: State, point: List<Double>, pixel: Double): List<BigInteger> {
    val scale = pixel + state.bits
    val lift = max(0, floor(scale).toInt() - 40)
    return state.center.zip(point) { value, step ->
        value + BigInteger.valueOf((step * 2.0.pow(scale - lift)).roundToLong()).shiftLeft(lift)
    }
}

/** The view's center minus the reference's c, in pixels. */
fun offset(state: State, reference: Reference, pixel: Double): List<Double> {
    val bits = max(state.bits, reference.bits)
    val center = align(state.center, state.bits, bits)
    val c = align(reference.c, reference.bits, bits)
    val power = -bits - pixel
    return listOf(scaled(center[0] - c[0], power), scaled(center[1] - c[1], power))
}

/** value * 2^power as a Double, for a BigInteger of any size. */
fun scaled(value: BigInteger, power: Double): Double {
    val drop = max(0, value.bitLength() - 60)
    return value.shiftRight(drop).toDouble() * 2.0.pow(power + drop)
}

/** The same fixed point numbers at another precision. */
fun align(values: List<BigInteger>, from: Int, to: Int): List<BigInteger> =
    values.map { if (to >= from) it.shiftLeft(to - from) else it.shiftRight(from - to) }

/**
 * The orbit of 0 under z^2 + c at [bits] of precision, rounded to Float for the GPU,
 * with Martin's series coefficients A, B and C at every iteration. [reach] holds the
 * log2 of the widest radius the series still covers at each iteration. [grow] adds
 * to it until a deadline; [length] is the last index computed so far.
 */
class Reference(val c: List<BigInteger>, val bits: Int) {
    val orbit = FloatArray(2 * ORBIT)
    val series = mutableListOf(listOf(Wide.ZERO, Wide.ZERO, Wide.ZERO))
    val reach = DoubleArray(ORBIT) { Double.POSITIVE_INFINITY }
    var length = 0
    var done = false
    private var z = c
    private var coefficients = listOf(Wide.ONE, Wide.ZERO, Wide.ZERO, Wide.ZERO)
    private var saved: Triple<Double, Double, Double>? = null
    private var checkpoint = 1

    fun grow(deadline: Long) {
        while (!done && System.nanoTime() < deadline) {
            repeat(64) { if (!done) step() }
        }
    }

    /** Adds the next iteration of the orbit and the series. */
    private fun step() {
        val n = length + 1
        val re = toDouble(z[0], bits)
        val im = toDouble(z[1], bits)
        orbit[2 * n] = re.toFloat()
        orbit[2 * n + 1] = im.toFloat()
        var (a, b, c, d) = coefficients
        series.add(listOf(a, b, c))
        // The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
        reach[n] = min(reach[n - 1], (log2(TOLERANCE) + a.log2Size() - d.log2Size()) / 3)
        length = n
        // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
        // An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
        // and nothing after adds anything. Brent's method compares it with powers of two.
        val settled = saved?.let { (r, i, size) -> hypot(re - r, im - i) < 1e-12 && abs(a.log2Size() - size) < 1e-9 } ?: false
        if (n == ORBIT - 1 || re * re + im * im > 4 || a.log2Size() > bits - 40 || settled) {
            done = true
            return
        }
        if (n == checkpoint) {
            saved = Triple(re, im, a.log2Size())
            checkpoint *= 2
        }
        // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
        d = d.times(2 * re, 2 * im) + ((a * c).times(2.0, 0.0) + b * b)
        c = c.times(2 * re, 2 * im) + (a * b).times(2.0, 0.0)
        b = b.times(2 * re, 2 * im) + a * a
        a = a.times(2 * re, 2 * im) + Wide.ONE
        coefficients = listOf(a, b, c, d)
        val (zr, zi) = z
        z = listOf(
            zr.multiply(zr).subtract(zi.multiply(zi)).shiftRight(bits).add(this.c[0]),
            zr.multiply(zi).shiftLeft(1).shiftRight(bits).add(this.c[1]),
        )
    }

    /** The last iteration the series covers for a view of this radius (log2). */
    fun skip(radius: Double): Int {
        var low = 0
        // The last step reads one entry past skip, so skip stops one short of the end.
        var high = maxOf(length - 1, 0)
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
    "A dive into a spiral of the Mandelbrot set, down to 1e-1000 and back. Scroll or pinch to zoom and drag to look around: a BigInt reference orbit follows the view, and a compute pass iterates every pixel in f32.",
  category: "Compute",
  thumbnail: "/playground/thumbs/mandelbrot.webp",
  kind: "compute",
  requires: "webgpu",
  needs: RELEASE,
  gestures: true,
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
