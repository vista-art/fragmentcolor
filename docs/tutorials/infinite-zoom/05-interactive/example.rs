//! Infinite Mandelbrot zoom, step 5: zoom and pan.
//!
//! Step 4's deep zoom, which the reader can now explore. A pinch or the wheel
//! zooms around the pointer, and a drag pans. The view's center is a `BigInt` that
//! gains digits as the zoom deepens. When the view drifts away from the reference
//! orbit, or zooms past its digits, a new reference grows a few milliseconds per
//! frame where the view is heading, and takes over once it covers the view.

use std::error::Error;
use std::mem;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use fragmentcolor::{
    App, Pass, RenderTarget, Renderer, RendererError, SetupResult, Shader, Size, SurfaceError,
    Target, call,
};
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive};
use parking_lot::Mutex;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase};
use winit::window::Window;

type Failure = Box<dyn Error + Send + Sync>;

const ITERATE: &str = r#"
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
"#;

const PRESENT: &str = r#"
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
"#;

/// Prints the view's depth in a corner of the window.
const LABEL: &str = r#"
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
}"#;

/// A Misiurewicz point: the orbit of 0 lands on a cycle of `period` after `preperiod` steps.
struct Point {
    re: &'static str,
    im: &'static str,
    preperiod: usize,
    period: usize,
}

// #region: constants
/// A spiral in Seahorse Valley. Newton's method refines these digits to the precision DEPTH needs.
const TARGET: Point = Point {
    re: "-0.7432918908524302",
    im: "0.1312405523087976",
    preperiod: 24,
    period: 2,
};
/// Until the first scroll, pinch or drag,
/// the zoom goes in until the view is 10^-DEPTH tall, then back out.
const DEPTH: f64 = 1000.0;
/// Zoom speed, in doublings per second.
const SPEED: f64 = 1.5;
/// Height of the first view.
const START: f64 = 3.0;
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger window renders at a lower resolution.
const ORBIT: usize = 1 << 16;
const PIXELS: u32 = 1 << 21;
/// Time per frame spent growing reference orbits, so a new one never stalls the view.
const WORK: Duration = Duration::from_millis(6);
// #endregion: constants

struct Zoom {
    iterate: Shader,
    present: Shader,
    compute: Pass,
    passes: Vec<Pass>,
    motion: Mutex<Motion>,
    input: Mutex<Input>,
}

impl Zoom {
    // #region: setup
    fn new() -> Result<Zoom, Failure> {
        let header = format!("const ORBIT: u32 = {ORBIT}u;\nconst PIXELS: u32 = {PIXELS}u;");

        let iterate = Shader::new([header.as_str(), ITERATE])?;
        iterate.set("view.budget", 183_u32)?;

        // `gradient/palette_iq` from the shader library supplies the palette that `PRESENT` calls.
        let present = Shader::new(["gradient/palette_iq", header.as_str(), LABEL, PRESENT])?;
        present.set("look.density", 0.023_f32)?;
        present.set("look.phase", 0.332_f32)?;
        present.set("look.inside_color", [0.02_f32, 0.02, 0.05])?;

        // The dive's reference orbit grows a few milliseconds per frame, and the dive waits for it.
        let bits = (DEPTH * 10_f64.log2()).ceil() as usize + 80;
        let (cr, ci) = refine(&TARGET, bits)?;
        let reference = Reference::start([cr, ci], bits);

        let compute = Pass::from_shader("iterate", &iterate);
        let draw = Pass::from_shader("present", &present);
        Ok(Zoom {
            passes: vec![compute.clone(), draw],
            iterate,
            present,
            compute,
            motion: Mutex::new(Motion {
                // The view's center, at `bits` of precision, starts on the reference.
                center: [reference.cr.clone(), reference.ci.clone()],
                bits,
                reference,
                next: None,
                zoom: 0.0,
                direction: 1.0,
                last: None,
                exploring: false,
                sent: None,
                drawn: None,
            }),
            input: Mutex::new(Input::default()),
        })
    }
    // #endregion: setup

