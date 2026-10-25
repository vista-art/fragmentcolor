//! Particle life on a grid.
//!
//! Four species of particles steer along each other's density.
//! They gather into cells, chase and flee each other, and swirl like a fluid.
//! Each frame runs four passes. `field` blurs the particles counted per grid cell
//! into density slopes, and `clear` empties the counts and the image. `update` moves
//! each particle along the slopes, counts it into the grid with `atomicAdd` and adds
//! its color to the image. `present` tone-maps the image to the window without stretching.
//!
//! Hold the left mouse button to attract and the right one to repel. R reshuffles
//! who follows whom, C cycles the colors (species, speed, density), and Up and Down
//! raise and lower the temperature. `PARTICLES` sets the count, one million by default.

use fragmentcolor::{App, Pass, Renderer, SetupResult, Shader, ShaderError, call};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

/// Grid rows. The columns follow the window's aspect ratio.
const ROWS: u32 = 96;
/// Grid columns for the widest window, 4:1.
const COLUMNS: u32 = ROWS * 4;
/// Image pixels. A larger window renders the image at a lower resolution.
const PIXELS: u32 = 1 << 21;
/// Mouse acceleration while a button is held, in cells per second squared.
const PULL: f32 = 600.0;
/// Starting temperature, the strength of random kicks.
const HEAT: f32 = 20.0;

/// WGSL every pass shares. `Life::new` puts the size constants in front of it.
const SHARED: &str = r#"
const CELLS: u32 = ROWS * COLUMNS;
const SPECIES: u32 = 4u;

// Position in [0, 1) across the window, velocity in cells per second.
struct Particle { pos: vec2<f32>, vel: vec2<f32> };

// Density slopes at one cell: each species along x and along y, then the crowd
// of all species, with its slope in xy and its density in z.
struct Field { slope_x: vec4<f32>, slope_y: vec4<f32>, crowd: vec4<f32> };
"#;

const FIELD: &str = r#"
@group(0) @binding(0) var<uniform> columns: u32;
@group(0) @binding(1) var<storage, read> counts: array<vec4<u32>, CELLS>;
@group(0) @binding(2) var<storage, read_write> field: array<Field, CELLS>;

const REACH: i32 = 4;    // kernel radius in cells
const SPREAD: f32 = 2.0; // how far species sense each other, in cells
const CROWD: f32 = 0.8;  // how far crowding pushes, in cells

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  if (gid.x >= columns || gid.y >= ROWS) { return; }
  // Densities relative to the mean, so forces look alike at any particle count.
  let scale = f32(columns * ROWS) / f32(N);

  // Gaussian-weighted slopes, normalized so a linear ramp reads as its own slope.
  var slope_x = vec4<f32>(0.0);
  var slope_y = vec4<f32>(0.0);
  var crowd = vec3<f32>(0.0);
  var sense_norm = 0.0;
  var crowd_norm = vec2<f32>(0.0);
  for (var dy = -REACH; dy <= REACH; dy++) {
    for (var dx = -REACH; dx <= REACH; dx++) {
      let x = u32(i32(gid.x) + dx + i32(columns)) % columns;
      let y = u32(i32(gid.y) + dy + i32(ROWS)) % ROWS;
      let density = vec4<f32>(counts[y * COLUMNS + x]) * scale;
      let total = density.x + density.y + density.z + density.w;
      let offset = vec2<f32>(f32(dx), f32(dy));
      let squared = dot(offset, offset);
      let sense = exp(-squared / (2.0 * SPREAD * SPREAD));
      let push = exp(-squared / (2.0 * CROWD * CROWD));
      slope_x += density * sense * offset.x;
      slope_y += density * sense * offset.y;
      crowd += vec3<f32>(offset, 1.0) * total * push;
      sense_norm += sense * offset.x * offset.x;
      crowd_norm += vec2<f32>(push * offset.x * offset.x, push);
    }
  }
  field[gid.y * COLUMNS + gid.x] = Field(
    slope_x / sense_norm,
    slope_y / sense_norm,
    vec4<f32>(crowd.xy / crowd_norm.x, crowd.z / crowd_norm.y, 0.0),
  );
}
"#;

const CLEAR: &str = r#"
@group(0) @binding(0) var<storage, read_write> counts: array<vec4<u32>, CELLS>;
@group(0) @binding(1) var<storage, read_write> image: array<u32, PIXELS * 3u>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i < CELLS) { counts[i] = vec4<u32>(0u); }
  if (i < PIXELS) {
    image[i * 3u] = 0u;
    image[i * 3u + 1u] = 0u;
    image[i * 3u + 2u] = 0u;
  }
}
"#;

