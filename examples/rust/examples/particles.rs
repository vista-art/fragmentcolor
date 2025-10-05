use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{App, Frame, Pass, Renderer, SetupResult, Shader, call, run};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// Particles (winit) — CPU-simulated gravity toward the center
// - Default: 10_000 particles (set PARTICLES=1000000 to try 1M, see notes below)
// - Each particle is a tiny triangle instance with per-instance offset (position) and color
// - We update positions on the CPU each frame and rebuild the instance buffer
//
// Notes on scaling to 1M particles efficiently are at the end of this file.

const SHADER_SRC: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Vertex attributes
// - @location(0) pos: base mesh vertex (vec2)
// - @location(1) offset: per-instance position in clip space (vec2)
// - @location(2) tint: per-instance color (vec4)
@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) offset: vec2<f32>,
    @location(2) tint: vec4<f32>,
) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(pos + offset, 0.0, 1.0);
    out.color = tint;
    return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    return v.color;
}
"#;

#[derive(Clone, Copy)]
struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    col: [f32; 4],
}

struct ParticlesState {
    mesh: Mesh,
    particles: Vec<Particle>,
    dt: f32,
    g: f32,
    damp: f32,
}

pub fn on_resize(app: &App, sz: &PhysicalSize<u32>) {
    app.resize([sz.width, sz.height]);
}

fn draw(app: &App) {
    use std::sync::{Mutex, OnceLock};
    static STATE: OnceLock<Mutex<ParticlesState>> = OnceLock::new();

    if let Some(state_lock) = STATE.get() {
        let mut state = state_lock.lock().unwrap();
        // integrate
        let dt = state.dt;
        let g = state.g;
        let damp = state.damp;
        for p in &mut state.particles {
            let x = p.pos[0];
            let y = p.pos[1];
            let r2 = x * x + y * y + 1e-4;
            let r = r2.sqrt();
            let ax = -g * x / (r2 * r);
            let ay = -g * y / (r2 * r);
            p.vel[0] = (p.vel[0] + ax * dt) * damp;
            p.vel[1] = (p.vel[1] + ay * dt) * damp;
            p.pos[0] = (p.pos[0] + p.vel[0] * dt).clamp(-2.0, 2.0);
            p.pos[1] = (p.pos[1] + p.vel[1] * dt).clamp(-2.0, 2.0);
        }
        // rebuild instances
        let mut tmp = Vec::with_capacity(state.particles.len());
        for p in &state.particles {
            tmp.push(
                Vertex::new([0.0, 0.0])
                    .set("offset", p.pos)
                    .set("tint", p.col),
            );
        }
        state.mesh.clear_instances();
        state.mesh.add_instances(tmp);
    }

    let id = app.primary_window_id();
    if let Some(pass) = app.get::<Pass>("pass.particles") {
        let mut frame = Frame::new();
        frame.add_pass(&pass);
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&frame, t));
    }
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    use std::sync::{Mutex, OnceLock};
    static STATE: OnceLock<Mutex<ParticlesState>> = OnceLock::new();

    let n: usize = std::env::var("PARTICLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);

    let shader = Shader::new(SHADER_SRC)?;
    let pass = Pass::from_shader("particles", &shader);

    // Tiny triangle in NDC centered at origin (instanced across the screen)
    let mesh = Mesh::new();
    let s = 0.004; // triangle size edge ~0.4% of screen
    mesh.add_vertices([[-s, -s], [s, -s], [0.0, s]]);

    // Build initial particle set and matching instances
    let mut parts = Vec::with_capacity(n);
    let mut insts = Vec::with_capacity(n);
    for _ in 0..n {
        let px = fastrand::f32() * 2.0 - 1.0; // [-1, 1]
        let py = fastrand::f32() * 2.0 - 1.0;
        let vx = (fastrand::f32() * 2.0 - 1.0) * 0.25; // initial speed scaled
        let vy = (fastrand::f32() * 2.0 - 1.0) * 0.25;
        let r = fastrand::f32();
        let g = fastrand::f32();
        let b = fastrand::f32();
        let col = [r, g, b, 1.0];
        parts.push(Particle {
            pos: [px, py],
            vel: [vx, vy],
            col,
        });
        insts.push(
            Vertex::new([0.0, 0.0])
                .set("offset", [px, py])
                .set("tint", col),
        );
    }
    mesh.add_instances(insts);

    pass.add_mesh(&mesh)?;

    // Store pass and state
    app.add("pass.particles", pass.clone());

    STATE.get_or_init(|| {
        Mutex::new(ParticlesState {
            mesh,
            particles: parts,
            dt: 1.0 / 60.0,
            g: 0.5,
            damp: 0.995,
        })
    });

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }

    Ok(())
}

fn main() {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);

    app.on_start(call!(setup))
        .on_redraw_requested(draw)
        .on_resize(on_resize)
        .on_close_requested(|_| {});

    run(&mut app);
}

// --------------------------------------------------------------------------------------------
// Scaling notes for 1,000,000 particles
// --------------------------------------------------------------------------------------------
// The example above rebuilds per-instance data each frame using a HashMap-backed API
// (Vertex/Instance store properties in HashMap). That’s ergonomic and fine for 10k,
// but becomes CPU-heavy at 1M due to:
//   - HashMap lookups during packing
//   - Allocating and serializing N instances into a fresh Vec<u8> every frame
//   - Uploading a large COPY_DST vertex buffer each frame
//
// With the current API you can still push toward 1M by:
//   - Reducing per-instance stride (e.g., only position, move color into a palette
//     or compute color in fragment from position/velocity)
//   - Lowering the update rate (subsample simulation or rebuild instances every N frames)
//   - Favoring simpler motion (compute position analytically from time without re-upload)
//
// However, for truly large N, two API adaptations make this scale much better:
// 1) Allow uniforms/storage buffers to be visible from the VERTEX stage.
//    Today, bindings are created with FRAGMENT visibility. If we set visibility to
//    VERTEX|FRAGMENT (or infer actual usage), the vertex shader can read a storage buffer
//    like: `@group(0) @binding(0) var<storage, read> particles: Particles;`
//    and index into it using `@builtin(instance_index)`. Then:
//      - Instance buffer can be tiny (just to set instance_count) or even contain only an `id:u32`
//      - Per-frame updates become a single contiguous CPU write into a storage blob
//      - No per-instance HashMap packing needed
//
// 2) Provide a fast-path to upload instance data as raw bytes.
//    Something like `mesh.set_instances_bytes(&[u8], stride)` (or a typed SoA writer) would skip
//    per-element property maps and pack directly. That makes CPU-side generation for 1M feasible.
//
// Longer term, adding a compute pass (already scaffolded in the codebase) would enable fully GPU-
// resident simulation (update storage in compute, render in graphics), eliminating CPU updates.
