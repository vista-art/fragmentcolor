//! Infinite zoom, step 4: past every exponent.
//!
//! One reference orbit, computed once with BigInt, guides a compute pass that
//! iterates every pixel as a small f32 difference from it. The difference is
//! kept as w * 2^s, so it never leaves f32's range, and Martin's series skips
//! the iterations every pixel shares. A render pass colors the counts. The zoom
//! dives toward a spiral until the view is 1e-1000 tall and climbs back, and the
//! pointer's height picks the depth while it moves.

use std::error::Error;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use fragmentcolor::{App, Pass, RenderTarget, Renderer, SetupResult, Shader, Target, call};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use parking_lot::Mutex;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::DeviceId;
use winit::window::Window;

type Failure = Box<dyn Error + Send + Sync>;

const ITERATE: &str = r#"
// #region: shader
// Iterates every pixel as a small difference from the reference orbit that
// main.js computes at full precision. main.js puts ORBIT and PIXELS ahead of
// this file, and present.wgsl reads the counts it writes.

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

// An orbit whose squared size passes this never comes back.
const ESCAPE: f32 = 65536.0;

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// x * 2^k, or zero once 2^k is too small for an f32.
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
  // that never escape. A direct test spares them the whole budget.
  if (e > -20) {
    let c = orbit[1] + scale(d, e);
    let q = dot(c - vec2<f32>(0.25, 0.0), c - vec2<f32>(0.25, 0.0));
    let bulb = c + vec2<f32>(1.0, 0.0);
    if (q * (q + c.x - 0.25) < 0.25 * c.y * c.y || dot(bulb, bulb) < 0.0625) {
      counts[index] = -1.0;
      return;
    }
  }

  // The series moves every pixel to iteration skip in one step. Its difference
  // from the reference is w * 2^s, with w kept near 1.
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
      // z becomes 2 Z z + z^2 + d, with every term divided by 2^s.
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
    // Rebase when the pixel is closer to 0 than to the reference, or the orbit runs out.
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
  // A smooth count: how far past ESCAPE the last step went adds the fraction.
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
// Colors the counts that iterate.wgsl writes. label.wgsl prints the depth.

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

// Fullscreen triangle: three vertices that cover the canvas.
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
  let exponent = u32(max(depth, 0.0));
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
/// A spiral in Seahorse Valley. Newton's method sharpens these digits to any precision.
const TARGET: Point = Point {
    re: "-0.7432918908524302",
    im: "0.1312405523087976",
    preperiod: 24,
    period: 2,
};
/// The zoom dives until the view is 10^-DEPTH tall, then climbs back.
const DEPTH: f64 = 1000.0;
/// Zoom speed, in doublings per second.
const SPEED: f64 = 1.5;
/// Height of the first view.
const START: f64 = 3.0;
/// Room for the reference orbit, and for the pixels the compute pass iterates.
/// A larger window renders at a lower resolution.
const ORBIT: usize = 1 << 16;
const PIXELS: u32 = 1 << 21;
/// Iterations the orbit keeps past the deepest view, for the budget.
const BUDGET: usize = 5000;
// #endregion: constants

struct Zoom {
    iterate: Shader,
    present: Shader,
    compute: Pass,
    passes: Vec<Pass>,
    reference: Reference,
    motion: Mutex<Motion>,
}

impl Zoom {
    // #region: setup
    fn new() -> Result<Zoom, Failure> {
        let header = format!("const ORBIT: u32 = {ORBIT}u;\nconst PIXELS: u32 = {PIXELS}u;");

        let iterate = Shader::new([header.as_str(), ITERATE])?;
        iterate.set("view.budget", 1000_u32)?;

        // The library brings the palette that present.wgsl calls.
        let present = Shader::new(["gradient/palette_iq", header.as_str(), LABEL, PRESENT])?;
        present.set("look.density", 0.02_f32)?;
        present.set("look.phase", 0.6_f32)?;
        present.set("look.inside_color", [0.02_f32, 0.02, 0.05])?;

        // One orbit at the precision the deepest view needs, sent to the GPU once.
        let reference = Reference::new(&TARGET, DEPTH)?;
        iterate.set("orbit", bytemuck::cast_slice::<f32, u8>(&reference.orbit))?;

        let compute = Pass::from_shader("iterate", &iterate);
        let draw = Pass::from_shader("present", &present);
        Ok(Zoom {
            passes: vec![compute.clone(), draw],
            iterate,
            present,
            compute,
            reference,
            motion: Mutex::new(Motion::new()),
        })
    }
    // #endregion: setup

