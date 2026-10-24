// Infinite zoom, step 2: the same loop on the GPU.
//
// A fragment shader runs the escape-time loop for every pixel in parallel, in
// f32. The view zooms toward a spiral in Seahorse Valley and back out, and the
// pointer's height picks the depth while it moves. Past 1e-5, f32 rounding
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
  // This pixel's c. Rows grow downward, and the imaginary axis points up.
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
}`;

// #region: constants
// A spiral in Seahorse Valley, the point the zoom dives toward.
const TARGET = [-0.7432918908524302, 0.1312405523087976];
// The zoom dives until the view is 10^-DEPTH tall, then climbs back.
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
  const state = { shader, zoom: 0, direction: 1, last: null, pointer: [0.5, 0.5], moved: -Infinity };
  follow(state);
  return state;
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, size) {
  const zoom = steer(state);
  const height = START * 2 ** -zoom;
  state.shader.set("view.resolution", size);
  state.shader.set("view.height", height);
  state.shader.set("view.depth", -Math.log10(height));
  renderer.render(state.shader, target);
}
// #endregion: frame

// #region: steer
// The zoom runs from the whole set down to 10^-DEPTH and back on its own. Moving
// the pointer takes over for two seconds: its height picks the depth, deepest at
// the top. Returns the zoom in doublings.
function steer(state) {
  const now = performance.now() / 1000;
  const delta = state.last === null ? 0 : Math.min(now - state.last, 0.1);
  state.last = now;
  const deepest = DEPTH * Math.log2(10) + Math.log2(START);
  if (now - state.moved < 2) {
    const goal = (1 - state.pointer[1]) * deepest;
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

// Keeps the pointer's position over any canvas, as fractions of its size.
function follow(state) {
  addEventListener("pointermove", (event) => {
    const canvas = event.composedPath().find((node) => node instanceof HTMLCanvasElement);
    if (!canvas) return;
    const box = canvas.getBoundingClientRect();
    state.pointer = [(event.clientX - box.left) / box.width, (event.clientY - box.top) / box.height];
    state.moved = performance.now() / 1000;
  });
}
