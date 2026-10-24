// Infinite zoom, step 3: perturbation.
//
// One reference orbit, computed once with BigInteger at the precision the
// deepest view needs, sits in a storage buffer. Each pixel iterates only its
// small f32 difference from that orbit, and rebases onto the start of the orbit
// when it drifts closer to 0 than to the reference. The zoom now passes 1e-30 and
// breaks down near 1e-35, where a pixel's width falls below the smallest f32.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

import java.math.BigInteger
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.math.ceil
import kotlin.math.hypot
import kotlin.math.log10
import kotlin.math.log2
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import org.fragmentcolor.*

const val SHADER = """
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
const val DEPTH = 45.0
/** Zoom speed, in doublings per second. */
const val SPEED = 4.0
/** Height of the first view. */
const val START = 3.0
/** Room for the reference orbit. */
const val ORBIT = 3001
// #endregion: constants

class State(val shader: Shader) {
    var zoom = 0.0
    var direction = 1.0
    var last: Float? = null
    var pointer: List<Float>? = null
    var moved = Float.NEGATIVE_INFINITY
}

// #region: setup
fun setup(): State {
    val shader = Shader(listOf("const ORBIT: u32 = ${ORBIT}u;", "gradient/palette_iq", LABEL, SHADER))
    // Compute one orbit at the precision the deepest view needs and send it to the GPU once.
    val (orbit, length) = reference(TARGET, DEPTH)
    val bytes = ByteBuffer.allocate(4 * orbit.size).order(ByteOrder.LITTLE_ENDIAN)
    orbit.forEach { bytes.putFloat(it) }
    shader.set("orbit", bytes.array())
    shader.set("view.length", length.toUInt())
    return State(shader)
}
// #endregion: setup

// #region: frame
fun frame(state: State, renderer: Renderer, target: WindowTarget, time: Float, size: List<Float>, pointer: List<Float>) {
    val zoom = steer(state, time, size, pointer)
    val height = START * 2.0.pow(-zoom)
    state.shader.set("view.resolution", size)
    state.shader.set("view.pixel", (height / size[1]).toFloat())
    state.shader.set("view.depth", (-log10(height)).toFloat())
    renderer.render(state.shader, target)
}
// #endregion: frame

// #region: reference
/**
 * The orbit of 0 under `z^2 + c`, computed with [BigInteger] at the precision the
 * deepest view needs and rounded to Float for the GPU. Rounding errors grow like
 * the derivative dz/dc, so the loop ends before they reach 2^-40.
 */
fun reference(target: Point, depth: Double): Pair<FloatArray, Int> {
    val bits = ceil(depth * log2(10.0)).toInt() + 80
    val (cr, ci) = refine(target, bits)
    val orbit = FloatArray(2 * ORBIT)
    var zr = BigInteger.ZERO
    var zi = BigInteger.ZERO
    var dr = 0.0
    var di = 0.0
    for (n in 0 until ORBIT) {
        val re = toDouble(zr, bits)
        val im = toDouble(zi, bits)
        orbit[2 * n] = re.toFloat()
        orbit[2 * n + 1] = im.toFloat()
        if (re * re + im * im > 4 || log2(hypot(dr, di)) > bits - 40) return Pair(orbit, n)
        val nextDr = 2 * (re * dr - im * di) + 1
        di = 2 * (re * di + im * dr)
        dr = nextDr
        val next = zr.multiply(zr).subtract(zi.multiply(zi)).shiftRight(bits).add(cr)
        zi = zr.multiply(zi).shiftLeft(1).shiftRight(bits).add(ci)
        zr = next
    }
    return Pair(orbit, ORBIT - 1)
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
 * The zoom runs from the whole set down to 10^-DEPTH and back on its own. While
 * the pointer moves, and for two seconds after, the pointer's vertical position
 * picks the depth, deepest at the top. Returns the zoom in doublings.
 */
fun steer(state: State, time: Float, size: List<Float>, pointer: List<Float>): Double {
    // A clock that starts over forgets the pointer.
    if (state.last != null && time < state.last!!) state.moved = Float.NEGATIVE_INFINITY
    val delta = state.last?.let { min(max(time - it, 0f), 0.1f).toDouble() } ?: 0.0
    state.last = time
    val deepest = DEPTH * log2(10.0) + log2(START)
    if (state.pointer != null && state.pointer != pointer) state.moved = time
    state.pointer = pointer
    if (time - state.moved < 2f) {
        val goal = (1 - (pointer[1] / size[1]).toDouble()) * deepest
        state.zoom += (goal - state.zoom) * min(1.0, delta * 4)
    } else {
        state.zoom += state.direction * SPEED * delta
        if (state.zoom >= deepest) state.direction = -1.0
        if (state.zoom <= 0) state.direction = 1.0
    }
    state.zoom = state.zoom.coerceIn(0.0, deepest)
    return state.zoom
}
// #endregion: steer
