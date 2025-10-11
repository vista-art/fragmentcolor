use fragmentcolor::{App, Frame, Pass, SetupResult, Shader, TextureFormat, call, run};
use std::sync::Arc;
use winit::window::Window;

fn handle_cursor_moved(
    app: &App,
    _dev: winit::event::DeviceId,
    pos: &winit::dpi::PhysicalPosition<f64>,
) {
    // Convert from window pixels to clip coords [-1,1] with origin at center, Y up
    let id = app.primary_window_id();
    if let Some(sz) = app.window_size(id) {
        let w = sz.width as f64;
        let h = sz.height as f64;
        if w > 0.0 && h > 0.0 {
            let cx = (pos.x / w) * 2.0 - 1.0;
            let cy = -((pos.y / h) * 2.0 - 1.0);
            // Make Z coordinate vary based on X position for more dynamic 3D effect
            let cz = cx.sin() as f32 * 0.5; // Oscillate Z between -0.5 and 0.5 based on X
            if let Some(update) = app.get::<Shader>("shader.update") {
                let _ = update.set("sim.cx", cx as f32);
                let _ = update.set("sim.cy", cy as f32);
                let _ = update.set("sim.cz", cz);
            }
        }
    }
}

// Compute-driven particle simulation (splat to storage texture)
// Pipeline per-frame:
// 1) compute: update positions/velocities in storage buffers
// 2) compute: clear storage texture
// 3) compute: splat particles into storage texture (last-writer-wins)
// 4) render: fullscreen sample of the storage texture

fn make_update_wgsl(n: u32) -> String {
    let tpl = r#"
const N: u32 = __N__u;

struct Sim {
  step: f32,  // dt seconds
  grav: f32,  // gravity strength
  damp: f32,  // velocity damping
  mode: f32,  // debug flag (unused by default)
  cx: f32,    // gravity center x in clip coords [-1,1]
  cy: f32,    // gravity center y in clip coords [-1,1]
  cz: f32,    // gravity center z in clip coords [-1,1]
};

@group(0) @binding(0) var<storage, read_write> positions: array<vec3<f32>, N>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec3<f32>, N>;
@group(0) @binding(2) var<storage, read> sim: Sim;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i >= N) { return; }
  let dt = sim.step;
  let g  = sim.grav;
  let damp = sim.damp;

  var p = positions[i];
  var v = velocities[i];

  let eps = 1e-4;
  let dx = p.x - sim.cx;
  let dy = p.y - sim.cy;
  let dz = p.z - sim.cz;
  let r2 = dx*dx + dy*dy + dz*dz + eps;
  let r  = sqrt(r2);
  let inv_r3 = 1.0 / (r2 * r);
  let ax = -g * dx * inv_r3;
  let ay = -g * dy * inv_r3;
  let az = -g * dz * inv_r3;

  // Update velocity then integrate position (semi-implicit Euler)
  v.x = (v.x + ax * dt) * damp;
  v.y = (v.y + ay * dt) * damp;
  v.z = (v.z + az * dt) * damp;

  p.x = p.x + v.x * dt;
  p.y = p.y + v.y * dt;
  p.z = p.z + v.z * dt;

  p.x = clamp(p.x, -2.0, 2.0);
  p.y = clamp(p.y, -2.0, 2.0);
  p.z = clamp(p.z, -2.0, 2.0);

  positions[i] = p;
  velocities[i] = v;
}
"#;
    tpl.replace("__N__", &n.to_string())
}

const CLEAR_WGSL: &str = r#"
@group(0) @binding(0) var img: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> res: vec2<u32>;