const UPDATE: &str = r#"
struct Params {
  attract: mat4x4<f32>, // attract[a][b]: how species a follows (+) or flees (-) species b
  mouse: vec2<f32>,     // cursor in [0, 1) across the window
  pull: f32,            // mouse force: positive attracts, negative repels
  heat: f32,            // strength of random kicks
  columns: u32,         // grid columns in use
  frame: u32,
  image: vec2<u32>,     // image size in pixels
  mode: u32,            // colors: 0 species, 1 speed, 2 density
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle, N>;
@group(0) @binding(2) var<storage, read> field: array<Field, CELLS>;
@group(0) @binding(3) var<storage, read_write> counts: array<atomic<u32>, CELLS * SPECIES>;
@group(0) @binding(4) var<storage, read_write> image: array<atomic<u32>, PIXELS * 3u>;

const DT: f32 = 1.0 / 60.0;
const FORCE: f32 = 60.0;     // acceleration per unit of density slope
const COHESION: f32 = 0.6;   // pull every species adds to the table, so crowds form cells
const PRESSURE: f32 = 0.3;   // how hard crowds push back, relative to FORCE
const FRICTION: f32 = 4.0;   // velocity decay per second
const TOP_SPEED: f32 = 40.0; // cells per second
const REACH: f32 = 16.0;     // mouse reach in cells
const HOT: f32 = 10.0;       // speed that shades as the hottest color

// The index of a grid cell, wrapping around the edges.
fn index(x: i32, y: i32) -> u32 {
  let column = u32(x + i32(params.columns)) % params.columns;
  let row = u32(y + i32(ROWS)) % ROWS;
  return row * COLUMNS + column;
}

// Two random numbers in [0, 1).
fn random(a: u32, b: u32) -> vec2<f32> {
  return vec2<f32>(pcg2d(vec2<u32>(a, b)) >> vec2<u32>(8u)) / 16777216.0;
}

fn blend(a: Field, b: Field, t: f32) -> Field {
  return Field(mix(a.slope_x, b.slope_x, t), mix(a.slope_y, b.slope_y, t), mix(a.crowd, b.crowd, t));
}

// The field at a point, blended from the four nearest cell centers.
fn sample(pos: vec2<f32>) -> Field {
  let corner = pos - 0.5;
  let x = i32(floor(corner.x));
  let y = i32(floor(corner.y));
  let t = fract(corner);
  let top = blend(field[index(x, y)], field[index(x + 1, y)], t.x);
  let bottom = blend(field[index(x, y + 1)], field[index(x + 1, y + 1)], t.x);
  return blend(top, bottom, t.y);
}

fn shade(species: u32, speed: f32, density: f32) -> vec3<f32> {
  switch params.mode {
    case 1u: { return turbo(speed); }
    case 2u: { return turbo(density / 8.0); }
    default: {
      let hue = palette_iq(
        f32(species) / f32(SPECIES),
        vec3<f32>(0.5), vec3<f32>(0.5), vec3<f32>(1.0), vec3<f32>(0.0, 0.33, 0.67),
      );
      return hue * (0.35 + speed);
    }
  }
}

@compute @workgroup_size(256)
fn cs_main(
  @builtin(global_invocation_id) gid: vec3<u32>,
  @builtin(num_workgroups) groups: vec3<u32>,
) {
  let i = gid.y * groups.x * 256u + gid.x;
  if (i >= N) { return; }
  let species = i % SPECIES;
  let size = vec2<f32>(f32(params.columns), f32(ROWS));

  var particle = particles[i];
  let pos = particle.pos * size;
  let here = sample(pos);
  let follow = params.attract[species] + COHESION;
  let slope = vec2<f32>(dot(follow, here.slope_x), dot(follow, here.slope_y));
  var accel = FORCE * (slope - PRESSURE * here.crowd.xy);

  if (params.pull != 0.0) {
    var toward = (params.mouse - particle.pos) * size;
    toward -= round(toward / size) * size; // the short way around the wrapped world
    let distance = length(toward);
    accel += params.pull * toward / max(distance, 1.0) * exp(-distance * distance / (REACH * REACH));
  }

  accel += (random(i, params.frame) - 0.5) * params.heat;

  particle.vel = particle.vel * exp(-FRICTION * DT) + accel * DT;
  let speed = length(particle.vel);
  if (speed > TOP_SPEED) { particle.vel *= TOP_SPEED / speed; }
  particle.pos = fract((pos + particle.vel * DT) / size);
  particles[i] = particle;

  // Count into one of the four nearest cells, picked with bilinear odds, so crowds stay round.
  let home = floor(particle.pos * size - 0.5 + random(params.frame, i));
  atomicAdd(&counts[index(i32(home.x), i32(home.y)) * SPECIES + species], 1u);

  let pixel = min(vec2<u32>(particle.pos * vec2<f32>(params.image)), params.image - 1u);
  let light = vec3<u32>(shade(species, min(speed / HOT, 1.0), here.crowd.z) * 255.0);
  let slot = (pixel.y * params.image.x + pixel.x) * 3u;
  atomicAdd(&image[slot], light.r);
  atomicAdd(&image[slot + 1u], light.g);
  atomicAdd(&image[slot + 2u], light.b);
}
"#;

const PRESENT: &str = r#"
@group(0) @binding(0) var<uniform> image_size: vec2<u32>;
@group(0) @binding(1) var<storage, read> image: array<u32, PIXELS * 3u>;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var corners = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  var uv = array<vec2<f32>, 3>(vec2<f32>(0.0, 1.0), vec2<f32>(2.0, 1.0), vec2<f32>(0.0, -1.0));
  return VOut(vec4<f32>(corners[i], 0.0, 1.0), uv[i]);
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  let pixel = min(vec2<u32>(v.uv * vec2<f32>(image_size)), image_size - 1u);
  let slot = (pixel.y * image_size.x + pixel.x) * 3u;
  let light = vec3<f32>(f32(image[slot]), f32(image[slot + 1u]), f32(image[slot + 2u])) / 255.0;
  // Expose for the mean particles per pixel, so any count looks alike.
  let exposure = 1.5 / (1.0 + f32(N) / f32(image_size.x * image_size.y));
  // Tone-map the brightest channel and scale the others with it, so crowds keep their hue.
  let peak = max(max(light.r, light.g), max(light.b, 1e-6));
  return vec4<f32>(light * (1.0 - exp(-peak * exposure)) / peak, 1.0);
}
"#;