    // #region: frame
    fn frame(&self, renderer: &Renderer, target: &RenderTarget) -> Result<(), Failure> {
        let size = target.size();
        let (pointer, gesture) = self.input.lock().take(size);
        let motion = &mut *self.motion.lock();
        // The first frame counts the whole clock, so a frame drawn at `now` shows the dive at `now`.
        let now = seconds();
        let delta = motion.last.map_or(now, |last| (now - last).min(0.1));
        motion.last = Some(now);

        // The compute pass fills at most PIXELS: the window, or the window with each side divided by a whole number.
        let shrink = (1..)
            .find(|k| (size.width / k) * (size.height / k) <= PIXELS)
            .unwrap_or(1);
        let image = [(size.width / shrink).max(1), (size.height / shrink).max(1)];
        let budget: u32 = self.iterate.get("view.budget")?;

        // The first scroll, pinch or drag ends the dive. From then on the view is the reader's.
        if gesture.zoom != 0.0 || gesture.pan != [0.0, 0.0] {
            motion.exploring = true;
        }
        let deadline = Instant::now() + WORK;
        if motion.exploring {
            motion.explore(size, image, pointer, gesture, budget, deadline);
        } else {
            motion.dive(image, delta, budget, deadline);
        }

        // A pixel is 2^pixel wide, and the view's center sits away from the reference by `center` pixels.
        let reference = &motion.reference;
        let pixel = (START / f64::from(image[1])).log2() - motion.zoom;
        let exponent = pixel.floor();
        let center = motion.offset(reference, pixel);
        // The series covers every pixel within 2^radius of the reference.
        let spread = f64::from(image[0]).hypot(f64::from(image[1])) / 2.0;
        let radius = pixel + (spread + center[0].hypot(center[1])).log2();
        let skip = reference.skip(radius);
        let [a, b, c] = reference.series[skip].map(Wide::exact);

        // The GPU gets the orbit again whenever it grew or another reference took over.
        let sent = (reference.id, reference.length);
        if motion.sent != Some(sent) {
            self.iterate
                .set("orbit", bytemuck::cast_slice::<f32, u8>(&reference.orbit))?;
            motion.sent = Some(sent);
        }
        self.iterate.set("view.image", image)?;
        self.iterate.set("view.exponent", exponent as i32)?;
        self.iterate
            .set("view.fraction", (pixel - exponent).exp2() as f32)?;
        self.iterate
            .set("view.center", [center[0] as f32, center[1] as f32])?;
        self.iterate.set("view.skip", skip as u32)?;
        self.iterate.set("view.length", reference.length as u32)?;
        self.iterate.set("view.a", [a.re as f32, a.im as f32])?;
        self.iterate.set("view.a_exponent", a.exponent)?;
        self.iterate.set("view.b", [b.re as f32, b.im as f32])?;
        self.iterate.set("view.b_exponent", b.exponent)?;
        self.iterate.set("view.c", [c.re as f32, c.im as f32])?;
        self.iterate.set("view.c_exponent", c.exponent)?;
        // 8 matches @workgroup_size in ITERATE.
        self.compute
            .set_compute_dispatch(image[0].div_ceil(8), image[1].div_ceil(8), 1);

        self.present
            .set("look.resolution", [size.width as f32, size.height as f32])?;
        self.present.set("look.image", image)?;
        let depth = motion.zoom * 2_f64.log10() - START.log10();
        self.present.set("look.depth", depth as f32)?;

        // When nothing moved, the counts from the last frame still hold, so only the colors redraw.
        let view = [
            f64::from(image[0]),
            f64::from(image[1]),
            pixel,
            center[0],
            center[1],
            skip as f64,
            reference.length as f64,
            reference.id as f64,
            f64::from(budget),
        ];
        if motion.drawn == Some(view) {
            renderer.render(&self.passes[1], target)?;
        } else {
            renderer.render(&self.passes, target)?;
        }
        motion.drawn = Some(view);
        Ok(())
    }
    // #endregion: frame
}

// #region: reference
// #region: series
// #region: grow
/// The orbit of 0 under `z^2 + c` at `bits` of precision, rounded to `f32` for the GPU,
/// with Martin's series coefficients A, B and C at every iteration. `reach` holds
/// the log2 of the widest radius the series still covers at each iteration.
/// `grow` adds to it until a deadline; `length` is the last index computed so far.
struct Reference {
    id: u64,
    cr: BigInt,
    ci: BigInt,
    bits: usize,
    orbit: Vec<f32>,
    series: Vec<[Wide; 3]>,
    reach: Vec<f64>,
    length: usize,
    done: bool,
    z: [BigInt; 2],
    coefficients: [Wide; 4],
    saved: Option<(f64, f64, f64)>,
    checkpoint: usize,
}

