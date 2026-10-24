// Infinite zoom, step 4: past every exponent.
//
// A compute pass iterates every pixel as a small f32 difference from one
// reference orbit, which the host computes once with BigInt. The shader keeps
// each difference as w * 2^s, which never leaves the range of an f32, and
// Martin's series skips the iterations every pixel shares. A render pass colors
// the counts. The zoom reaches a view 1e-1000 tall and comes back out, and the
// pointer's vertical position picks the depth while it moves.
// Needs the BigInt package (https://github.com/attaswift/BigInt).
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate.

import BigInt
import Foundation
import FragmentColor

let iterateSource = """
// #region: shader
// Iterates every pixel as a small difference from the reference orbit, which
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
// #endregion: shader
"""

let presentSource = """
// #region: present-shader
// Colors the counts that the iterate shader writes, and prints the depth with
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
// #endregion: present-shader
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
let depth = 1000.0
/// Zoom speed, in doublings per second.
let speed = 1.5
/// Height of the first view.
let start = 3.0
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger screen renders at a lower resolution.
let orbitSize = 1 << 16
let pixels: UInt32 = 1 << 21
// #endregion: constants

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

// #region: setup
func setup() throws -> State {
    let header = "const ORBIT: u32 = \(orbitSize)u;\nconst PIXELS: u32 = \(pixels)u;"

    let iterate = try Shader([header, iterateSource])
    try iterate.set("view.budget", 1000)

    // `gradient/palette_iq` from the shader library supplies the palette that `presentSource` calls.
    let present = try Shader(["gradient/palette_iq", header, label, presentSource])
    try present.set("look.density", 0.02)
    try present.set("look.phase", 0.6)
    try present.set("look.inside_color", [0.02, 0.02, 0.05])

    // Compute the reference orbit once and send it to the GPU.
    let reference = Reference(target, depth: depth)
    try iterate.set("orbit", reference.orbit.withUnsafeBytes { Data($0) })

    let compute = Pass.fromShader("iterate", iterate)
    let draw = Pass.fromShader("present", present)
    return State(iterate: iterate, present: present, compute: compute, passes: [compute, draw], reference: reference)
}
// #endregion: setup

// #region: frame
func frame(
    _ state: inout State, _ renderer: Renderer, _ target: WindowTarget, _ time: Float, _ size: [Float], _ pointer: [Float]
) throws {
    let zoom = steer(&state, time, size, pointer)

    // The compute pass fills at most `pixels` cells: the screen, or the screen with each side divided by a whole number.
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
    let coefficients = state.reference.series[skip].map(\.exact)
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
    // 8 matches @workgroup_size in iterateSource.
    state.compute.setComputeDispatch((image[0] + 7) / 8, (image[1] + 7) / 8, 1)

    try state.present.set("look.resolution", size)
    try state.present.set("look.image", image)
    try state.present.set("look.depth", Float(zoom * log10(2) - log10(start)))
    try renderer.render(state.passes, target)
}
// #endregion: frame

// #region: reference
// #region: series
/// The orbit of 0 under z^2 + c at full precision, rounded to Float for the GPU,
/// with Martin's series coefficients A, B and C at every iteration. `reach` holds
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
// #endregion: series

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