/// The shaders and passes the window callbacks reach.
struct Life {
    field: Shader,
    clear: Pass,
    update: Shader,
    present: Shader,
    passes: Vec<Pass>,
    frame: AtomicU32,
}

impl Life {
    fn new(count: u32) -> Result<Self, ShaderError> {
        let shared = format!(
            "const N: u32 = {count}u;\nconst ROWS: u32 = {ROWS}u;\n\
             const COLUMNS: u32 = {COLUMNS}u;\nconst PIXELS: u32 = {PIXELS}u;\n{SHARED}"
        );
        let field = Shader::new([shared.as_str(), FIELD])?;
        let clear = Shader::new([shared.as_str(), CLEAR])?;
        // Slugs pull the color ramps and the hash from the shader library.
        let update = Shader::new([
            "gradient/palette_iq",
            "gradient/turbo",
            "hash/pcg2d",
            shared.as_str(),
            UPDATE,
        ])?;
        let present = Shader::new([shared.as_str(), PRESENT])?;

        update.set("particles", bytemuck::cast_slice(&scatter(count)))?;
        update.set("params.attract", shuffle())?;
        update.set("params.heat", HEAT)?;

        let field_pass = Pass::from_shader("field", &field);
        field_pass.set_compute_dispatch(COLUMNS / 8, ROWS / 8, 1);
        let clear = Pass::from_shader("clear", &clear);
        let update_pass = Pass::from_shader("update", &update);
        let groups = count.div_ceil(256);
        update_pass.set_compute_dispatch(groups.min(65535), groups.div_ceil(65535), 1);
        let present_pass = Pass::from_shader("present", &present);

        Ok(Life {
            passes: vec![field_pass, clear.clone(), update_pass, present_pass],
            field,
            clear,
            update,
            present,
            frame: AtomicU32::new(0),
        })
    }

    /// Fits the grid and the image to the window, so nothing stretches.
    fn resize(&self, size: &PhysicalSize<u32>) -> Result<(), ShaderError> {
        let width = size.width.max(1);
        let height = size.height.max(1);
        let columns = (ROWS * width / height).clamp(16, COLUMNS);
        let scale = (1..)
            .find(|k| (width / k) * (height / k) <= PIXELS)
            .unwrap_or(1);
        let image = [(width / scale).max(1), (height / scale).max(1)];

        self.field.set("columns", columns)?;
        self.update.set("params.columns", columns)?;
        self.update.set("params.image", image)?;
        self.present.set("image_size", image)?;
        let threads = (image[0] * image[1]).max(ROWS * COLUMNS);
        self.clear.set_compute_dispatch(threads.div_ceil(256), 1, 1);
        Ok(())
    }