impl Reference {
    fn start([cr, ci]: [BigInt; 2], bits: usize) -> Reference {
        // Each reference has its own id, so a frame can tell when another one takes over.
        static COUNT: AtomicU64 = AtomicU64::new(0);
        // Iteration 0 is the same for every c: z and every coefficient are 0.
        // The next iteration starts from z = c and A = 1.
        Reference {
            id: COUNT.fetch_add(1, Ordering::Relaxed),
            z: [cr.clone(), ci.clone()],
            cr,
            ci,
            bits,
            orbit: vec![0.0; 2 * ORBIT],
            series: vec![[Wide::ZERO; 3]],
            reach: vec![f64::INFINITY; ORBIT],
            length: 0,
            done: false,
            coefficients: [Wide::ONE, Wide::ZERO, Wide::ZERO, Wide::ZERO],
            saved: None,
            checkpoint: 1,
        }
    }

    /// Adds iterations until the orbit ends or the deadline passes.
    fn grow(&mut self, deadline: Instant) {
        while !self.done && Instant::now() < deadline {
            for _ in 0..64 {
                if self.done {
                    break;
                }
                self.step();
            }
        }
    }

    /// Adds the next iteration of the orbit and the series.
    fn step(&mut self) {
        let n = self.length + 1;
        let [zr, zi] = &self.z;
        let (re, im) = (to_float(zr, self.bits), to_float(zi, self.bits));
        self.orbit[2 * n] = re as f32;
        self.orbit[2 * n + 1] = im as f32;
        let [mut a, mut b, mut c, mut d] = self.coefficients;
        self.series.push([a, b, c]);
        // The series holds while the first dropped term, D * delta^4, stays below TOLERANCE * A * delta.
        let limit = (TOLERANCE.log2() + a.log2_size() - d.log2_size()) / 3.0;
        self.reach[n] = self.reach[n - 1].min(limit);
        self.length = n;
        // Rounding errors in the orbit grow like A, so the loop ends before they reach 2^-40.
        // An orbit caught in an attracting cycle ends too: it repeats, A stops changing,
        // and nothing after adds anything. Brent's method compares it with powers of two.
        let settled = self.saved.is_some_and(|(real, imaginary, size)| {
            (re - real).hypot(im - imaginary) < 1e-12 && (a.log2_size() - size).abs() < 1e-9
        });
        let bits = self.bits as f64;
        if n == ORBIT - 1 || re * re + im * im > 4.0 || a.log2_size() > bits - 40.0 || settled {
            self.done = true;
            return;
        }
        if n == self.checkpoint {
            self.saved = Some((re, im, a.log2_size()));
            self.checkpoint *= 2;
        }
        // Each new coefficient reads the old values of the ones before it, so D updates first and A last.
        let twice = (2.0 * re, 2.0 * im);
        d = d
            .times(twice)
            .add(a.multiply(c).times((2.0, 0.0)).add(b.multiply(b)));
        c = c.times(twice).add(a.multiply(b).times((2.0, 0.0)));
        b = b.times(twice).add(a.multiply(a));
        a = a.times(twice).add(Wide::ONE);
        self.coefficients = [a, b, c, d];
        let next = ((zr * zr - zi * zi) >> self.bits) + &self.cr;
        let imaginary = ((2 * zr * zi) >> self.bits) + &self.ci;
        self.z = [next, imaginary];
    }
}
// #endregion: grow

/// The largest the first dropped term of the series may grow, relative to the first term.
const TOLERANCE: f64 = 1e-7;

impl Reference {
    /// The last iteration the series covers for a view of this radius (log2).
    fn skip(&self, radius: f64) -> usize {
        // The last step reads one entry past skip, so skip stops one short of the end.
        let covered = self.reach[..self.length.max(1)].partition_point(|&reach| reach > radius);
        covered.saturating_sub(1)
    }
}
// #endregion: series

