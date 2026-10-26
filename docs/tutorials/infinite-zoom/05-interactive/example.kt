// Infinite Mandelbrot zoom, step 5: zoom and pan.
//
// Step 4's deep zoom, which the reader can now explore. A pinch zooms around the
// point between the fingers and a drag pans. The view's center is a BigInteger that
// gains digits as the zoom deepens. When the view drifts away from the reference
// orbit, or zooms past its digits, a new reference grows a few milliseconds per
// frame where the view is heading, and takes over once it covers the view.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback,
// with a `Gesture` that holds the pinch and the drag since the last frame, such as from a `ScaleGestureDetector`.

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
/** Until the first pinch or drag, the zoom goes in until the view is 10^-DEPTH tall, then back out. */
const val DEPTH = 1000.0
/** Zoom speed, in doublings per second. */
const val SPEED = 1.5
/** Height of the first view. */
const val START = 3.0
/** Room for the reference orbit, and for the pixels the compute pass iterates. A larger screen renders at a lower resolution. */
const val ORBIT = 1 shl 16
val PIXELS = 1u shl 21
/** Milliseconds per frame spent growing reference orbits, so a new one never stalls the view. */
const val WORK = 6L
// #endregion: constants

/**
 * What a pinch and a drag did since the last frame: zoom in doublings
 * (positive zooms in) and pan as [dx, dy] in view pixels, with y pointing down.
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
    var drawn: List<Any>? = null
}

// #region: setup
fun setup(): State {
    val header = "const ORBIT: u32 = ${ORBIT}u;\nconst PIXELS: u32 = ${PIXELS}u;"

    val iterate = Shader(listOf(header, ITERATE))
    iterate.set("view.budget", 183u)

    // `gradient/palette_iq` from the shader library supplies the palette that `PRESENT` calls.
    val present = Shader(listOf("gradient/palette_iq", header, LABEL, PRESENT))
    present.set("look.density", 0.023f)
    present.set("look.phase", 0.332f)
    present.set("look.inside_color", listOf(0.02f, 0.02f, 0.05f))

    // The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
    val bits = ceil(DEPTH * log2(10.0)).toInt() + 80
    val reference = Reference(refine(TARGET, bits).toList(), bits)

    val compute = Pass.fromShader("iterate", iterate)
    val draw = Pass.fromShader("present", present)
    // The view's center, at `bits` of precision, starts on the reference.
    return State(iterate, present, compute, listOf(compute, draw), reference, center = reference.c, bits = bits)
}
// #endregion: setup

// #region: frame
fun frame(
    state: State,
    renderer: Renderer,
    target: WindowTarget,
    time: Float,
    size: List<Float>,
    pointer: List<Float>,
    gesture: Gesture = Gesture(),
) {
    // The first frame counts the whole clock, so a frame drawn at time t shows the dive at t.
    val delta = state.last?.let { min(max(time - it, 0f), 0.1f).toDouble() } ?: time.toDouble()
    state.last = time

    // The compute pass fills at most PIXELS: the screen, or the screen with each side divided by a whole number.
    val width = maxOf(size[0].toUInt(), 1u)
    val height = maxOf(size[1].toUInt(), 1u)
    var shrink = 1u
    while ((width / shrink) * (height / shrink) > PIXELS) shrink++
    val image = listOf(maxOf(width / shrink, 1u), maxOf(height / shrink, 1u))
    val budget = (state.iterate.get("view.budget").float ?: 0f).toInt()

    // The first pinch or drag ends the dive. From then on the view is the reader's.
    if (gesture.zoom != 0.0 || gesture.pan[0] != 0.0 || gesture.pan[1] != 0.0) state.exploring = true
    val deadline = System.nanoTime() + WORK * 1_000_000
    if (state.exploring) {
        explore(state, size, image, pointer, gesture, budget, deadline)
    } else {
        dive(state, image, delta, budget, deadline)
    }

    // A pixel is 2^pixel wide, and the view's center sits away from the reference by `center` pixels.
    val reference = state.reference
    val pixel = log2(START / image[1].toDouble()) - state.zoom
    val exponent = floor(pixel)
    val center = offset(state, reference, pixel)
    // The series covers every pixel within 2^radius of the reference.
    val radius = pixel + log2(hypot(image[0].toDouble(), image[1].toDouble()) / 2 + hypot(center[0], center[1]))
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
    // 8 matches @workgroup_size in ITERATE.
    state.compute.setComputeDispatch((image[0] + 7u) / 8u, (image[1] + 7u) / 8u, 1u)

    state.present.set("look.resolution", size)
    state.present.set("look.image", image)
    state.present.set("look.depth", (state.zoom * log10(2.0) - log10(START)).toFloat())

    // When nothing moved, the counts from the last frame still hold, so only the colors redraw.
    val view = listOf(image, pixel, center, skip, reference.length, budget)
    renderer.render(if (view == state.drawn) state.passes.drop(1) else state.passes, target)
    state.drawn = view
}
// #endregion: frame

// #region: reference
// #region: series
// #region: grow
/**
 * The orbit of 0 under z^2 + c at [bits] of precision, rounded to Float for the GPU,
 * with Martin's series coefficients A, B and C at every iteration. [reach] holds
 * the log2 of the widest radius the series still covers at each iteration.
 * [grow] adds to it until a deadline; [length] is the last index computed so far.
 */
