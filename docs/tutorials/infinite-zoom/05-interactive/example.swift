// Infinite Mandelbrot zoom, step 5: zoom and pan.
//
// Step 4's deep zoom, which the reader can now explore. A pinch zooms around the
// point between the fingers and a drag pans. The view's center is a BigInt that
// gains digits as the zoom deepens. When the view drifts away from the reference
// orbit, or zooms past its digits, a new reference grows a few milliseconds per
// frame where the view is heading, and takes over once it covers the view.
// Needs the BigInt package (https://github.com/attaswift/BigInt).
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate,
// with the pinch and the drag gathered since the last refresh.

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
/// Until the first pinch or drag, the zoom goes in
/// until the view is 10^-depth tall, then back out.
let depth = 1000.0
/// Zoom speed, in doublings per second.
let speed = 1.5
/// Height of the first view.
let start = 3.0
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger screen renders at a lower resolution.
let orbitSize = 1 << 16
let pixels: UInt32 = 1 << 21
/// Seconds per frame spent growing reference orbits, so a new one never stalls the view.
let work: TimeInterval = 0.006
// #endregion: constants

/// What a pinch and a drag did since the last frame.
/// `zoom` counts doublings, and a positive one zooms in.
/// `pan` is [dx, dy] in view pixels, with y pointing down.
/// During a pinch, pass the point between the fingers as `frame`'s pointer.
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
    var sent: (reference: Reference, length: Int)?
    var drawn: [Double]?
}

// #region: setup
func setup() throws -> State {
    let header = "const ORBIT: u32 = \(orbitSize)u;\nconst PIXELS: u32 = \(pixels)u;"

    let iterate = try Shader([header, iterateSource])
    try iterate.set("view.budget", 183)

    // `gradient/palette_iq` from the shader library supplies the palette that `presentSource` calls.
    let present = try Shader(["gradient/palette_iq", header, label, presentSource])
    try present.set("look.density", 0.023)
    try present.set("look.phase", 0.332)
    try present.set("look.inside_color", [0.02, 0.02, 0.05])

    // The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
    let bits = Int((depth * log2(10)).rounded(.up)) + 80
    let (cr, ci) = refine(target, bits: bits)
    let reference = Reference([cr, ci], bits: bits)

    let compute = Pass.fromShader("iterate", iterate)
    let draw = Pass.fromShader("present", present)
    // The view's center, at `bits` of precision, starts on the reference.
    return State(
        iterate: iterate, present: present, compute: compute, passes: [compute, draw], reference: reference,
        center: reference.c, bits: bits
    )
}
// #endregion: setup

