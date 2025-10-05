# Guide: Compute‑driven simulation (update on the GPU)

Let's move the particle update to run entirely in the GPU!

A compute pass will nudge positions in place (simple gravity),
and then a render pass will draw them. No per‑frame CPU uploads.

Why this matters

- Keep large buffers on the GPU and update them in place
- Separate concerns: one compute pass, one render pass, composed in a Frame

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Pass, Frame, Target};
use fragmentcolor::mesh::{Mesh, Vertex};

let n: u32 = 1_000_000;
# let n: u32 = std::env::var("FC_PARTICLES").ok()
#     .and_then(|s| s.parse().ok())
#     .unwrap_or(100_000);

// 1) Renderer + offscreen target
let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

// 2) Compute shader (updates positions/velocities in place)
let cs_src = r#"
const N: u32 = 1000000; // sized arrays (tweak for local stress tests)

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>, N>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec2<f32>, N>;
@group(0) @binding(2) var<uniform> sim: vec4<f32>; // dt, g, damp, _

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if (i >= N) { return; }

  let dt = sim.x; let g = sim.y; let damp = sim.z;

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

  // soft clamp
  p.x = clamp(p.x, -2.0, 2.0);
  p.y = clamp(p.y, -2.0, 2.0);

  positions[i] = p;
  velocities[i] = v;
}
"#;

// 3) Render shader (reads positions + colors)
let fs_src = r#"
const N: u32 = 1000000;
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

let cs = Shader::new(cs_src)?;
let fs = Shader::new(fs_src)?;

// 4) Create initial storage blobs (full spans to match WGSL N)
let pos_stride = 8usize; // vec2<f32>
let vel_stride = 8usize; // vec2<f32>
let col_stride = 16usize; // vec4<f32>
let n_full = 1_000_000usize; // matches WGSL N
let mut pos = vec![0u8; pos_stride * n_full];
let mut vel = vec![0u8; vel_stride * n_full];
let mut col = vec![0u8; col_stride * n_full];

// seed a few visible particles and colors near the center
let mut write_f32 = |dst: &mut [u8], idx: usize, val: f32| {
    dst[idx..idx+4].copy_from_slice(&val.to_le_bytes());
};
let seed = |i: usize, x: f32, y: f32, color: [f32;4], pos: &mut [u8], col: &mut [u8]| {
    let p = i * pos_stride; // positions
    write_f32(pos, p+0, x); write_f32(pos, p+4, y);
    let c = i * col_stride; // colors
    write_f32(col, c+0, color[0]); write_f32(col, c+4, color[1]);
    write_f32(col, c+8, color[2]); write_f32(col, c+12, color[3]);
};
seed(0,  0.00,  0.00, [1.0, 0.2, 0.2, 1.0], &mut pos, &mut col);
seed(1, -0.35, -0.20, [0.2, 1.0, 0.2, 1.0], &mut pos, &mut col);
seed(2,  0.40,  0.25, [0.2, 0.2, 1.0, 1.0], &mut pos, &mut col);

// Upload initial data
cs.set("positions", &pos[..])?;
cs.set("velocities", &vel[..])?;
cs.set("sim", [1.0f32 / 60.0, 0.35, 0.9975, 0.0])?; // dt, g, damp
fs.set("colors", &col[..])?;

// 5) Passes: compute then render
let pass_cs = Pass::from_shader("update", &cs);
let wx = n.div_ceil(256).max(1);
pass_cs.set_compute_dispatch(wx, 1, 1);

let pass_fs = Pass::from_shader("render", &fs);

// 6) Geometry: tiny triangle, drawn n instances via instance_index
let mut mesh = Mesh::new();
mesh.add_vertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.02],
]);
mesh.set_instance_count(n);
pass_fs.add_mesh(&mesh)?;

// 7) Compose passes in a Frame and render
let mut frame = Frame::new();
frame.add_pass(&pass_cs);
frame.add_pass(&pass_fs);
renderer.render(&frame, &target)?;

// 8) Quick check
let image = target.get_image();
assert_eq!(image.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

What's happening under the hood

- The compute pass updates positions/velocities in persistent storage buffers (kept by the renderer).
- The render pass reads positions + colors and draws n instances.
- A Frame collects both passes and runs them in order.

Pitfalls / gotchas

- WGSL array sizes are fixed at compile time; we allocate full‑span buffers once.
- Use set_compute_dispatch to cover your active instance count (n).

Next steps

- Try multipass-with-shadows to layer passes creatively (e.g., a simple cast‑shadow pass followed by the main pass).
