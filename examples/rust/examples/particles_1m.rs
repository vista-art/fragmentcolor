use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{App, Pass, Shader, run};

// One million particles via storage buffer + instance_index
// - Uses a storage buffer (var<storage, read>) visible to the vertex stage
// - No per-instance attributes; we draw N instances with mesh.set_instance_count(N)
// - Positions/colors live in a packed byte blob updated via shader.set("particles", &bytes)

const N: usize = 1_000_000;
const STRIDE: usize = 24; // pos: vec2<f32> (8) + col: vec4<f32> (16)

const SHADER_SRC: &str = r#"
struct Particle { pos: vec2<f32>, col: vec4<f32> };
struct Particles { data: array<Particle, 1000000> };

@group(0) @binding(0) var<storage, read> particles: Particles;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) col: vec4<f32> };

@vertex
fn vs_main(@location(0) base: vec2<f32>, @builtin(instance_index) i: u32) -> VOut {
  let p = particles.data[i];
  var out: VOut;
  out.pos = vec4<f32>(base + p.pos, 0.0, 1.0);
  out.col = p.col;
  return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.col; }
"#;

fn pack_particle(buf: &mut [u8], x: f32, y: f32, col: [f32; 4]) {
    // layout: [pos.xy][col.rgba]
    buf[0..4].copy_from_slice(&x.to_ne_bytes());
    buf[4..8].copy_from_slice(&y.to_ne_bytes());
    buf[8..12].copy_from_slice(&col[0].to_ne_bytes());
    buf[12..16].copy_from_slice(&col[1].to_ne_bytes());
    buf[16..20].copy_from_slice(&col[2].to_ne_bytes());
    buf[20..24].copy_from_slice(&col[3].to_ne_bytes());
}

pub fn on_resize(app: &App, sz: &winit::dpi::PhysicalSize<u32>) {
    app.resize([sz.width, sz.height]);
}

fn main() {
    // Shader + pass
    let shader = Shader::new(SHADER_SRC).expect("shader");
    let pass = Pass::from_shader("particles_1m", &shader);

    // Base mesh: tiny triangle
    let mesh = Mesh::new();
    let s = 0.0035f32;
    mesh.add_vertices([
        Vertex::new([-s, -s]),
        Vertex::new([s, -s]),
        Vertex::new([0.0, s]),
    ]);
    // Draw N instances with no per-instance attributes
    mesh.set_instance_count(N as u32);
    pass.add_mesh(&mesh).expect("mesh is compatible");

    // CPU buffers: positions + velocities in SoA for updates; blob for upload
    let mut pos_x: Vec<f32> = Vec::with_capacity(N);
    let mut pos_y: Vec<f32> = Vec::with_capacity(N);
    let mut vel_x: Vec<f32> = Vec::with_capacity(N);
    let mut vel_y: Vec<f32> = Vec::with_capacity(N);
    let mut col: Vec<[f32; 4]> = Vec::with_capacity(N);

    for _ in 0..N {
        let x = fastrand::f32() * 2.0 - 1.0;
        let y = fastrand::f32() * 2.0 - 1.0;
        let vx = (fastrand::f32() * 2.0 - 1.0) * 0.15;
        let vy = (fastrand::f32() * 2.0 - 1.0) * 0.15;
        let c = [fastrand::f32(), fastrand::f32(), fastrand::f32(), 1.0];
        pos_x.push(x);
        pos_y.push(y);
        vel_x.push(vx);
        vel_y.push(vy);
        col.push(c);
    }

    let mut blob = vec![0u8; N * STRIDE];
    for i in 0..N {
        let base = i * STRIDE;
        pack_particle(&mut blob[base..base + STRIDE], pos_x[i], pos_y[i], col[i]);
    }
    // Upload initial data
    shader.set("particles", &blob[..]).expect("set particles");

    // App
    let mut app = App::new();
    app.on_resize(on_resize);

    // Simple gravity toward center with damping
    let dt = 1.0 / 60.0;
    let g = 0.35f32;
    let damp = 0.9975f32;

    app.scene(pass.clone()).on_redraw_requested(move |_app| {
        // Update CPU state
        for i in 0..N {
            let x = pos_x[i];
            let y = pos_y[i];
            let r2 = x * x + y * y + 1e-4;
            let r = r2.sqrt();
            let ax = -g * x / (r2 * r);
            let ay = -g * y / (r2 * r);
            vel_x[i] = (vel_x[i] + ax * dt) * damp;
            vel_y[i] = (vel_y[i] + ay * dt) * damp;
            pos_x[i] = (x + vel_x[i] * dt).clamp(-2.0, 2.0);
            pos_y[i] = (y + vel_y[i] * dt).clamp(-2.0, 2.0);
        }
        // Repack updated positions (colors unchanged)
        for i in 0..N {
            let base = i * STRIDE;
            pack_particle(&mut blob[base..base + STRIDE], pos_x[i], pos_y[i], col[i]);
        }
        // Upload
        let _ = shader.set("particles", &blob[..]);
    });

    run(&mut app);
}