    fn step(&self) -> Result<(), ShaderError> {
        let frame = self.frame.fetch_add(1, Ordering::Relaxed);
        self.update.set("params.frame", frame)
    }

    fn cycle_colors(&self) -> Result<(), ShaderError> {
        let mode: u32 = self.update.get("params.mode")?;
        self.update.set("params.mode", (mode + 1) % 3)
    }

    fn warm(&self, factor: f32) -> Result<(), ShaderError> {
        let heat: f32 = self.update.get("params.heat")?;
        self.update
            .set("params.heat", (heat * factor).clamp(1.0, 2000.0))
    }
}

/// Random positions across the window and small random velocities, laid out as `Particle`.
fn scatter(count: u32) -> Vec<f32> {
    (0..count)
        .flat_map(|_| {
            let speed = || (fastrand::f32() - 0.5) * 4.0;
            [fastrand::f32(), fastrand::f32(), speed(), speed()]
        })
        .collect()
}

/// A random attraction table: row `a` says how species `a` follows each species.
fn shuffle() -> [[f32; 4]; 4] {
    std::array::from_fn(|_| std::array::from_fn(|_| fastrand::f32() * 2.0 - 1.0))
}

/// Prints an error from a window callback, which has no caller to return it to.
fn report(result: Result<(), ShaderError>) {
    if let Err(error) = result {
        eprintln!("particles: {error}");
    }
}

fn on_resize(app: &App, size: &PhysicalSize<u32>) {
    if let Err(error) = app.resize([size.width, size.height]) {
        eprintln!("resize failed: {error}");
    }
    if let Some(life) = app.get_state::<Life>("life") {
        report(life.resize(size));
    }
}

fn on_cursor_moved(app: &App, _device: DeviceId, position: &PhysicalPosition<f64>) {
    let window = app.window_size(app.primary_window_id());
    if let (Some(life), Some(size)) = (app.get_state::<Life>("life"), window) {
        let x = (position.x / size.width.max(1) as f64) as f32;
        let y = (position.y / size.height.max(1) as f64) as f32;
        report(life.update.set("params.mouse", [x, y]));
    }
}

fn on_mouse_input(app: &App, _device: DeviceId, state: ElementState, button: MouseButton) {
    let pull = match (state, button) {
        (ElementState::Pressed, MouseButton::Left) => PULL,
        (ElementState::Pressed, MouseButton::Right) => -PULL,
        _ => 0.0,
    };
    if let Some(life) = app.get_state::<Life>("life") {
        report(life.update.set("params.pull", pull));
    }
}

fn on_keyboard_input(app: &App, _device: DeviceId, event: &KeyEvent, _synthetic: bool) {
    let Some(life) = app.get_state::<Life>("life") else {
        return;
    };
    if event.state != ElementState::Pressed {
        return;
    }
    report(match event.physical_key {
        PhysicalKey::Code(KeyCode::KeyR) => life.update.set("params.attract", shuffle()),
        PhysicalKey::Code(KeyCode::KeyC) => life.cycle_colors(),
        PhysicalKey::Code(KeyCode::ArrowUp) => life.warm(1.5),
        PhysicalKey::Code(KeyCode::ArrowDown) => life.warm(1.0 / 1.5),
        _ => Ok(()),
    });
}

fn on_redraw(app: &App) {
    let Some(life) = app.get_state::<Life>("life") else {
        return;
    };
    report(life.step());
    let renderer = app.get_renderer();
    let rendered = app.with_target(app.primary_window_id(), |target| {
        renderer.render(&life.passes, target)
    });
    if let Some(Err(error)) = rendered {
        eprintln!("particles: {error}");
    }
}

// Creates a render target for each window and fits the simulation to it.
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    for window in windows {
        let target = app.get_renderer().create_target(window.clone()).await?;
        app.add_target(window.id(), target);
        if let Some(life) = app.get_state::<Life>("life") {
            life.resize(&window.inner_size())?;
        }
    }
    Ok(())
}

fn main() -> Result<(), ShaderError> {
    let _ = env_logger::try_init();
    let count = std::env::var("PARTICLES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1_000_000);

    let mut app = App::new(Renderer::new());
    app.add_state("life", Life::new(count)?);
    app.on_start(call!(setup))
        .on_resize(on_resize)
        .on_cursor_moved(on_cursor_moved)
        .on_mouse_input(on_mouse_input)
        .on_keyboard_input(on_keyboard_input)
        .on_redraw_requested(on_redraw);
    app.run();
    Ok(())
}
