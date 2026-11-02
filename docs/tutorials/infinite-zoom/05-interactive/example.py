"""Infinite Mandelbrot zoom, step 5: zoom and pan.

Step 4's deep zoom, which the reader can now explore. The wheel zooms around the
pointer and a drag pans. The view's center is a Python int that gains digits as
the zoom deepens. When the view drifts away from the reference orbit, or zooms
past its digits, a new reference grows a few milliseconds per frame where the
view is heading, and takes over once it covers the view.

From the repository root, run `python docs/tutorials/infinite-zoom/host.py 05-interactive`.
"""

import math
from array import array
from time import perf_counter
from types import SimpleNamespace

from fragmentcolor import Pass, Shader

ITERATE = """
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

PRESENT = """
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

# Prints the view's depth in a corner of the window.
LABEL = """
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
}"""

# #region: constants
# A Misiurewicz point in Seahorse Valley: the orbit of 0 lands on a cycle of
# period 2 after 24 steps. Newton's method refines these digits to the precision DEPTH needs.
TARGET = SimpleNamespace(re="-0.7432918908524302", im="0.1312405523087976", preperiod=24, period=2)
# Until the first scroll or drag, the zoom goes in until the view is 10^-DEPTH tall,
# then back out.
DEPTH = 1000
# Zoom speed, in doublings per second.
SPEED = 1.5
# Height of the first view.
START = 3
# Room for the reference orbit, and for the pixels the compute pass iterates.
# A larger window renders at a lower resolution.
ORBIT = 1 << 16
PIXELS = 1 << 21
# Milliseconds per frame spent growing reference orbits, so a new one never stalls the view.
WORK = 6
# #endregion: constants


# #region: setup
def setup():
    header = f"const ORBIT: u32 = {ORBIT}u;\nconst PIXELS: u32 = {PIXELS}u;"

    iterate = Shader([header, ITERATE])
    iterate.set("view.budget", 183)

    # `gradient/palette_iq` from the shader library supplies the palette that `PRESENT` calls.
    present = Shader(["gradient/palette_iq", header, LABEL, PRESENT])
    present.set("look.density", 0.023)
    present.set("look.phase", 0.332)
    present.set("look.inside_color", [0.02, 0.02, 0.05])

    # The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
    bits = math.ceil(DEPTH * math.log2(10)) + 80
    reference = start_reference(refine(TARGET, bits), bits)

    compute = Pass.from_shader("iterate", iterate)
    draw = Pass.from_shader("present", present)
    return SimpleNamespace(
        iterate=iterate,
        present=present,
        compute=compute,
        passes=[compute, draw],
        reference=reference,
        next=None,
        # The view's center, at `bits` of precision, starts on the reference.
        center=[reference.cr, reference.ci],
        bits=bits,
        zoom=0.0,
        direction=1.0,
        last=None,
        exploring=False,
        sent=None,
        drawn=None,
    )
# #endregion: setup


