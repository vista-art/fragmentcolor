// Infinite zoom, step 3: perturbation.
//
// One reference orbit, computed once with BigInt at the precision the deepest
// view needs, sits in a storage buffer. Each pixel iterates only its small f32
// difference from that orbit, and rebases onto the start of the orbit when it
// drifts closer to 0 than to the reference. The zoom now passes 1e-30 and
// breaks down near 1e-35, where a pixel's width falls below the smallest f32.

import { Shader } from "fragmentcolor";

const SHADER = `
// #region: shader
struct View {
  resolution: vec2<f32>,
  // The width of one pixel in the complex plane, and the view's depth for the label.
  pixel: f32,
  depth: f32,
  // The last index of the reference orbit.
  length: u32,
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage, read> orbit: array<vec2<f32>, ORBIT>;

const LIMIT: u32 = 3000u;

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  // How far this pixel's c sits from the reference's c.
  let dc = (position.xy - view.resolution * 0.5) * vec2<f32>(1.0, -1.0) * view.pixel;
  var color = vec3<f32>(0.02, 0.02, 0.05);
  var z = vec2<f32>(0.0); // the pixel's difference from the reference orbit
  var m = 0u; // where the pixel is along the reference orbit
  for (var n = 1u; n <= LIMIT; n++) {
    z = 2.0 * mul(orbit[m], z) + mul(z, z) + dc;
    m += 1u;
    let full = orbit[m] + z;
    let size2 = dot(full, full);
    if (size2 > 65536.0) {
      let count = f32(n) + 1.0 - log2(log2(size2) / 16.0);
      color = palette_iq(
        count * 0.02 + 0.6,
        vec3<f32>(0.5),
        vec3<f32>(0.5),
        vec3<f32>(1.0),
        vec3<f32>(0.0, 0.1, 0.2),
      );
      break;
    }
    // Rebase when the pixel is closer to 0 than to the reference, or reaches the end of the orbit.
    if (size2 < dot(z, z) || m == view.length) {
      z = full;
      m = 0u;
    }
  }
  return vec4<f32>(print_depth(color, position.xy, view.resolution, view.depth), 1.0);
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
// #endregion: shader
`;

// Prints the view's depth in a corner of the canvas.
const LABEL = `
// Prints "1e-<depth>" in the bottom left corner: white with a dark shadow.
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
}`;

// #region: constants
// A Misiurewicz point in Seahorse Valley: the orbit of 0 lands on a cycle of
// period 2 after 24 steps. Newton's method refines these digits to the precision DEPTH needs.
const TARGET = { re: "-0.7432918908524302", im: "0.1312405523087976", preperiod: 24, period: 2 };
// The zoom goes in until the view is 10^-DEPTH tall, then back out.
const DEPTH = 45;
// Zoom speed, in doublings per second.
const SPEED = 4;
// Height of the first view.
const START = 3;
// Room for the reference orbit.
const ORBIT = 3001;
// #endregion: constants

// #region: setup
export async function setup(renderer, target) {
  const shader = await Shader.fetch([`const ORBIT: u32 = ${ORBIT}u;`, "gradient/palette_iq", LABEL, SHADER]);
  // Compute the reference orbit once and send it to the GPU.
  const { orbit, length } = reference(TARGET, DEPTH);
  shader.set("orbit", new Uint8Array(orbit.buffer));
  shader.set("view.length", length);
  const state = { shader, zoom: 0, direction: 1, last: null, pointer: null, moved: -Infinity };
  return state;
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, size, pointer) {
  const zoom = steer(state, time, size, pointer);
  const height = START * 2 ** -zoom;
  state.shader.set("view.resolution", size);
  state.shader.set("view.pixel", height / size[1]);
  state.shader.set("view.depth", -Math.log10(height));
  renderer.render(state.shader, target);
}
// #endregion: frame

// #region: reference
// The orbit of 0 under z^2 + c, computed with BigInt at the precision the
// deepest view needs and rounded to f32 for the GPU. Rounding errors grow like
// the derivative dz/dc, so the loop ends before they reach 2^-40.
function reference(target, depth) {
  const bits = Math.ceil(depth * Math.log2(10)) + 80;
  const shift = BigInt(bits);
  const [cr, ci] = refine(target, bits);
  const orbit = new Float32Array(2 * ORBIT);
  let [zr, zi] = [0n, 0n];
  let [dr, di] = [0, 0];
  for (let n = 0; n < ORBIT; n++) {
    const [re, im] = [toFloat(zr, bits), toFloat(zi, bits)];
    orbit[2 * n] = re;
    orbit[2 * n + 1] = im;
    if (re * re + im * im > 4 || Math.log2(Math.hypot(dr, di)) > bits - 40) return { orbit, length: n };
    [dr, di] = [2 * (re * dr - im * di) + 1, 2 * (re * di + im * dr)];
    const next = ((zr * zr - zi * zi) >> shift) + cr;
    zi = ((2n * zr * zi) >> shift) + ci;
    zr = next;
  }
  return { orbit, length: ORBIT - 1 };
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
// #endregion: reference

// #region: steer
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
// #endregion: steer