/// A `BigInt` `n` stands for the real number `n / 2^bits`.
fn parse(text: &str, bits: usize) -> Result<BigInt, Failure> {
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text),
    };
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    let value = BigInt::parse_bytes(format!("{whole}{fraction}").as_bytes(), 10)
        .ok_or_else(|| format!("{text} is not a decimal number"))?;
    let scale = BigInt::from(10).pow(fraction.len() as u32);
    Ok(sign * ((value << bits) / scale))
}

fn to_float(value: &BigInt, bits: usize) -> f64 {
    (value >> (bits - 60)).to_f64().unwrap_or(f64::NAN) / 2_f64.powi(60)
}

/// Newton's method on z(preperiod + period) - z(preperiod) moves `c` until the orbit
/// of 0 enters its cycle, to the working precision. Each round doubles the correct digits.
fn refine(target: &Point, bits: usize) -> Result<(BigInt, BigInt), Failure> {
    let one = BigInt::from(1) << bits;
    let mut re = parse(target.re, bits)?;
    let mut im = parse(target.im, bits)?;
    for _ in 0..64 {
        let [mut zr, mut zi, mut dr, mut di] = [(); 4].map(|_| BigInt::ZERO);
        let mut kept = [(); 4].map(|_| BigInt::ZERO);
        for n in 0..target.preperiod + target.period {
            if n == target.preperiod {
                kept = [zr.clone(), zi.clone(), dr.clone(), di.clone()];
            }
            // dz/dc becomes 2 z dz/dc + 1, then z becomes z^2 + c.
            let next_dr = ((2 * (&zr * &dr - &zi * &di)) >> bits) + &one;
            di = (2 * (&zr * &di + &zi * &dr)) >> bits;
            dr = next_dr;
            let next_zr = ((&zr * &zr - &zi * &zi) >> bits) + &re;
            zi = ((2 * &zr * &zi) >> bits) + &im;
            zr = next_zr;
        }
        let [kr, ki, kdr, kdi] = kept;
        let (fr, fi, gr, gi) = (zr - kr, zi - ki, dr - kdr, di - kdi);
        let size = (&gr * &gr + &gi * &gi) >> bits;
        if size == BigInt::ZERO {
            break;
        }
        let sr = (((&fr * &gr + &fi * &gi) >> bits) << bits) / &size;
        let si = (((&fi * &gr - &fr * &gi) >> bits) << bits) / &size;
        re -= &sr;
        im -= &si;
        if sr.abs() + si.abs() < BigInt::from(256) {
            break;
        }
    }
    Ok((re, im))
}

/// A complex number beyond the range of an `f64`: `(re + i im) * 2^exponent`.
#[derive(Clone, Copy)]
struct Wide {
    re: f64,
    im: f64,
    exponent: i32,
}

impl Wide {
    const ZERO: Wide = Wide {
        re: 0.0,
        im: 0.0,
        exponent: 0,
    };
    const ONE: Wide = Wide {
        re: 1.0,
        im: 0.0,
        exponent: 0,
    };

    fn new(re: f64, im: f64, exponent: i32) -> Wide {
        let size = re.abs().max(im.abs());
        if size == 0.0 {
            return Wide::ZERO;
        }
        if size > 2_f64.powi(-64) && size < 2_f64.powi(64) {
            return Wide { re, im, exponent };
        }
        let k = size.log2().round() as i32;
        let scale = 2_f64.powi(-k);
        Wide {
            re: re * scale,
            im: im * scale,
            exponent: exponent + k,
        }
    }

    /// The same number with re and im below 1, as the shader expects.
    fn exact(self) -> Wide {
        let size = self.re.abs().max(self.im.abs());
        if size == 0.0 {
            return Wide::ZERO;
        }
        let k = size.log2().floor() as i32 + 1;
        let scale = 2_f64.powi(-k);
        Wide {
            re: self.re * scale,
            im: self.im * scale,
            exponent: self.exponent + k,
        }
    }

    fn add(self, other: Wide) -> Wide {
        let exponent = self.exponent.max(other.exponent);
        let a = 2_f64.powi(self.exponent - exponent);
        let b = 2_f64.powi(other.exponent - exponent);
        Wide::new(
            self.re * a + other.re * b,
            self.im * a + other.im * b,
            exponent,
        )
    }