    // #region: frame
    fn frame(&self, renderer: &Renderer, target: &RenderTarget) -> Result<(), Failure> {
        let zoom = self.motion.lock().steer(seconds());
        let size = target.size();

        // The compute pass fills at most PIXELS: the window, or a whole fraction of it.
        let shrink = (1..)
            .find(|k| (size.width / k) * (size.height / k) <= PIXELS)
            .unwrap_or(1);
        let image = [(size.width / shrink).max(1), (size.height / shrink).max(1)];

        // A pixel is 2^pixel wide. The series covers every pixel within radius of the target.
        let pixel = (START / f64::from(image[1])).log2() - zoom;
        let exponent = pixel.floor();
        let radius = pixel + (f64::from(image[0]).hypot(f64::from(image[1])) / 2.0).log2();
        let skip = self.reference.skip(radius);
        let [a, b, c] = self.reference.series[skip].map(Wide::exact);

        self.iterate.set("view.image", image)?;
        self.iterate.set("view.exponent", exponent as i32)?;
        self.iterate
            .set("view.fraction", (pixel - exponent).exp2() as f32)?;
        self.iterate.set("view.skip", skip as u32)?;
        self.iterate
            .set("view.length", self.reference.length as u32)?;
        self.iterate.set("view.a", [a.re as f32, a.im as f32])?;
        self.iterate.set("view.a_exponent", a.exponent)?;
        self.iterate.set("view.b", [b.re as f32, b.im as f32])?;
        self.iterate.set("view.b_exponent", b.exponent)?;
        self.iterate.set("view.c", [c.re as f32, c.im as f32])?;
        self.iterate.set("view.c_exponent", c.exponent)?;
        // 8 matches @workgroup_size in iterate.wgsl.
        self.compute
            .set_compute_dispatch(image[0].div_ceil(8), image[1].div_ceil(8), 1);

        self.present
            .set("look.resolution", [size.width as f32, size.height as f32])?;
        self.present.set("look.image", image)?;
        let depth = zoom * 2_f64.log10() - START.log10();
        self.present.set("look.depth", depth as f32)?;
        renderer.render(&self.passes, target)?;
        Ok(())
    }
    // #endregion: frame
}

// #region: reference
/// The orbit of 0 under z^2 + c at full precision, rounded to f32 for the GPU,
/// with Martin's series coefficients A, B and C at every iteration. `reach` holds
/// the log2 of the widest radius the series still covers at each iteration.
struct Reference {
    orbit: Vec<f32>,
    series: Vec<[Wide; 3]>,
    reach: Vec<f64>,
    length: usize,
}

/// How far the dropped terms of the series may drift, relative to the first one.
const TOLERANCE: f64 = 1e-7;

