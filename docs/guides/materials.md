# Guide: Materials (draw the same shape with different textures)

Let's treat a Shader like a "material." We'll use the same WGSL program twice, 
but with different textures, and draw both in a single pass.

Why this is nice

- Mesh stays pure geometry
- Shader holds the texture (and other parameters)
- The renderer can reuse the same pipeline while swapping textures — it feels like magic

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Pass, Target, Mesh, Quad};

// 1) Renderer + offscreen target
let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

// 2) Two tiny 2×2 textures in RGBA8
let pix_a: Vec<u8> = vec![
    255, 0,   0, 255,   0, 255,   0, 255,
      0, 0, 255, 255, 255, 255, 255, 255,
];
let tex_a = renderer.create_texture_with_size(&pix_a, [2u32, 2u32]).await?;

let pix_b: Vec<u8> = vec![
      0,   0,   0, 255, 255, 255,   0, 255,
    255,   0, 255, 255,   0, 255, 255, 255,
];
let tex_b = renderer.create_texture_with_size(&pix_b, [2u32, 2u32]).await?;

// 3) One WGSL program (shared) + two Shader instances (“materials”)
let wgsl = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> VOut {
  var out: VOut; out.pos = vec4<f32>(pos, 0.0, 1.0); out.uv = uv; return out;
}
@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  return textureSample(tex, samp, v.uv);
}
"#;

let left_shader  = Shader::new(wgsl)?;
let right_shader = Shader::new(wgsl)?;
left_shader.set("tex", &tex_a)?;
right_shader.set("tex", &tex_b)?;

// 4) One pass, two shaders
let pass = Pass::from_shader("materials", &left_shader);
pass.add_shader(&right_shader);

// 5) Two quads, each attached to a different Shader
let left:  Mesh = Quad::new([-0.9, -0.5], [-0.1, 0.5]).into();
let right: Mesh = Quad::new([ 0.1, -0.5], [ 0.9, 0.5]).into();

left_shader.add_mesh(&left)?;
right_shader.add_mesh(&right)?;

// 6) Render
renderer.render(&pass, &target)?;
assert_eq!(target.size().width, 64);
assert_eq!(target.size().height, 64);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

What's happening under the hood

- Both Shader instances share the same WGSL, so the renderer can reuse the pipeline.
- Each Shader binds a different texture, so you get two textured draws in one pass.

Pitfalls / gotchas

- Remember to declare both `texture_2d<f32>` and `sampler` for sampling.
- set("tex", &Texture) wires the texture automatically (a default sampler is provided if needed).

Next steps

- Continue to instancing-basics to paint a small swarm with per-instance offsets and tints (coming next).