    fn multiply(self, other: Wide) -> Wide {
        Wide::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
            self.exponent + other.exponent,
        )
    }

    /// self * (re + i im), for a plain complex number.
    fn times(self, (re, im): (f64, f64)) -> Wide {
        Wide::new(
            self.re * re - self.im * im,
            self.re * im + self.im * re,
            self.exponent,
        )
    }

    fn log2_size(self) -> f64 {
        self.re.hypot(self.im).log2() + f64::from(self.exponent)
    }
}
// #endregion: reference

/// Where the view is and how it moves: its zoom in doublings,
/// its center at `bits` of precision, and the reference orbits that serve it.
struct Motion {
    reference: Reference,
    next: Option<Reference>,
    center: [BigInt; 2],
    bits: usize,
    zoom: f64,
    direction: f64,
    last: Option<f64>,
    exploring: bool,
    sent: Option<(u64, usize)>,
    drawn: Option<[f64; 9]>,
}

// #region: dive
impl Motion {
    /// The zoom runs from the whole set down to 10^-DEPTH and back on its own, never
    /// deeper than the reference orbit computed so far covers.
    fn dive(&mut self, image: [u32; 2], delta: f64, budget: u32, deadline: Instant) {
        self.reference.grow(deadline);
        let deepest = DEPTH * 10_f64.log2() + START.log2();
        let zoom = (self.zoom + self.direction * SPEED * delta).clamp(0.0, deepest);
        if covers(&self.reference, image, zoom, [0.0, 0.0], budget) {
            self.zoom = zoom;
        }
        if self.zoom >= deepest {
            self.direction = -1.0;
        }
        if self.zoom <= 0.0 {
            self.direction = 1.0;
        }
    }
}

/// Whether this reference serves a view at this zoom: its orbit is complete, or long
/// enough for the series skip plus every pixel's budget.
fn covers(
    reference: &Reference,
    image: [u32; 2],
    zoom: f64,
    center: [f64; 2],
    budget: u32,
) -> bool {
    if reference.done {
        return true;
    }
    let pixel = (START / f64::from(image[1])).log2() - zoom;
    let spread = f64::from(image[0]).hypot(f64::from(image[1])) / 2.0;
    let radius = pixel + (spread + center[0].hypot(center[1])).log2();
    reference.skip(radius) + budget as usize + 200 <= reference.length
}
// #endregion: dive