// #region: frame
func frame(
    _ state: inout State, _ renderer: Renderer, _ target: WindowTarget, _ time: Float, _ size: [Float],
    _ pointer: [Float], _ gesture: Gesture = Gesture()
) throws {
    // The first frame counts the whole clock, so a frame drawn at time t shows the dive at t.
    let delta = state.last.map { Double(min(max(time - $0, 0), 0.1)) } ?? Double(time)
    state.last = time

    // The compute pass fills at most `pixels` cells: the screen, or the screen with each side divided by a whole number.
    let width = max(UInt32(size[0]), 1)
    let height = max(UInt32(size[1]), 1)
    var shrink: UInt32 = 1
    while (width / shrink) * (height / shrink) > pixels {
        shrink += 1
    }
    let image = [max(width / shrink, 1), max(height / shrink, 1)]
    let budget = Int(try state.iterate.get("view.budget").float ?? 0)

    // The first pinch or drag ends the dive. From then on the view is the reader's.
    if gesture.zoom != 0 || gesture.pan[0] != 0 || gesture.pan[1] != 0 {
        state.exploring = true
    }
    let deadline = Date().addingTimeInterval(work)
    if state.exploring {
        explore(&state, size, image, pointer, gesture, budget, deadline)
    } else {
        dive(&state, image, delta, budget, deadline)
    }

    // A pixel is 2^pixel wide, and the view's center sits away from the reference by `center` pixels.
    let reference = state.reference
    let pixel = log2(start / Double(image[1])) - state.zoom
    let exponent = pixel.rounded(.down)
    let center = offset(state, reference, pixel)
    // The series covers every pixel within 2^radius of the reference.
    let radius = pixel + log2(hypot(Double(image[0]), Double(image[1])) / 2 + hypot(center[0], center[1]))
    let skip = reference.skip(radius)
    let coefficients = reference.series[skip].map(\.exact)
    let (a, b, c) = (coefficients[0], coefficients[1], coefficients[2])

    // The GPU gets the orbit again whenever it grew or another reference took over.
    if state.sent?.reference !== reference || state.sent?.length != reference.length {
        try state.iterate.set("orbit", reference.orbit.withUnsafeBytes { Data($0) })
        state.sent = (reference, reference.length)
    }
    try state.iterate.set("view.image", image)
    try state.iterate.set("view.exponent", Int32(exponent))
    try state.iterate.set("view.fraction", Float(exp2(pixel - exponent)))
    try state.iterate.set("view.center", center)
    try state.iterate.set("view.skip", UInt32(skip))
    try state.iterate.set("view.length", UInt32(reference.length))
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
    try state.present.set("look.depth", Float(state.zoom * log10(2) - log10(start)))

    // When nothing moved, the counts from the last frame still hold, so only the colors redraw.
    let view = [
        Double(image[0]), Double(image[1]), pixel, center[0], center[1],
        Double(skip), Double(reference.length), Double(budget),
    ]
    try renderer.render(view == state.drawn ? Array(state.passes.dropFirst()) : state.passes, target)
    state.drawn = view
}
// #endregion: frame

// #region: reference
// #region: series
// #region: grow
/// The orbit of 0 under z^2 + c at `bits` of precision, rounded to Float
/// for the GPU, with Martin's series coefficients A, B and C at every iteration.
/// `reach` holds the log2 of the widest radius the series still covers at each iteration.
/// `grow` extends the orbit until a deadline, and `length` is the last index computed so far.
final class Reference {
    let c: [BigInt]
    let bits: Int
    var orbit: [Float]
    var series: [[Wide]] = []
    var reach: [Double]
    var length = -1
    var done = false
    var z = [BigInt(0), BigInt(0)]
    var coefficients = [Wide.zero, .zero, .zero, .zero]
    var saved: (re: Double, im: Double, size: Double)?
    var checkpoint = 1

    init(_ c: [BigInt], bits: Int) {
        self.c = c
        self.bits = bits
        orbit = [Float](repeating: 0, count: 2 * orbitSize)
        reach = [Double](repeating: .infinity, count: orbitSize)
    }

    func grow(until deadline: Date) {
        while !done && (length < 0 || Date() < deadline) {
            for _ in 0..<64 where !done {
                step()
            }
        }
    }

