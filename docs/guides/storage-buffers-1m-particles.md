# Guide: Storage buffers — one million particles

Let's render a big crowd with minimal CPU work. We'll keep geometry tiny (a triangle)
and drive positions/colors from a storage buffer, indexing by the instance index.
The same technique scales to millions of instances.

Why this matters

- Move instance data out of the mesh and into a storage buffer
- Draw many copies with mesh.set_instance_count(N)
- Update data by replacing a single byte blob: shader.set("particles", &bytes)

```rust
# // Hidden harness so this runs as a doctest and transpiles to other languages

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Pass, Target};
use fragmentcolor::mesh::{Mesh, Vertex};

// Visible target count for the guide (stress test): one million
let n: u32 = 1_000_000;
# // CI safety override: allow smaller N via env or default to 100k
# let n: u32 = std::env::var("FC_PARTICLES").ok()
#     .and_then(|s| s.parse().ok())
#     .unwrap_or(100_000);

// 1) Renderer + offscreen target
let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

// 2) WGSL: storage-driven rendering. No per-instance attributes; we index by instance_index.
let wgsl = r#"
const N: u32 = 1000000; // sized storage array (bump for bigger stress tests)

struct Particle { pos: vec2<f32>, col: vec4<f32> };
struct Particles { data: array<Particle, N> };

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

let shader = Shader::new(wgsl)?;

// 3) Provide a storage blob. Each Particle packs 24 bytes: pos(vec2)=8 + col(vec4)=16.
const STRIDE: usize = 24;
let bytes_len = (STRIDE as u64 * 1_000_000u64) as usize; // full span for the shader’s N
let mut blob = vec![0u8; bytes_len];
// Optional: seed a few visible points near center in red/green/blue (kept minimal for speed)
{
    let mut write_particle = |idx: usize, x: f32, y: f32, c: [f32; 4]| {
        let b = &mut blob[idx * STRIDE..idx * STRIDE + STRIDE];
        b[0..4].copy_from_slice(&x.to_le_bytes());
        b[4..8].copy_from_slice(&y.to_le_bytes());
        b[8..12].copy_from_slice(&c[0].to_le_bytes());
        b[12..16].copy_from_slice(&c[1].to_le_bytes());
        b[16..20].copy_from_slice(&c[2].to_le_bytes());
        b[20..24].copy_from_slice(&c[3].to_le_bytes());
    };
    write_particle(0,  0.00,  0.00, [1.0, 0.2, 0.2, 1.0]);
    write_particle(1, -0.35, -0.20, [0.2, 1.0, 0.2, 1.0]);
    write_particle(2,  0.40,  0.25, [0.2, 0.2, 1.0, 1.0]);
}
shader.set("particles", &blob[..])?; // upload once; draw can reuse

// 4) Base mesh: one tiny triangle; we’ll draw it n times via instance_index
let mut mesh = Mesh::new();
mesh.add_vertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.02],
]);
mesh.set_instance_count(n);

let pass = Pass::from_shader("particles", &shader);
pass.add_mesh(&mesh)?;
renderer.render(&pass, &target)?;

// 5) Quick check
let image = target.get_image();
assert_eq!(image.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

What's happening under the hood

- The storage root "particles" becomes a GPU buffer. We upload one big blob of bytes once.
- The vertex shader fetches a Particle by instance_index and offsets a tiny triangle.
- mesh.set_instance_count(n) draws n copies with no per-instance attributes.

Pitfalls / gotchas

- Keep your struct layout stable: vec2 (8 bytes) + vec4 (16 bytes) = 24 bytes per particle.
- The WGSL array length is fixed at compile time. For interactive stress tests, set FC_PARTICLES to override the runtime instance count.

Next steps

- Continue to compute-driven-sim to update positions on the GPU each frame using a compute pass.