# #region: frame
def frame(state, renderer, target, time, size, pointer, gesture=None):
    # The zoom in doublings, positive in, and the pan in pixels, since the last frame.
    gesture = gesture or {"zoom": 0.0, "pan": [0.0, 0.0]}
    # The first frame counts the whole clock, so a frame drawn at time t shows the dive at t.
    delta = time if state.last is None else min(max(time - state.last, 0.0), 0.1)
    state.last = time

    # The compute pass fills at most PIXELS: the window, or the window with each side divided by a whole number.
    width, height = max(int(size[0]), 1), max(int(size[1]), 1)
    shrink = 1
    while (width // shrink) * (height // shrink) > PIXELS:
        shrink += 1
    image = [max(width // shrink, 1), max(height // shrink, 1)]
    budget = state.iterate.get("view.budget")

    # The first scroll or drag ends the dive. From then on the view is the reader's.
    if gesture["zoom"] or gesture["pan"][0] or gesture["pan"][1]:
        state.exploring = True
    deadline = perf_counter() + WORK / 1000
    if state.exploring:
        explore(state, size, image, pointer, gesture, budget, deadline)
    else:
        dive(state, image, delta, budget, deadline)

    # A pixel is 2^pixel wide, and the view's center sits away from the reference by `center` pixels.
    reference = state.reference
    pixel = math.log2(START / image[1]) - state.zoom
    exponent = math.floor(pixel)
    center = offset(state, reference, pixel)
    # The series covers every pixel within 2^radius of the reference.
    radius = pixel + math.log2(math.hypot(image[0], image[1]) / 2 + math.hypot(center[0], center[1]))
    skip = skip_for(reference, radius)
    a, b, c = (exact(value) for value in reference.series[skip])

    # The GPU gets the orbit again whenever it grew or another reference took over.
    if state.sent != (id(reference), reference.length):
        state.iterate.set("orbit", reference.orbit.tobytes())
        state.sent = (id(reference), reference.length)
    state.iterate.set("view.image", image)
    state.iterate.set("view.exponent", exponent)
    state.iterate.set("view.fraction", 2 ** (pixel - exponent))
    state.iterate.set("view.center", center)
    state.iterate.set("view.skip", skip)
    state.iterate.set("view.length", reference.length)
    state.iterate.set("view.a", [a[0], a[1]])
    state.iterate.set("view.a_exponent", a[2])
    state.iterate.set("view.b", [b[0], b[1]])
    state.iterate.set("view.b_exponent", b[2])
    state.iterate.set("view.c", [c[0], c[1]])
    state.iterate.set("view.c_exponent", c[2])
    # 8 matches @workgroup_size in ITERATE.
    state.compute.set_compute_dispatch(math.ceil(image[0] / 8), math.ceil(image[1] / 8), 1)

    state.present.set("look.resolution", size)
    state.present.set("look.image", image)
    state.present.set("look.depth", state.zoom * math.log10(2) - math.log10(START))

    # When nothing moved, the counts from the last frame still hold, so only the colors redraw.
    view = (*image, pixel, *center, skip, reference.length, budget)
    renderer.render(state.passes[1:] if view == state.drawn else state.passes, target)
    state.drawn = view
# #endregion: frame


# #region: reference
# #region: series
# #region: grow
# The orbit of 0 under z^2 + c at `bits` of precision, rounded to f32 for the GPU,
# with Martin's series coefficients A, B and C at every iteration. The reach list
# holds the log2 of the widest radius the series still covers at each iteration.
# grow adds to it until a deadline; length is the last index computed so far.
def start_reference(c, bits):
    cr, ci = c
    return SimpleNamespace(
        cr=cr,
        ci=ci,
        bits=bits,
        orbit=array("f", [0.0]) * (2 * ORBIT),
        series=[],
        reach=[math.inf] * ORBIT,
        length=-1,
        done=False,
        z=(0, 0),
        coefficients=(ZERO, ZERO, ZERO, ZERO),
        saved=None,
        checkpoint=1,
    )


def grow(reference, deadline):
    bits = reference.bits
    zr, zi = reference.z
    a, b, c, d = reference.coefficients
    while not reference.done and (reference.length < 0 or perf_counter() < deadline):
        for _ in range(64):
            n = reference.length + 1
            re, im = to_float(zr, bits), to_float(zi, bits)
            reference.orbit[2 * n], reference.orbit[2 * n + 1] = re, im
            reference.series.append((a, b, c))
            # The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
            if n > 0:
                reference.reach[n] = min(reference.reach[n - 1], (math.log2(TOLERANCE) + log2_size(a) - log2_size(d)) / 3)
            reference.length = n
            # Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
            # An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
            # and nothing after adds anything. Brent's method compares it with powers of two.
            saved = reference.saved
            settled = (
                saved is not None
                and math.hypot(re - saved[0], im - saved[1]) < 1e-12
                and abs(log2_size(a) - saved[2]) < 1e-9
            )
            if n == ORBIT - 1 or re * re + im * im > 4 or log2_size(a) > bits - 40 or settled:
                reference.done = True
                break
            if n == reference.checkpoint:
                reference.saved = (re, im, log2_size(a))
                reference.checkpoint *= 2
            # Each new coefficient reads the old values of the ones before it, so D updates first and A last.
            twice = (2 * re, 2 * im)
            d = add(times(d, *twice), add(times(multiply(a, c), 2, 0), multiply(b, b)))
            c = add(times(c, *twice), times(multiply(a, b), 2, 0))
            b = add(times(b, *twice), multiply(a, a))
            a = add(times(a, *twice), ONE)
            zr, zi = ((zr * zr - zi * zi) >> bits) + reference.cr, ((2 * zr * zi) >> bits) + reference.ci
    reference.z = (zr, zi)
    reference.coefficients = (a, b, c, d)
# #endregion: grow


# The largest the first dropped term of the series may grow, relative to the first term.
TOLERANCE = 1e-7


# The last iteration the series covers for a view of this radius (log2).
def skip_for(reference, radius):
    # The last step reads one entry past skip, so skip stops one short of the end.
    low, high = 0, max(reference.length - 1, 0)
    while low < high:
        middle = (low + high + 1) // 2
        if reference.reach[middle] > radius:
            low = middle
        else:
            high = middle - 1
    return low

# #endregion: series


# Complex numbers beyond the range of a float: (re, im, exponent) = (re + i im) * 2^exponent.
ZERO = (0.0, 0.0, 0)
ONE = (1.0, 0.0, 0)


def normalize(re, im, exponent):
    size = max(abs(re), abs(im))
    if size == 0:
        return ZERO
    if 2.0**-64 < size < 2.0**64:
        return (re, im, exponent)
    k = round(math.log2(size))
    return (re * 2.0**-k, im * 2.0**-k, exponent + k)


# The same number with re and im below 1, as the shader expects.
def exact(value):
    re, im, exponent = value
    size = max(abs(re), abs(im))
    if size == 0:
        return ZERO
    k = math.floor(math.log2(size)) + 1
    return (re * 2.0**-k, im * 2.0**-k, exponent + k)


def add(a, b):
    exponent = max(a[2], b[2])
    sa, sb = 2.0 ** (a[2] - exponent), 2.0 ** (b[2] - exponent)
    return normalize(a[0] * sa + b[0] * sb, a[1] * sa + b[1] * sb, exponent)


def multiply(a, b):
    return normalize(a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0], a[2] + b[2])


# a * (re + i im), for a plain complex number.
def times(a, re, im):
    return normalize(a[0] * re - a[1] * im, a[0] * im + a[1] * re, a[2])


def log2_size(a):
    size = math.hypot(a[0], a[1])
    return math.log2(size) + a[2] if size else -math.inf


# An int n stands for the real number n / 2^bits.
def parse(text, bits):
    whole, _, fraction = text.lstrip("-").partition(".")
    value = (int(whole + fraction) << bits) // 10 ** len(fraction)
    return -value if text.startswith("-") else value


def to_float(value, bits):
    return (value >> (bits - 60)) / 2**60


# Newton's method on z(preperiod + period) - z(preperiod) moves c until the orbit
# of 0 enters its cycle, to the working precision. Each round doubles the correct digits.
def refine(target, bits):
    one = 1 << bits
    re, im = parse(target.re, bits), parse(target.im, bits)
    for _ in range(64):
        zr = zi = dr = di = 0
        kr = ki = kdr = kdi = 0
        for n in range(target.preperiod + target.period):
            if n == target.preperiod:
                kr, ki, kdr, kdi = zr, zi, dr, di
            # dz/dc becomes 2 z dz/dc + 1, then z becomes z^2 + c.
            dr, di = ((2 * (zr * dr - zi * di)) >> bits) + one, (2 * (zr * di + zi * dr)) >> bits
            zr, zi = ((zr * zr - zi * zi) >> bits) + re, ((2 * zr * zi) >> bits) + im
        fr, fi, gr, gi = zr - kr, zi - ki, dr - kdr, di - kdi
        size = (gr * gr + gi * gi) >> bits
        if size == 0:
            break
        sr = (((fr * gr + fi * gi) >> bits) << bits) // size
        si = (((fi * gr - fr * gi) >> bits) << bits) // size
        re, im = re - sr, im - si
        if abs(sr) + abs(si) < 256:
            break
    return re, im
# #endregion: reference


# #region: dive
# The zoom runs from the whole set down to 10^-DEPTH and back on its own, never
# deeper than the reference orbit computed so far covers.
def dive(state, image, delta, budget, deadline):
    grow(state.reference, deadline)
    deepest = DEPTH * math.log2(10) + math.log2(START)
    zoom = min(max(state.zoom + state.direction * SPEED * delta, 0.0), deepest)
    if covers(state.reference, image, zoom, [0.0, 0.0], budget):
        state.zoom = zoom
    if state.zoom >= deepest:
        state.direction = -1.0
    if state.zoom <= 0:
        state.direction = 1.0


# Whether this reference serves a view at this zoom: its orbit is complete, or long
# enough for the series skip plus every pixel's budget.
def covers(reference, image, zoom, center, budget):
    if reference.done:
        return True
    pixel = math.log2(START / image[1]) - zoom
    radius = pixel + math.log2(math.hypot(image[0], image[1]) / 2 + math.hypot(center[0], center[1]))
    return skip_for(reference, radius) + budget + 200 <= reference.length
# #endregion: dive


# #region: explore
# The reader's view: the wheel zooms around the pointer, and a drag pans. When the
# view drifts away from the reference, or zooms past its digits, a new reference
# starts where the view is heading and takes over once it covers the view.
def explore(state, size, image, pointer, gesture, budget, deadline):
    pixel = math.log2(START / image[1]) - state.zoom
    # The pointer and the drag, in image pixels with y pointing up.
    scale = image[1] / size[1]
    at = [(pointer[0] - size[0] / 2) * scale, (size[1] / 2 - pointer[1]) * scale]
    # #region: pan
    # Dragging right moves the picture right, so the center moves left.
    state.center = point_at(state, -gesture["pan"][0] * scale, gesture["pan"][1] * scale, pixel)
    # #endregion: pan

    # #region: zoom
    deepest = DEPTH * math.log2(10) + math.log2(START)
    zoom = min(max(state.zoom + gesture["zoom"], 0.0), deepest)
    after = math.log2(START / image[1]) - zoom
    # The center needs a few more digits than the deepest pixel it has seen.
    if state.bits < math.ceil(-after) + 64:
        state.center = [value << 128 for value in state.center]
        state.bits += 128
    # Zooming around the pointer keeps the point under it in place: the center moves
    # toward it by `stay` of the way, and the reference's offset moves with it.
    stay = 1 - 2 ** (state.zoom - zoom)
    reference = state.reference
    shift = 2 ** (pixel - after) * stay
    x, y = offset(state, reference, after)
    drift = [x + at[0] * shift, y + at[1] * shift]
    # Zooming in waits while the reference lacks the digits, is more than two views
    # from where the zoom lands, or hasn't grown far enough yet.
    near = math.hypot(*drift) <= 2 * image[1]
    deep = reference.bits >= math.ceil(-after) + 40
    far = zoom > state.zoom and not (near and deep)
    if zoom < state.zoom or (not far and covers(reference, image, zoom, drift, budget)):
        state.center = point_at(state, at[0] * stay, at[1] * stay, pixel)
        state.zoom = zoom
    # #endregion: zoom

    # #region: follow
    # A zoom in heads for the point under the pointer; anything else stays at the center.
    now = math.log2(START / image[1]) - state.zoom
    aim = at if gesture["zoom"] > 0 else [0.0, 0.0]

    # The offset is the center minus the reference, so a reference at the aim sits at -aim.
    def away(other, limit):
        x, y = offset(state, other, now)
        return math.hypot(x + aim[0], y + aim[1]) > limit

    # A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
    finest = after if far else now
    if far or away(reference, image[1] / 4) or reference.bits < math.ceil(-now) + 120:
        if state.next is None or away(state.next, image[1] / 2) or state.next.bits < math.ceil(-finest) + 40:
            bits = math.ceil(-finest) + 200
            state.next = start_reference(align_to(point_at(state, aim[0], aim[1], now), state.bits, bits), bits)
    if state.next is not None:
        grow(state.next, deadline)
        if covers(state.next, image, state.zoom, offset(state, state.next, now), budget):
            state.reference, state.next = state.next, None
    grow(state.reference, deadline)
    # #endregion: follow
# #endregion: explore


# #region: view
# The point (x, y) pixels from the view's center, as fixed point numbers at state.bits.
def point_at(state, x, y, pixel):
    scale = pixel + state.bits
    lift = max(0, math.floor(scale) - 40)

    def step(value):
        return round(value * 2 ** (scale - lift)) << lift

    return [state.center[0] + step(x), state.center[1] + step(y)]


# The view's center minus the reference's c, in pixels.
def offset(state, reference, pixel):
    bits = max(state.bits, reference.bits)
    cr, ci = align_to(state.center, state.bits, bits)
    rr, ri = align_to([reference.cr, reference.ci], reference.bits, bits)
    return [scaled(cr - rr, -bits - pixel), scaled(ci - ri, -bits - pixel)]


# value * 2^power as a float, for an int of any size.
def scaled(value, power):
    drop = max(0, abs(value).bit_length() - 60)
    return (value >> drop) * 2.0 ** (power + drop)


# The same fixed point numbers at another precision.
def align_to(values, old, new):
    return [value << (new - old) if new >= old else value >> (old - new) for value in values]
# #endregion: view
