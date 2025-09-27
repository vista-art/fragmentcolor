use fragmentcolor::{App, Frame, Pass, Shader, TextureFormat, run};

fn handle_cursor_moved(
    app: &App,
    _dev: winit::event::DeviceId,
    pos: &winit::dpi::PhysicalPosition<f64>,
) {
    // Convert from window pixels to clip coords [-1,1] with origin at center, Y up
    let id = app.window_id();
    if let Some(sz) = app.size(id) {
        let w = sz.width as f64;
        let h = sz.height as f64;
        if w > 0.0 && h > 0.0 {
            let cx = (pos.x / w) * 2.0 - 1.0;
            let cy = -((pos.y / h) * 2.0 - 1.0);
            let _ = app.set_uniform(id, "sim.cx", cx as f32);
            let _ = app.set_uniform(id, "sim.cy", cy as f32);
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
};

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>, N>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<f32>, N>;
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
  let r2 = dx*dx + dy*dy + eps;
  let r  = sqrt(r2);
  let inv_r3 = 1.0 / (r2 * r);
  let ax = -g * dx * inv_r3;
  let ay = -g * dy * inv_r3;

  // Update velocity then integrate position (semi-implicit Euler)
  v.x = (v.x + ax * dt) * damp;
  v.y = (v.y + ay * dt) * damp;

  p.x = p.x + v.x * dt;
  p.y = p.y + v.y * dt;

  p.x = clamp(p.x, -2.0, 2.0);
  p.y = clamp(p.y, -2.0, 2.0);

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

fn make_splat_wgsl(n: u32) -> String {
    let tpl = r#"
const N: u32 = __N__u;

@group(0) @binding(0) var img: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> res: vec2<u32>;
@group(0) @binding(2) var<storage, read> positions: array<vec2<f32>, N>;
@group(0) @binding(3) var<storage, read> colors: array<vec4<f32>, N>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i >= N) { return; }
  let p = positions[i];
  // map clip-space [-1,1] to UV [0,1]
  let uv = vec2<f32>(p.x * 0.5 + 0.5, p.y * 0.5 + 0.5);
  let px = i32(uv.x * f32(res.x));
  let py = i32(uv.y * f32(res.y));
  if (px < 0 || py < 0 || px >= i32(res.x) || py >= i32(res.y)) { return; }
  let c = colors[i];
  // last-writer-wins (no read/modify; write-only storage texture)
  textureStore(img, vec2<i32>(px, py), c);
}
"#;
    tpl.replace("__N__", &n.to_string())
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

fn main() {
    let _ = env_logger::try_init();
    let n: u32 = std::env::var("PARTICLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);

    // Shaders
    let cs_update = Shader::new(&make_update_wgsl(n)).expect("update shader");
    let cs_clear = Shader::new(CLEAR_WGSL).expect("clear shader");
    let cs_splat = Shader::new(&make_splat_wgsl(n)).expect("splat shader");
    let fs_present = Shader::new(FS_PRESENT_WGSL).expect("present shader");

    // Storage buffers: positions/velocities/colors
    let n_usize = n as usize;
    let mut pos = vec![0f32; n_usize * 2];
    let mut vel = vec![0f32; n_usize * 2];
    let mut col = vec![0f32; n_usize * 4];
    for i in 0..n_usize {
        let x = fastrand::f32() * 2.0 - 1.0;
        let y = fastrand::f32() * 2.0 - 1.0;
        pos[2 * i] = x;
        pos[2 * i + 1] = y;
        let vx = (fastrand::f32() * 2.0 - 1.0) * 0.15;
        let vy = (fastrand::f32() * 2.0 - 1.0) * 0.15;
        vel[2 * i] = vx;
        vel[2 * i + 1] = vy;
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
    // Do not set positions on the splat shader; it shares the persistent GPU buffer via registry
    cs_splat
        .set("colors", bytemuck::cast_slice(&col))
        .expect("bind colors");

    // Storage texture for splat
    let mut app = App::new();
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

    // Drive
    app.on_resize(on_resize).scene(frame);

    run(&mut app);
}