    /// Adds the next iteration of the orbit and the series.
    private func step() {
        let n = length + 1
        let (re, im) = (toDouble(z[0], bits: bits), toDouble(z[1], bits: bits))
        orbit[2 * n] = Float(re)
        orbit[2 * n + 1] = Float(im)
        var (a, b, c, d) = (coefficients[0], coefficients[1], coefficients[2], coefficients[3])
        series.append([a, b, c])
        // The series holds while the first dropped term, D * delta^4, stays below tolerance * A * delta.
        if n > 0 {
            reach[n] = min(reach[n - 1], (log2(tolerance) + a.log2Size - d.log2Size) / 3)
        }
        length = n
        // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
        // An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
        // and nothing after adds anything. Brent's method compares it with powers of two.
        let settled = saved.map { hypot(re - $0.re, im - $0.im) < 1e-12 && abs(a.log2Size - $0.size) < 1e-9 } ?? false
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
}
// #endregion: grow

/// The largest the first dropped term of the series may grow, relative to the first term.
let tolerance = 1e-7

extension Reference {
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

// #region: dive
/// The zoom runs from the whole set down to 10^-depth and back
/// on its own, never deeper than the reference orbit computed so far covers.
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

/// Whether this reference serves a view at this zoom:
/// its orbit is complete, or long enough for the series skip plus every pixel's budget.
func covers(_ reference: Reference, _ image: [UInt32], _ zoom: Double, _ center: [Double], _ budget: Int) -> Bool {
    if reference.done {
        return true
    }
    let pixel = log2(start / Double(image[1])) - zoom
    let radius = pixel + log2(hypot(Double(image[0]), Double(image[1])) / 2 + hypot(center[0], center[1]))
    return reference.skip(radius) + budget + 200 <= reference.length
}
// #endregion: dive

// #region: explore
/// The reader's view: a pinch zooms around the pointer, and a drag pans.
/// When the view drifts away from the reference, or zooms past its digits,
/// a new reference starts where the view is heading and takes over once it covers the view.
func explore(
    _ state: inout State, _ size: [Float], _ image: [UInt32], _ pointer: [Float], _ gesture: Gesture,
    _ budget: Int, _ deadline: Date
) {
    let pixel = log2(start / Double(image[1])) - state.zoom
    // The pointer and the drag, in image pixels with y pointing up.
    let scale = Double(image[1]) / Double(size[1])
    let at = [Double(pointer[0] - size[0] / 2) * scale, Double(size[1] / 2 - pointer[1]) * scale]
    // #region: pan
    // Dragging right moves the picture right, so the center moves left.
    state.center = pointAt(state, -gesture.pan[0] * scale, gesture.pan[1] * scale, pixel)
    // #endregion: pan

    // #region: zoom
    let deepest = depth * log2(10) + log2(start)
    let zoom = min(max(state.zoom + gesture.zoom, 0), deepest)
    let after = log2(start / Double(image[1])) - zoom
    // The center needs a few more digits than the deepest pixel it has seen.
    if state.bits < Int((-after).rounded(.up)) + 64 {
        state.center = state.center.map { $0 << 128 }
        state.bits += 128
    }
    // Zooming around the pointer keeps the point under it in place: the center moves
    // toward it by `stay` of the way, and the reference's offset moves with it.
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
        state.center = pointAt(state, at[0] * stay, at[1] * stay, pixel)
        state.zoom = zoom
    }
    // #endregion: zoom

    // #region: follow
    // A zoom in heads for the point under the pointer; anything else stays at the center.
    let now = log2(start / Double(image[1])) - state.zoom
    let aim = gesture.zoom > 0 ? at : [0, 0]
    // The offset is the center minus the reference, so a reference at the aim sits at -aim.
    func away(_ other: Reference, _ limit: Double) -> Bool {
        let point = offset(state, other, now)
        return hypot(point[0] + aim[0], point[1] + aim[1]) > limit
    }
    // A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
    let finest = far ? after : now
    let shallow = Int((-finest).rounded(.up)) + 40
    if far || away(reference, Double(image[1]) / 4) || reference.bits < Int((-now).rounded(.up)) + 120 {
        if state.next.map({ away($0, Double(image[1]) / 2) || $0.bits < shallow }) ?? true {
            let bits = Int((-finest).rounded(.up)) + 200
            state.next = Reference(align(pointAt(state, aim[0], aim[1], now), from: state.bits, to: bits), bits: bits)
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
    // #endregion: follow
}
// #endregion: explore

// #region: view
/// The point (x, y) pixels from the view's center, as fixed point numbers at `state.bits`.
func pointAt(_ state: State, _ x: Double, _ y: Double, _ pixel: Double) -> [BigInt] {
    let scale = pixel + Double(state.bits)
    let lift = max(0, Int(scale.rounded(.down)) - 40)
    func step(_ value: Double) -> BigInt {
        BigInt((value * exp2(scale - Double(lift))).rounded()) << lift
    }
    return [state.center[0] + step(x), state.center[1] + step(y)]
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
// #endregion: view