@compute @workgroup_size(16,16,1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  if (gid.x >= res.x || gid.y >= res.y) { return; }
  textureStore(img, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}
"#;

fn make_splat_wgsl(n: u32, size: u32, min_size: f32, max_size: f32) -> String {
    let tpl = r#"
const N: u32 = __N__u;
const SIZE: u32 = __SIZE__u;
const MIN_SIZE: f32 = __MIN_SIZE__;
const MAX_SIZE: f32 = __MAX_SIZE__;

@group(0) @binding(0) var img: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> res: vec2<u32>;
@group(0) @binding(2) var<storage, read> positions: array<vec3<f32>, N>;
@group(0) @binding(3) var<storage, read> colors: array<vec4<f32>, N>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i >= N) { return; }
  let p = positions[i];
  
  // Simple perspective projection: use Z coordinate to scale particle size
  // Map Z from [-2, 2] to a scaling factor [MIN_SIZE, MAX_SIZE]
  // Closer particles (positive Z) are larger, farther particles (negative Z) are smaller
  let z_normalized = (p.z + 2.0) / 4.0; // Map [-2,2] to [0,1]
  let size_scale = MIN_SIZE + z_normalized * (MAX_SIZE - MIN_SIZE); // Map [0,1] to [MIN_SIZE,MAX_SIZE]
  let actual_size = u32(f32(SIZE) * size_scale);
  
  // Project 3D position to 2D screen coordinates (simple orthographic projection)
  // Use only X and Y coordinates for screen position
  let uv = vec2<f32>(p.x * 0.5 + 0.5, 1.0 - (p.y * 0.5 + 0.5));
  let px = i32(uv.x * f32(res.x));
  let py = i32(uv.y * f32(res.y));
  let c = colors[i];
  
  // Render each particle as a size-scaled square
  for (var dx = 0u; dx < actual_size; dx++) {
    for (var dy = 0u; dy < actual_size; dy++) {
      let x = px + i32(dx) - i32(actual_size / 2u);
      let y = py + i32(dy) - i32(actual_size / 2u);
      if (x >= 0 && y >= 0 && x < i32(res.x) && y < i32(res.y)) {
        textureStore(img, vec2<i32>(x, y), c);
      }
    }
  }
}
"#;
    tpl.replace("__N__", &n.to_string())
        .replace("__SIZE__", &size.to_string())
        .replace("__MIN_SIZE__", &min_size.to_string())
        .replace("__MAX_SIZE__", &max_size.to_string())
}

const FS_PRESENT_WGSL: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
  var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
  var out: VOut;
  out.pos = vec4<f32>(p[i], 0., 1.);
  out.uv = uv[i];
  return out;
}
@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  return textureSample(tex, samp, v.uv);
}
"#;

pub fn on_resize(app: &App, sz: &winit::dpi::PhysicalSize<u32>) {
    app.resize([sz.width, sz.height]);
}

// Create a render target for each window on startup
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }
    Ok(())
}