// #region: explore
impl Motion {
    /// The reader's view: a pinch or the wheel zooms around the pointer, and a drag pans.
    /// When the view drifts away from the reference, or zooms past its digits, a new
    /// reference starts where the view is heading and takes over once it covers the view.
    fn explore(
        &mut self,
        size: Size,
        image: [u32; 2],
        pointer: [f64; 2],
        gesture: Gesture,
        budget: u32,
        deadline: Instant,
    ) {
        let pixel = (START / f64::from(image[1])).log2() - self.zoom;
        // The pointer and the drag, in image pixels with y pointing up.
        let [width, height] = [f64::from(size.width), f64::from(size.height)];
        let scale = f64::from(image[1]) / height;
        let at = [
            (pointer[0] - width / 2.0) * scale,
            (height / 2.0 - pointer[1]) * scale,
        ];
        // #region: pan
        // Dragging right moves the picture right, so the center moves left.
        self.center = self.point_at([-gesture.pan[0] * scale, gesture.pan[1] * scale], pixel);
        // #endregion: pan

        // #region: zoom
        let deepest = DEPTH * 10_f64.log2() + START.log2();
        let zoom = (self.zoom + gesture.zoom).clamp(0.0, deepest);
        let after = (START / f64::from(image[1])).log2() - zoom;
        // The center needs a few more digits than the deepest pixel it has seen.
        if self.bits < (-after).ceil() as usize + 64 {
            for value in &mut self.center {
                *value <<= 128;
            }
            self.bits += 128;
        }
        // Zooming around the pointer keeps the point under it in place: the center moves
        // toward it by `stay` of the way, and the reference's offset moves with it.
        let stay = 1.0 - (self.zoom - zoom).exp2();
        let reference = &self.reference;
        let shift = (pixel - after).exp2() * stay;
        let [x, y] = self.offset(reference, after);
        let drift = [x + at[0] * shift, y + at[1] * shift];
        // Zooming in waits while the reference lacks the digits, is more than two views
        // from where the zoom lands, or hasn't grown far enough yet.
        let near = drift[0].hypot(drift[1]) <= 2.0 * f64::from(image[1]);
        let deep = reference.bits >= (-after).ceil() as usize + 40;
        let far = zoom > self.zoom && !(near && deep);
        if zoom < self.zoom || (!far && covers(reference, image, zoom, drift, budget)) {
            self.center = self.point_at([at[0] * stay, at[1] * stay], pixel);
            self.zoom = zoom;
        }
        // #endregion: zoom

        // #region: follow
        // A zoom in heads for the point under the pointer; anything else stays at the center.
        let now = (START / f64::from(image[1])).log2() - self.zoom;
        let aim = if gesture.zoom > 0.0 { at } else { [0.0, 0.0] };
        // The offset is the center minus the reference, so a reference at the aim sits at -aim.
        let away = |other: &Reference, limit: f64| {
            let [x, y] = self.offset(other, now);
            (x + aim[0]).hypot(y + aim[1]) > limit
        };
        // A zoom that waits on a far or shallow reference gets a new one, with the digits it needs.
        let finest = if far { after } else { now };
        let stale = far
            || away(&self.reference, f64::from(image[1]) / 4.0)
            || self.reference.bits < (-now).ceil() as usize + 120;
        let half = f64::from(image[1]) / 2.0;
        let shallow = (-finest).ceil() as usize + 40;
        if stale && self.next.as_ref().is_none_or(|next| away(next, half) || next.bits < shallow) {
            let bits = (-finest).ceil() as usize + 200;
            let goal = self.point_at(aim, now);
            self.next = Some(Reference::start(
                align(goal.each_ref(), self.bits, bits),
                bits,
            ));
        }
        if let Some(mut next) = self.next.take() {
            next.grow(deadline);
            let distance = self.offset(&next, now);
            if covers(&next, image, self.zoom, distance, budget) {
                self.reference = next;
            } else {
                self.next = Some(next);
            }
        }
        self.reference.grow(deadline);
        // #endregion: follow
    }
}
// #endregion: explore

// #region: view
impl Motion {
    /// The point (x, y) pixels from the view's center, as fixed point numbers at `bits`.
    fn point_at(&self, [x, y]: [f64; 2], pixel: f64) -> [BigInt; 2] {
        let scale = pixel + self.bits as f64;
        let lift = (scale.floor() - 40.0).max(0.0);
        let step = |value: f64| {
            BigInt::from((value * (scale - lift).exp2()).round() as i64) << (lift as usize)
        };
        [&self.center[0] + step(x), &self.center[1] + step(y)]
    }

    /// The view's center minus the reference's c, in pixels.
    fn offset(&self, reference: &Reference, pixel: f64) -> [f64; 2] {
        let bits = self.bits.max(reference.bits);
        let [cr, ci] = align(self.center.each_ref(), self.bits, bits);
        let [rr, ri] = align([&reference.cr, &reference.ci], reference.bits, bits);
        let power = -(bits as f64) - pixel;
        [scaled(&(cr - rr), power), scaled(&(ci - ri), power)]
    }
}

/// `value * 2^power` as an `f64`, for a `BigInt` of any size.
fn scaled(value: &BigInt, power: f64) -> f64 {
    let drop = value.bits().saturating_sub(60);
    (value >> drop).to_f64().unwrap_or(0.0) * (power + drop as f64).exp2()
}

/// The same fixed point numbers at another precision.
fn align(values: [&BigInt; 2], from: usize, to: usize) -> [BigInt; 2] {
    values.map(|value| {
        if to >= from {
            value << (to - from)
        } else {
            value >> (from - to)
        }
    })
}
// #endregion: view

/// What the wheel, a pinch and a drag did since the last frame:
/// zoom in doublings, positive inward, and pan in window pixels with y down.
#[derive(Clone, Copy, Default)]
struct Gesture {
    zoom: f64,
    pan: [f64; 2],
}

