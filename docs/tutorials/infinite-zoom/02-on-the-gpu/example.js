// Infinite Mandelbrot zoom, step 2: the same loop on the GPU.
//
// A fragment shader runs the escape-time loop for every pixel in parallel, in
// f32. The view zooms toward a spiral in Seahorse Valley and back out, and the
// pointer's vertical position picks the depth while it moves. Past 1e-5, f32 rounding
// breaks the picture into blocks.

import { Shader } from "fragmentcolor";

const SHADER = `
// #region: shader
struct View {
  resolution: vec2<f32>,
  center: vec2<f32>,
  // The view's height in the complex plane, which is about 10^-depth.
  height: f32,
  depth: f32,
}

@group(0) @binding(0) var<uniform> view: View;

const LIMIT: u32 = 1000u;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  // This pixel's c. Screen y counts down while the imaginary axis points up, hence the -1.
  let offset = (position.xy - view.resolution * 0.5) * vec2<f32>(1.0, -1.0);
  let c = view.center + offset * view.height / view.resolution.y;
  var color = vec3<f32>(0.02, 0.02, 0.05);
  var z = vec2<f32>(0.0);
  for (var n = 1u; n <= LIMIT; n++) {
    z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
    let size2 = dot(z, z);
    if (size2 > 65536.0) {
      // The distance past the escape radius smooths the count.
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
// A spiral in Seahorse Valley, the point the zoom approaches.
const TARGET = [-0.7432918908524302, 0.1312405523087976];
// The zoom goes in until the view is 10^-DEPTH tall, then back out.
const DEPTH = 7;
// Zoom speed, in doublings per second.
const SPEED = 2;
// Height of the first view.
const START = 3;
// #endregion: constants

// #region: setup
export async function setup(renderer, target) {
  const shader = await Shader.fetch(["gradient/palette_iq", LABEL, SHADER]);
  shader.set("view.center", TARGET);
  const state = { shader, zoom: 0, direction: 1, last: null, pointer: null, moved: -Infinity };
  return state;
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, size, pointer) {
  const zoom = steer(state, time, size, pointer);
  const height = START * 2 ** -zoom;
  state.shader.set("view.resolution", size);
  state.shader.set("view.height", height);
  state.shader.set("view.depth", -Math.log10(height));
  renderer.render(state.shader, target);
}
// #endregion: frame

// #region: steer
// The zoom runs from the whole set down to 10^-DEPTH and back on its own.
// While the pointer moves, and for two seconds after, its height sets the speed:
// the view dives above the middle and climbs below it. Returns the zoom in doublings.
function steer(state, time, size, [x, y]) {
  // A clock that starts over, as when a paused page resumes, forgets the pointer.
  if (state.last !== null && time < state.last) state.moved = -Infinity;
  // The first frame counts the whole clock, so a frame drawn at time t shows the dive at t.
  const delta = state.last === null ? time : Math.min(Math.max(time - state.last, 0), 0.1);
  state.last = time;
  const deepest = DEPTH * Math.log2(10) + Math.log2(START);
  if (state.pointer && (x !== state.pointer[0] || y !== state.pointer[1])) state.moved = time;
  state.pointer = [x, y];
  if (time - state.moved < 2) {
    // The speed grows toward the edges, up to four times SPEED, and a band across the middle holds still.
    const lean = 1 - (2 * y) / size[1];
    const push = (Math.sign(lean) * Math.max(Math.abs(lean) - 0.1, 0)) / 0.9;
    state.zoom += push * 4 * SPEED * delta;
  } else {
    state.zoom += state.direction * SPEED * delta;
    if (state.zoom >= deepest) state.direction = -1;
    if (state.zoom <= 0) state.direction = 1;
  }
  state.zoom = Math.min(Math.max(state.zoom, 0), deepest);
  return state.zoom;
}
// #endregion: steer