fn main() {
    let _ = env_logger::try_init();
    let n: u32 = std::env::var("PARTICLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);

    let particle_size: u32 = std::env::var("PARTICLE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let min_size: f32 = std::env::var("MIN_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.1);

    let max_size: f32 = std::env::var("MAX_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4.0);

    // Shaders
    let cs_update = Shader::new(&make_update_wgsl(n)).expect("update shader");
    let cs_clear = Shader::new(CLEAR_WGSL).expect("clear shader");
    let cs_splat =
        Shader::new(&make_splat_wgsl(n, particle_size, min_size, max_size)).expect("splat shader");
    let fs_present = Shader::new(FS_PRESENT_WGSL).expect("present shader");

    // Storage buffers: positions/velocities/colors (now 3D)
    let particles = n as usize;
    let mut pos = vec![0f32; particles * 3]; // Changed to 3D
    let mut vel = vec![0f32; particles * 3]; // Changed to 3D
    let mut col = vec![0f32; particles * 4];
    for i in 0..particles {
        let x = fastrand::f32() * 2.0 - 1.0;
        let y = fastrand::f32() * 2.0 - 1.0;
        let z = fastrand::f32() * 2.0 - 1.0; // Added Z coordinate
        pos[3 * i] = x;
        pos[3 * i + 1] = y;
        pos[3 * i + 2] = z; // Store Z coordinate
        let vx = (fastrand::f32() * 2.0 - 1.0) * 0.15;
        let vy = (fastrand::f32() * 2.0 - 1.0) * 0.15;
        let vz = (fastrand::f32() * 2.0 - 1.0) * 0.2; // Increased Z velocity range for more dynamic movement
        vel[3 * i] = vx;
        vel[3 * i + 1] = vy;
        vel[3 * i + 2] = vz; // Store Z velocity
        let r = fastrand::f32();
        let g = fastrand::f32();
        let b = fastrand::f32();
        col[4 * i] = r;
        col[4 * i + 1] = g;
        col[4 * i + 2] = b;
        col[4 * i + 3] = 1.0;
    }

    cs_update
        .set("positions", bytemuck::cast_slice(&pos))
        .expect("set pos");
    cs_update
        .set("velocities", bytemuck::cast_slice(&vel))
        .expect("set vel");
    cs_update.set("sim.step", 1.0 / 60.0).expect("sim.step");
    cs_update.set("sim.grav", 0.35).expect("sim.grav");
    cs_update.set("sim.damp", 0.9975).expect("sim.damp");
    cs_update.set("sim.mode", 0.0).expect("sim.mode");
    cs_update.set("sim.cx", 0.0f32).expect("sim.cx");
    cs_update.set("sim.cy", 0.0f32).expect("sim.cy");
    cs_update.set("sim.cz", 0.0f32).expect("sim.cz");
    // Do not set positions on the splat shader; it shares the persistent GPU buffer via registry
    cs_splat
        .set("colors", bytemuck::cast_slice(&col))
        .expect("bind colors");

    // Storage texture for splat
    let renderer = fragmentcolor::Renderer::new();
    let mut app = App::new(renderer);
    let tex_size = [1024u32, 1024u32];
    let tex = pollster::block_on(async {
        app.renderer()
            .create_storage_texture(tex_size, TextureFormat::Rgba8Unorm, None)
            .await
            .expect("create splat texture")
    });

    // Bind storage texture and resolutions
    cs_clear.set("img", &tex).expect("bind img clear");
    cs_clear
        .set("res", [tex_size[0], tex_size[1]])
        .expect("set res clear");

    cs_splat.set("img", &tex).expect("bind img splat");
    cs_splat
        .set("res", [tex_size[0], tex_size[1]])
        .expect("set res splat");

    fs_present.set("tex", &tex).expect("bind present tex");

    // Make update shader available to cursor callback
    app.add("shader.update", cs_update.clone());

    // Passes
    let pass_update = Pass::from_shader("update", &cs_update);
    let wx = n.div_ceil(256).max(1);
    pass_update.set_compute_dispatch(wx, 1, 1);

    let pass_clear = Pass::from_shader("clear", &cs_clear);
    let cx = tex_size[0].div_ceil(16);
    let cy = tex_size[1].div_ceil(16);
    pass_clear.set_compute_dispatch(cx, cy, 1);

    let pass_splat = Pass::from_shader("splat", &cs_splat);
    pass_splat.set_compute_dispatch(wx, 1, 1);

    let pass_present = Pass::from_shader("present", &fs_present);

    // Frame
    let mut frame = Frame::new();
    frame.add_pass(&pass_update);
    frame.add_pass(&pass_clear);
    frame.add_pass(&pass_splat);
    frame.add_pass(&pass_present);

    // Mouse tracking and per-frame center update
    app.on_cursor_moved(handle_cursor_moved);

    // Store frame
    app.add("frame.main", frame);

    // Drive explicit rendering in draw
    app.on_start(call!(setup))
        .on_resize(on_resize)
        .on_redraw_requested(|app| {
            let id = app.primary_window_id();
            if let Some(frame) = app.get::<Frame>("frame.main") {
                let r = app.get_renderer();
                let _ = app.with_target(id, |t| r.render(&*frame, t));
            }
        });

    run(&mut app);
}