/// The cursor in window pixels and the left button,
/// with the gesture they and the wheel gathered since the last frame.
#[derive(Default)]
struct Input {
    pointer: Option<[f64; 2]>,
    dragging: bool,
    gesture: Gesture,
}

impl Input {
    /// Moves the pointer. While the left button is down, the move pans the view.
    fn point(&mut self, position: [f64; 2]) {
        if self.dragging
            && let Some([x, y]) = self.pointer
        {
            self.gesture.pan[0] += position[0] - x;
            self.gesture.pan[1] += position[1] - y;
        }
        self.pointer = Some(position);
    }

    /// The pointer, at the window's center until the cursor first moves,
    /// and the gesture since the last call, which then starts over from nothing.
    fn take(&mut self, size: Size) -> ([f64; 2], Gesture) {
        let center = [f64::from(size.width) / 2.0, f64::from(size.height) / 2.0];
        (self.pointer.unwrap_or(center), mem::take(&mut self.gesture))
    }
}

fn seconds() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn on_cursor_moved(app: &App, _device: DeviceId, position: &PhysicalPosition<f64>) {
    let Some(zoom) = app.get_state::<Zoom>("zoom") else {
        return;
    };
    zoom.input.lock().point([position.x, position.y]);
}

fn on_button(app: &App, _device: DeviceId, state: ElementState, button: MouseButton) {
    let Some(zoom) = app.get_state::<Zoom>("zoom") else {
        return;
    };
    if button == MouseButton::Left {
        zoom.input.lock().dragging = state.is_pressed();
    }
}

fn on_wheel(app: &App, _device: DeviceId, delta: &MouseScrollDelta, _phase: TouchPhase) {
    let Some(zoom) = app.get_state::<Zoom>("zoom") else {
        return;
    };
    // A window has no page to scroll, so any wheel zooms. Scrolling up zooms in,
    // and a notch, like 100 logical pixels of a trackpad, is a quarter doubling.
    let doublings = match delta {
        MouseScrollDelta::LineDelta(_, lines) => f64::from(*lines) / 4.0,
        MouseScrollDelta::PixelDelta(pixels) => {
            let window = app.window(app.primary_window_id());
            let scale = window.map_or(1.0, |window| window.scale_factor());
            pixels.to_logical::<f64>(scale).y / 400.0
        }
    };
    zoom.input.lock().gesture.zoom += doublings;
}

fn on_pinch(app: &App, _device: DeviceId, delta: f64, _phase: TouchPhase) {
    let Some(zoom) = app.get_state::<Zoom>("zoom") else {
        return;
    };
    // A trackpad pinch scales the picture by 1 + delta, which is log2(1 + delta) doublings.
    if delta > -1.0 {
        zoom.input.lock().gesture.zoom += (1.0 + delta).log2();
    }
}

fn on_resize(app: &App, size: &PhysicalSize<u32>) {
    app.resize([size.width, size.height]);
}

fn on_redraw(app: &App) {
    let Some(zoom) = app.get_state::<Zoom>("zoom") else {
        return;
    };
    let renderer = app.get_renderer();
    let rendered = app.with_target(app.primary_window_id(), |target| {
        zoom.frame(renderer, target)
    });
    if let Some(Err(error)) = rendered {
        // A hidden or minimized window has nothing to draw on, so the frame is skipped.
        let occluded = matches!(
            error.downcast_ref::<RendererError>(),
            Some(RendererError::SurfaceError(SurfaceError::Occluded))
        );
        if !occluded {
            eprintln!("infinite zoom: {error}");
        }
    }
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    for window in windows {
        let target = app.get_renderer().create_target(window.clone()).await?;
        app.add_target(window.id(), target);
    }
    Ok(())
}

fn main() -> Result<(), Failure> {
    let _ = env_logger::try_init();
    let mut app = App::new(Renderer::new());
    app.add_state("zoom", Zoom::new()?);
    app.on_start(call!(setup))
        .on_resize(on_resize)
        .on_cursor_moved(on_cursor_moved)
        .on_mouse_input(on_button)
        .on_mouse_wheel(on_wheel)
        .on_pinch_gesture(on_pinch)
        .on_redraw_requested(on_redraw);
    app.run();
    Ok(())
}
