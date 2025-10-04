# Guide: Instancing basics (small swarm)

Let's draw the same tiny triangle many times, each with its own offset and tint.
This introduces per‑instance attributes, a simple way to place many copies of the same geometry.

## Why this matters

- One mesh, many instances: efficient and readable
- Instance attributes let you move and color copies independently

```rust
# // Hidden harness so this runs as a doctest and transpiles to other languages

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Pass, Target};
use fragmentcolor::mesh::{Mesh, Vertex};

// 1) Renderer + offscreen target
let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

// 2) WGSL with per-instance offset (@location(1)) and tint (@location(2))
let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) col: vec4<f32> };

@vertex
fn vs_main(
  @location(0) pos: vec3<f32>,
  @location(1) offset: vec2<f32>,
  @location(2) tint: vec4<f32>,
) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos.xy + offset, pos.z, 1.0);
  out.col = tint;
  return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.col; }
"#;

let shader = Shader::new(wgsl)?;
let pass = Pass::from_shader("instancing", &shader);

// 3) One tiny triangle (shared by all instances)
let mut mesh = Mesh::new();
mesh.add_vertices([
    [-0.05, -0.05, 0.0],
    [ 0.05, -0.05, 0.0],
    [ 0.00,  0.08, 0.0],
]);

// 4) Two instances with different offsets and tints
mesh.add_instances([
    Vertex::new([-0.25, 0.0])
        .set("offset", [-0.25f32, 0.0])
        .set("tint",   [1.0f32, 0.2, 0.2, 1.0]), // reddish
    Vertex::new([ 0.25, 0.0])
        .set("offset", [ 0.25f32, 0.0])
        .set("tint",   [0.2f32, 0.2, 1.0, 1.0]), // bluish
]);

// 5) Attach mesh to this shader and render
pass.add_mesh(&mesh)?; // validates locations/types match the shader
renderer.render(&pass, &target)?;

// 6) Quick check
assert_eq!(target.size().width, 64);
assert_eq!(target.size().height, 64);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## What's happening under the hood

- The mesh provides one set of vertices (the triangle) plus a list of instances.
- The shader declares which attributes it needs at each @location; FragmentColor validates and binds them.
- At draw time, the GPU runs the vertex stage once per instance and composes pos + offset.

## Pitfalls / gotchas

- Match your attribute names and types: here we used "offset" (vec2) and "tint" (vec4).
- Locations must be unique; if you later add UVs or normals, pick different indices.

## Next steps

- Move on to uniforms-and-push-constants to parameterize your shader with simple values.
