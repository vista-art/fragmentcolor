// Infinite Mandelbrot zoom, step 4: past every exponent.
//
// A compute pass iterates every pixel as a small f32 difference from one
// reference orbit, which the host computes once with BigInteger. The shader
// keeps each difference as w * 2^s, which never leaves the range of an f32, and
// Martin's series skips the iterations every pixel shares. A render pass colors
// the counts. The zoom reaches a view 1e-1000 tall and comes back out, and the
// pointer's vertical position picks the depth while it moves.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

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
import kotlin.math.withSign
import org.fragmentcolor.*

const val ITERATE = """
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

const val PRESENT = """
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

/** Prints the view's depth in a corner of the screen. */
const val LABEL = """
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

/** A Misiurewicz point: the orbit of 0 lands on a cycle of [period] after [preperiod] steps. */
class Point(val re: String, val im: String, val preperiod: Int, val period: Int)

// #region: constants
/** A spiral in Seahorse Valley. Newton's method refines these digits to the precision DEPTH needs. */
val TARGET = Point("-0.7432918908524302", "0.1312405523087976", preperiod = 24, period = 2)
/** The zoom goes in until the view is 10^-DEPTH tall, then back out. */
const val DEPTH = 1000.0
/** Zoom speed, in doublings per second. */
const val SPEED = 1.5
/** Height of the first view. */
const val START = 3.0
/** Room for the reference orbit, and for the pixels the compute pass iterates. A larger screen renders at a lower resolution. */
const val ORBIT = 1 shl 16
val PIXELS = 1u shl 21
// #endregion: constants

class State(val iterate: Shader, val present: Shader, val compute: Pass, val passes: List<Pass>, val reference: Reference) {
    var zoom = 0.0
    var direction = 1.0
    var last: Float? = null
    var pointer: List<Float>? = null
    var moved = Float.NEGATIVE_INFINITY
}

// #region: setup
fun setup(): State {
    val header = "const ORBIT: u32 = ${ORBIT}u;\nconst PIXELS: u32 = ${PIXELS}u;"

    val iterate = Shader(listOf(header, ITERATE))
    iterate.set("view.budget", 1000u)

    // `gradient/palette_iq` from the shader library supplies the palette that `PRESENT` calls.
    val present = Shader(listOf("gradient/palette_iq", header, LABEL, PRESENT))
    present.set("look.density", 0.02f)
    present.set("look.phase", 0.6f)
    present.set("look.inside_color", listOf(0.02f, 0.02f, 0.05f))

    // Compute the reference orbit once and send it to the GPU.
    val reference = Reference(TARGET, DEPTH)
    val bytes = ByteBuffer.allocate(4 * reference.orbit.size).order(ByteOrder.LITTLE_ENDIAN)
    reference.orbit.forEach { bytes.putFloat(it) }
    iterate.set("orbit", bytes.array())

    val compute = Pass.fromShader("iterate", iterate)
    val draw = Pass.fromShader("present", present)
    return State(iterate, present, compute, listOf(compute, draw), reference)
}
// #endregion: setup

// #region: frame
fun frame(state: State, renderer: Renderer, target: WindowTarget, time: Float, size: List<Float>, pointer: List<Float>) {
    val zoom = steer(state, time, size, pointer)

    // The compute pass fills at most PIXELS: the screen, or the screen with each side divided by a whole number.
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
    // 8 matches @workgroup_size in ITERATE.
    state.compute.setComputeDispatch((image[0] + 7u) / 8u, (image[1] + 7u) / 8u, 1u)

    state.present.set("look.resolution", size)
    state.present.set("look.image", image)
    state.present.set("look.depth", (zoom * log10(2.0) - log10(START)).toFloat())
    renderer.render(state.passes, target)
}
// #endregion: frame

// #region: reference
// #region: series
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
// #endregion: series

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
// #endregion: reference

// #region: steer
/**
 * The zoom runs from the whole set down to 10^-DEPTH and back on its own.
 * While the pointer moves, and for two seconds after, its height sets the speed:
 * the view dives above the middle and climbs below it. Returns the zoom in doublings.
 */
fun steer(state: State, time: Float, size: List<Float>, pointer: List<Float>): Double {
    // A clock that starts over forgets the pointer.
    if (state.last != null && time < state.last!!) state.moved = Float.NEGATIVE_INFINITY
    // The first frame counts the whole clock, so a frame drawn at time t shows the dive at t.
    val delta = state.last?.let { min(max(time - it, 0f), 0.1f).toDouble() } ?: time.toDouble()
    state.last = time
    val deepest = DEPTH * log2(10.0) + log2(START)
    if (state.pointer != null && state.pointer != pointer) state.moved = time
    state.pointer = pointer
    if (time - state.moved < 2f) {
        // The speed grows toward the edges, up to four times SPEED, and a band across the middle holds still.
        val lean = 1 - 2 * (pointer[1] / size[1]).toDouble()
        val push = (max(abs(lean) - 0.1, 0.0) / 0.9).withSign(lean)
        state.zoom += push * 4 * SPEED * delta
    } else {
        state.zoom += state.direction * SPEED * delta
        if (state.zoom >= deepest) state.direction = -1.0
        if (state.zoom <= 0) state.direction = 1.0
    }
    state.zoom = state.zoom.coerceIn(0.0, deepest)
    return state.zoom
}
// #endregion: steer