class Reference(val c: List<BigInteger>, val bits: Int) {
    val orbit = FloatArray(2 * ORBIT)
    val series = mutableListOf<List<Wide>>()
    val reach = DoubleArray(ORBIT) { Double.POSITIVE_INFINITY }
    var length = -1
        private set
    var done = false
        private set
    private var z = listOf(BigInteger.ZERO, BigInteger.ZERO)
    private var coefficients = listOf(Wide.ZERO, Wide.ZERO, Wide.ZERO, Wide.ZERO)
    private var saved: Triple<Double, Double, Double>? = null
    private var checkpoint = 1

    fun grow(deadline: Long) {
        while (!done && (length < 0 || System.nanoTime() < deadline)) {
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
        if (n > 0) reach[n] = min(reach[n - 1], (log2(TOLERANCE) + a.log2Size() - d.log2Size()) / 3)
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
}
// #endregion: grow

/** The largest the first dropped term of the series may grow, relative to the first term. */
private const val TOLERANCE = 1e-7

/** The last iteration the series covers for a view of this radius (log2). */
fun Reference.skip(radius: Double): Int {
    var low = 0
    // The last step reads one entry past skip, so skip stops one short of the end.
    var high = maxOf(length - 1, 0)
    while (low < high) {
        val middle = (low + high + 1) / 2
        if (reach[middle] > radius) low = middle else high = middle - 1
    }
    return low
}
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

// #region: dive
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
 * Whether this reference serves a view at this zoom: its orbit is complete, or long
 * enough for the series skip plus every pixel's budget.
 */
fun covers(reference: Reference, image: List<UInt>, zoom: Double, center: List<Double>, budget: Int): Boolean {
    if (reference.done) return true
    val pixel = log2(START / image[1].toDouble()) - zoom
    val radius = pixel + log2(hypot(image[0].toDouble(), image[1].toDouble()) / 2 + hypot(center[0], center[1]))
    return reference.skip(radius) + budget + 200 <= reference.length
}
// #endregion: dive

// #region: explore
/**
 * The reader's view: a pinch zooms around the pointer, and a drag pans. When the
 * view drifts away from the reference, or zooms past its digits, a new reference
 * starts where the view is heading and takes over once it covers the view.
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
    // #region: pan
    // Dragging right moves the picture right, so the center moves left.
    state.center = pointAt(state, -gesture.pan[0] * scale, gesture.pan[1] * scale, pixel)
    // #endregion: pan

    // #region: zoom
    val deepest = DEPTH * log2(10.0) + log2(START)
    val zoom = (state.zoom + gesture.zoom).coerceIn(0.0, deepest)
    val after = log2(START / image[1].toDouble()) - zoom
    // The center needs a few more digits than the deepest pixel it has seen.
    if (state.bits < ceil(-after).toInt() + 64) {
        state.center = state.center.map { it.shiftLeft(128) }
        state.bits += 128
    }
    // Zooming around the pointer keeps the point under it in place: the center moves
    // toward it by `stay` of the way, and the reference's offset moves with it.
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
        state.center = pointAt(state, at[0] * stay, at[1] * stay, pixel)
        state.zoom = zoom
    }
    // #endregion: zoom

    // #region: follow
    // A zoom in heads for the point under the pointer; anything else stays at the center.
    val now = log2(START / image[1].toDouble()) - state.zoom
    val aim = if (gesture.zoom > 0) at else listOf(0.0, 0.0)
    // The offset is the center minus the reference, so a reference at the aim sits at -aim.
    fun away(other: Reference, limit: Double): Boolean {
        val (x, y) = offset(state, other, now)
        return hypot(x + aim[0], y + aim[1]) > limit
    }
    // A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
    val finest = if (far) after else now
    if (far || away(reference, image[1].toDouble() / 4) || reference.bits < ceil(-now).toInt() + 120) {
        val next = state.next
        if (next == null || away(next, image[1].toDouble() / 2) || next.bits < ceil(-finest).toInt() + 40) {
            val bits = ceil(-finest).toInt() + 200
            state.next = Reference(align(pointAt(state, aim[0], aim[1], now), state.bits, bits), bits)
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
    // #endregion: follow
}
// #endregion: explore

// #region: view
/** The point (x, y) pixels from the view's center, as fixed point numbers at state.bits. */
fun pointAt(state: State, x: Double, y: Double, pixel: Double): List<BigInteger> {
    val scale = pixel + state.bits
    val lift = max(0, floor(scale).toInt() - 40)
    fun step(value: Double) = BigInteger.valueOf((value * 2.0.pow(scale - lift)).roundToLong()).shiftLeft(lift)
    return listOf(state.center[0] + step(x), state.center[1] + step(y))
}

/** The view's center minus the reference's c, in pixels. */
fun offset(state: State, reference: Reference, pixel: Double): List<Double> {
    val bits = max(state.bits, reference.bits)
    val (cr, ci) = align(state.center, state.bits, bits)
    val (rr, ri) = align(reference.c, reference.bits, bits)
    return listOf(scaled(cr - rr, -bits - pixel), scaled(ci - ri, -bits - pixel))
}

/** value * 2^power as a Double, for a BigInteger of any size. */
fun scaled(value: BigInteger, power: Double): Double {
    val drop = max(0, value.bitLength() - 60)
    return value.shiftRight(drop).toDouble() * 2.0.pow(power + drop)
}

/** The same fixed point numbers at another precision. */
fun align(values: List<BigInteger>, from: Int, to: Int): List<BigInteger> =
    values.map { if (to >= from) it.shiftLeft(to - from) else it.shiftRight(from - to) }
// #endregion: view
