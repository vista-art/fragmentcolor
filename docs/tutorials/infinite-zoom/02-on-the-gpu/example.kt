// Infinite Mandelbrot zoom, step 2: the same loop on the GPU.
//
// A fragment shader runs the escape-time loop for every pixel in parallel, in
// f32. The view zooms toward a spiral in Seahorse Valley and back out, and the
// pointer's vertical position picks the depth while it moves. Past 1e-5, f32 rounding
// breaks the picture into blocks.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

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

// #region: constants
/** A spiral in Seahorse Valley, the point the zoom approaches. */
val TARGET = listOf(-0.7432918908524302f, 0.1312405523087976f)
/** The zoom goes in until the view is 10^-DEPTH tall, then back out. */
const val DEPTH = 7.0
/** Zoom speed, in doublings per second. */
const val SPEED = 2.0
/** Height of the first view. */
const val START = 3.0
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
    val shader = Shader(listOf("gradient/palette_iq", LABEL, SHADER))
    shader.set("view.center", TARGET)
    return State(shader)
}
// #endregion: setup

// #region: frame
fun frame(state: State, renderer: Renderer, target: WindowTarget, time: Float, size: List<Float>, pointer: List<Float>) {
    val zoom = steer(state, time, size, pointer)
    val height = START * 2.0.pow(-zoom)
    state.shader.set("view.resolution", size)
    state.shader.set("view.height", height.toFloat())
    state.shader.set("view.depth", (-log10(height)).toFloat())
    renderer.render(state.shader, target)
}
// #endregion: frame

// #region: steer
/**
 * The zoom runs from the whole set down to 10^-DEPTH and back on its own. While
 * the pointer moves, and for two seconds after, the pointer's vertical position
 * picks the depth, deepest at the top. Returns the zoom in doublings.
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
