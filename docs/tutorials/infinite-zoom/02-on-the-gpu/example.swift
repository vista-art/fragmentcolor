// Infinite Mandelbrot zoom, step 2: the same loop on the GPU.
//
// A fragment shader runs the escape-time loop for every pixel in parallel, in
// f32. The view zooms toward a spiral in Seahorse Valley and back out, and the
// pointer's vertical position picks the depth while it moves. Past 1e-5, f32 rounding
// breaks the picture into blocks.
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate.

import Foundation
import FragmentColor

let shaderSource = """
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
"""

/// Prints the view's depth in a corner of the screen.
let label = """
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
}
"""

// #region: constants
/// A spiral in Seahorse Valley, the point the zoom approaches.
let target: [Float] = [-0.7432918908524302, 0.1312405523087976]
/// The zoom goes in until the view is 10^-depth tall, then back out.
let depth = 7.0
/// Zoom speed, in doublings per second.
let speed = 2.0
/// Height of the first view.
let start = 3.0
// #endregion: constants

struct State {
    let shader: Shader
    var zoom = 0.0
    var direction = 1.0
    var last: Float?
    var pointer: [Float]?
    var moved = -Float.infinity
}

// #region: setup
func setup() throws -> State {
    let shader = try Shader(["gradient/palette_iq", label, shaderSource])
    try shader.set("view.center", target)
    return State(shader: shader)
}
// #endregion: setup

// #region: frame
func frame(
    _ state: inout State, _ renderer: Renderer, _ target: WindowTarget, _ time: Float, _ size: [Float], _ pointer: [Float]
) throws {
    let zoom = steer(&state, time, size, pointer)
    let height = start * exp2(-zoom)
    try state.shader.set("view.resolution", size)
    try state.shader.set("view.height", Float(height))
    try state.shader.set("view.depth", Float(-log10(height)))
    try renderer.render(state.shader, target)
}
// #endregion: frame

// #region: steer
/// The zoom runs from the whole set down to 10^-depth and back on its own.
/// While the pointer moves, and for two seconds after, its height sets the speed:
/// the view dives above the middle and climbs below it. Returns the zoom in doublings.
func steer(_ state: inout State, _ time: Float, _ size: [Float], _ pointer: [Float]) -> Double {
    // A clock that starts over forgets the pointer.
    if let last = state.last, time < last {
        state.moved = -.infinity
    }
    // The first frame counts the whole clock, so a frame drawn at time t shows the dive at t.
    let delta = state.last.map { Double(min(max(time - $0, 0), 0.1)) } ?? Double(time)
    state.last = time
    let deepest = depth * log2(10) + log2(start)
    if let last = state.pointer, last != pointer {
        state.moved = time
    }
    state.pointer = pointer
    if time - state.moved < 2 {
        // The speed grows toward the edges, up to four times `speed`, and a band across the middle holds still.
        let lean = 1 - 2 * Double(pointer[1] / size[1])
        let push = copysign(max(abs(lean) - 0.1, 0) / 0.9, lean)
        state.zoom += push * 4 * speed * delta
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
// #endregion: steer
