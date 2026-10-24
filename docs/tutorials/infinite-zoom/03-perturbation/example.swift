// Infinite zoom, step 3: perturbation.
//
// One reference orbit, computed once with BigInt at the precision the deepest
// view needs, sits in a storage buffer. Each pixel iterates only its small f32
// difference from that orbit, and rebases onto the start of the orbit when it
// drifts closer to 0 than to the reference. The zoom now passes 1e-30 and breaks
// down near 1e-35, where a pixel's width falls below the smallest f32.
// Needs the BigInt package (https://github.com/attaswift/BigInt).
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate.

import BigInt
import Foundation
import FragmentColor

let shaderSource = """
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

/// A Misiurewicz point: the orbit of 0 lands on a cycle of `period` after `preperiod` steps.
struct Point {
    let re: String
    let im: String
    let preperiod: Int
    let period: Int
}

// #region: constants
/// A spiral in Seahorse Valley. Newton's method refines these digits to the precision depth needs.
let target = Point(re: "-0.7432918908524302", im: "0.1312405523087976", preperiod: 24, period: 2)
/// The zoom goes in until the view is 10^-depth tall, then back out.
let depth = 45.0
/// Zoom speed, in doublings per second.
let speed = 4.0
/// Height of the first view.
let start = 3.0
/// Room for the reference orbit.
let orbitSize = 3001
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
    let shader = try Shader(["const ORBIT: u32 = \(orbitSize)u;", "gradient/palette_iq", label, shaderSource])
    // Compute one orbit at the precision the deepest view needs and send it to the GPU once.
    let (orbit, length) = reference(target, depth: depth)
    try shader.set("orbit", orbit.withUnsafeBytes { Data($0) })
    try shader.set("view.length", UInt32(length))
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
    try state.shader.set("view.pixel", Float(height / Double(size[1])))
    try state.shader.set("view.depth", Float(-log10(height)))
    try renderer.render(state.shader, target)
}
// #endregion: frame

// #region: reference
/// The orbit of 0 under `z^2 + c`, computed with `BigInt` at the precision the
/// deepest view needs and rounded to `Float` for the GPU. Rounding errors grow
/// like the derivative dz/dc, so the loop ends before they reach 2^-40.
func reference(_ target: Point, depth: Double) -> ([Float], Int) {
    let bits = Int((depth * log2(10)).rounded(.up)) + 80
    let (cr, ci) = refine(target, bits: bits)
    var orbit = [Float](repeating: 0, count: 2 * orbitSize)
    var (zr, zi) = (BigInt(0), BigInt(0))
    var (dr, di) = (0.0, 0.0)
    for n in 0..<orbitSize {
        let (re, im) = (toDouble(zr, bits: bits), toDouble(zi, bits: bits))
        orbit[2 * n] = Float(re)
        orbit[2 * n + 1] = Float(im)
        if re * re + im * im > 4 || log2(hypot(dr, di)) > Double(bits - 40) {
            return (orbit, n)
        }
        (dr, di) = (2 * (re * dr - im * di) + 1, 2 * (re * di + im * dr))
        (zr, zi) = (((zr * zr - zi * zi) >> bits) + cr, ((2 * zr * zi) >> bits) + ci)
    }
    return (orbit, orbitSize - 1)
}

/// A `BigInt` `n` stands for the real number `n / 2^bits`.
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

/// Newton's method on z(preperiod + period) - z(preperiod) moves `c` until the orbit
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
// #endregion: reference

// #region: steer
/// The zoom runs from the whole set down to 10^-depth and back on its own. While
/// the pointer moves, and for two seconds after, the pointer's vertical position
/// picks the depth, deepest at the top. Returns the zoom in doublings.
func steer(_ state: inout State, _ time: Float, _ size: [Float], _ pointer: [Float]) -> Double {
    // A clock that starts over forgets the pointer.
    if let last = state.last, time < last {
        state.moved = -.infinity
    }
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
// #endregion: steer
