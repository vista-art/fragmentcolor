use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{App, Frame, Pass, Renderer, SetupResult, Shader, run};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// Compute-driven particle simulation (no per-frame CPU uploads)
// - positions/velocities/colors live in storage buffers
// - compute pass updates velocities/positions in-place
// - render pass draws instanced tiny triangles, fetching data by instance_index
// - N is configurable via env PARTICLES (default 1_000_000)

fn make_compute_wgsl(n: u32) -> String {
    let tpl = r#"
const N: u32 = __N__u;

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>, N>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<f32>, N>;
@group(0) @binding(2) var<uniform> sim: vec4<f32>; // dt, g, damp, _

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i >= N) { return; }
  let dt = sim.x;
  let g  = sim.y;
  let damp = sim.z;

  var p = positions[i];
  var v = velocities[i];

  let eps = 1e-4;
  let r2 = p.x*p.x + p.y*p.y + eps;
  let r  = sqrt(r2);
  let inv_r3 = 1.0 / (r2 * r);
  let ax = -g * p.x * inv_r3;
  let ay = -g * p.y * inv_r3;

  v.x = (v.x + ax * dt) * damp;
  v.y = (v.y + ay * dt) * damp;

  p.x = p.x + v.x * dt;
  p.y = p.y + v.y * dt;

  // soft clamp to avoid drifting too far (clip space-ish)
  p.x = clamp(p.x, -2.0, 2.0);
  p.y = clamp(p.y, -2.0, 2.0);

  positions[i] = p;
  velocities[i] = v;
}
"#;
    tpl.replace("__N__", &n.to_string())
}

fn make_render_wgsl(n: u32) -> String {
    let tpl = r#"
const N: u32 = __N__u;

@group(0) @binding(0) var<storage, read> positions: array<vec2<f32>, N>;
@group(0) @binding(1) var<storage, read> colors: array<vec4<f32>, N>;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) col: vec4<f32> };

@vertex
fn vs_main(@location(0) base: vec2<f32>, @builtin(instance_index) i: u32) -> VOut {
  let p = positions[i];
  let c = colors[i];
  var out: VOut;
  out.pos = vec4<f32>(base + p, 0.0, 1.0);
  out.col = c;
  return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.col; }
"#;
    tpl.replace("__N__", &n.to_string())
}

pub fn on_resize(app: &App, sz: &PhysicalSize<u32>) {
    app.resize([sz.width, sz.height]);
}

fn draw(app: &App) {
    // Optional: tweak sim constants per-frame; keep constant for stability
    if let Some(cs) = app.get::<Shader>("shader.compute") {
        let _ = cs.set("sim", [1.0 / 60.0, 0.35, 0.9975, 0.0]);
    }

    let id = app.primary_window_id();
    if let Some(frame) = app.get::<Frame>("frame.main") {
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*frame, t));
    }
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let n: u32 = std::env::var("PARTICLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);

    // Shaders
    let cs_src = make_compute_wgsl(n);
    let fs_src = make_render_wgsl(n);

    let cs = Shader::new(&cs_src)?;
    let fs = Shader::new(&fs_src)?;

    // Prepare initial blobs (std430: vec2 stride 8, vec4 stride 16)
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

    // Upload initial storage to compute (positions/velocities) and to render (colors)
    cs.set("positions", bytemuck::cast_slice(&pos))
        .expect("set pos");
    cs.set("velocities", bytemuck::cast_slice(&vel))
        .expect("set vel");
    cs.set("sim", [1.0 / 60.0, 0.35, 0.9975, 0.0])
        .expect("set sim");

    fs.set("colors", bytemuck::cast_slice(&col))
        .expect("set colors");

    // Passes
    let pass_cs = Pass::from_shader("compute", &cs);
    let wx = n.div_ceil(256).max(1);
    pass_cs.set_compute_dispatch(wx, 1, 1);

    let pass_fs = Pass::from_shader("render", &fs);

    // Geometry: tiny triangle at origin; instanced N times
    let mesh = Mesh::new();
    let s = 0.0035f32;
    mesh.add_vertices([
        Vertex::new([-s, -s]),
        Vertex::new([s, -s]),
        Vertex::new([0.0, s]),
    ]);
    mesh.set_instance_count(n);
    pass_fs.add_mesh(&mesh).expect("mesh is compatible");

    // Frame: compute then render
    let mut frame = Frame::new();
    frame.add_pass(&pass_cs);
    frame.add_pass(&pass_fs);

    app.add("shader.compute", cs);
    app.add("frame.main", frame);

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }

    Ok(())
}

fn main() {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);
    app.on_resize(on_resize)
        .on_start(setup)
        .on_redraw_requested(draw);
    run(&mut app);
}
