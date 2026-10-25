//! Infinite zoom, step 2: the same loop on the GPU.
//!
//! A fragment shader runs the escape-time loop for every pixel in parallel, in
//! f32. The view zooms toward a spiral in Seahorse Valley and back out, and the
//! pointer's vertical position picks the depth while it moves. Past 1e-5, f32 rounding
//! breaks the picture into blocks.

use std::error::Error;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use fragmentcolor::{
    App, RenderTarget, Renderer, RendererError, SetupResult, Shader, SurfaceError, Target, call,
};
use parking_lot::Mutex;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::DeviceId;
use winit::window::Window;

type Failure = Box<dyn Error + Send + Sync>;

const SHADER: &str = r#"
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

// #region: constants
/// A spiral in Seahorse Valley, the point the zoom approaches.
const TARGET: [f64; 2] = [-0.7432918908524302, 0.1312405523087976];
/// The zoom goes in until the view is 10^-DEPTH tall, then back out.
const DEPTH: f64 = 7.0;
/// Zoom speed, in doublings per second.
const SPEED: f64 = 2.0;
/// Height of the first view.
const START: f64 = 3.0;
// #endregion: constants

struct Zoom {
    shader: Shader,
    motion: Mutex<Motion>,
}

impl Zoom {
    // #region: setup
    fn new() -> Result<Zoom, Failure> {
        let shader = Shader::new(["gradient/palette_iq", LABEL, SHADER])?;
        shader.set("view.center", [TARGET[0] as f32, TARGET[1] as f32])?;
        Ok(Zoom {
            shader,
            motion: Mutex::new(Motion::new()),
        })
    }
    // #endregion: setup

    // #region: frame
    fn frame(&self, renderer: &Renderer, target: &RenderTarget) -> Result<(), Failure> {
        let zoom = self.motion.lock().steer(seconds());
        let size = target.size();
        let height = START * (-zoom).exp2();
        self.shader
            .set("view.resolution", [size.width as f32, size.height as f32])?;
        self.shader.set("view.height", height as f32)?;
        self.shader.set("view.depth", -height.log10() as f32)?;
        renderer.render(&self.shader, target)?;
        Ok(())
    }
    // #endregion: frame
}

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

    // #region: steer
    /// The zoom runs from the whole set down to 10^-DEPTH and back on its own.
    /// While the pointer moves, and for two seconds after, its vertical position
    /// picks the depth, deepest at the top. Returns the zoom in doublings.
    fn steer(&mut self, now: f64) -> f64 {
        // The first frame counts the whole clock, so a frame drawn at `now` shows the dive at `now`.
        let delta = self.last.map_or(now, |last| (now - last).min(0.1));
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
    // #endregion: steer

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
        .on_redraw_requested(on_redraw);
    app.run();
    Ok(())
}