impl Reference {
    fn new(target: &Point, depth: f64) -> Result<Reference, Failure> {
        let deepest = -depth * 10_f64.log2() - 1.0;
        let bits = (depth * 10_f64.log2()).ceil() as usize + 80;
        let (cr, ci) = refine(target, bits)?;
        let mut orbit = vec![0.0_f32; 2 * ORBIT];
        let mut series = Vec::new();
        let mut reach = vec![f64::INFINITY; ORBIT];
        let (mut zr, mut zi) = (BigInt::ZERO, BigInt::ZERO);
        let [mut a, mut b, mut c, mut d] = [Wide::ZERO; 4];
        let mut end = ORBIT - 1;
        for n in 0..ORBIT {
            let (re, im) = (to_float(&zr, bits), to_float(&zi, bits));
            orbit[2 * n] = re as f32;
            orbit[2 * n + 1] = im as f32;
            series.push([a, b, c]);
            // The series holds while the first dropped term, D d^4, stays below TOLERANCE * A d.
            if n > 0 {
                let limit = (TOLERANCE.log2() + a.log2_size() - d.log2_size()) / 3.0;
                reach[n] = reach[n - 1].min(limit);
            }
            if reach[n] < deepest {
                end = end.min(n + BUDGET);
            }
            if n == end || re * re + im * im > 4.0 {
                end = n;
                break;
            }
            // Each coefficient follows from the old ones, so they update in this order.
            let twice = (2.0 * re, 2.0 * im);
            d = d
                .times(twice)
                .add(a.multiply(c).times((2.0, 0.0)).add(b.multiply(b)));
            c = c.times(twice).add(a.multiply(b).times((2.0, 0.0)));
            b = b.times(twice).add(a.multiply(a));
            a = a.times(twice).add(Wide::ONE);
            let next = ((&zr * &zr - &zi * &zi) >> bits) + &cr;
            zi = ((2 * &zr * &zi) >> bits) + &ci;
            zr = next;
        }
        Ok(Reference {
            orbit,
            series,
            reach,
            length: end,
        })
    }

    /// The last iteration the series covers for a view of this radius (log2).
    fn skip(&self, radius: f64) -> usize {
        let covered = self.reach[..=self.length].partition_point(|&reach| reach > radius);
        covered.saturating_sub(1)
    }
}

/// A BigInt n stands for the real number n / 2^bits.
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

/// Newton's method on z(preperiod + period) - z(preperiod) moves c until the orbit
/// of 0 lands exactly on its cycle. Each round doubles the correct digits.
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
        if sr.magnitude().bits() + si.magnitude().bits() < 16 {
            break;
        }
    }
    Ok((re, im))
}

/// A complex number beyond a double's range: (re + i im) * 2^exponent.
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

/// Where the zoom is, in doublings from the first view, and where the pointer
/// last moved, as fractions of the window.
struct Motion {
    zoom: f64,
    direction: f64,
    last: Option<f64>,
    pointer: [f64; 2],
    moved: f64,
}

impl Motion {
    fn new() -> Motion {
        Motion {
            zoom: 0.0,
            direction: 1.0,
            last: None,
            pointer: [0.5, 0.5],
            moved: f64::NEG_INFINITY,
        }
    }

    /// The zoom runs from the whole set down to 10^-DEPTH and back on its own.
    /// Moving the pointer takes over for two seconds: its height picks the depth,
    /// deepest at the top. Returns the zoom in doublings.
    fn steer(&mut self, now: f64) -> f64 {
        let delta = self.last.map_or(0.0, |last| (now - last).min(0.1));
        self.last = Some(now);
        let deepest = DEPTH * 10_f64.log2() + START.log2();
        if now - self.moved < 2.0 {
            let goal = (1.0 - self.pointer[1]) * deepest;
            self.zoom += (goal - self.zoom) * (delta * 4.0).min(1.0);
        } else {
            self.zoom += self.direction * SPEED * delta;
            if self.zoom >= deepest {
                self.direction = -1.0;
            }
            if self.zoom <= 0.0 {
                self.direction = 1.0;
            }
        }
        self.zoom = self.zoom.clamp(0.0, deepest);
        self.zoom
    }

    fn point(&mut self, now: f64, pointer: [f64; 2]) {
        self.pointer = pointer;
        self.moved = now;
    }
}

fn seconds() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}
fn on_cursor_moved(app: &App, _device: DeviceId, position: &PhysicalPosition<f64>) {
    let (Some(zoom), Some(size)) = (
        app.get_state::<Zoom>("zoom"),
        app.window_size(app.primary_window_id()),
    ) else {
        return;
    };
    let pointer = [
        position.x / f64::from(size.width.max(1)),
        position.y / f64::from(size.height.max(1)),
    ];
    zoom.motion.lock().point(seconds(), pointer);
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
        eprintln!("infinite zoom: {error}");
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
        .on_redraw_requested(on_redraw);
    app.run();
    Ok(())
}
