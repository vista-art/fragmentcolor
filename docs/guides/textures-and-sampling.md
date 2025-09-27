# Guide: Textures and sampling (a tiny 2×2 image)

Now let’s introduce textures. We’ll upload a tiny 2×2 RGBA image and sample it in the fragment shader.

What we’ll build
- A 64×64 offscreen target
- A 2×2 RGBA texture
- A shader that samples it with a default sampler

Why this matters
- Textures are the backbone of materials. Starting small keeps everything portable and fast.



```rust
# // Hidden harness so this runs as a doctest and transpiles to other languages

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Pass, Target};

// 1) Renderer + offscreen target
let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

// 2) Create a tiny 2×2 RGBA texture: red, green, blue, white
#[rustfmt::skip]
let pixels: Vec<u8> = vec![
    255, 0,   0, 255,   0, 255,   0, 255,
      0, 0, 255, 255, 255, 255, 255, 255,
];
let tex = renderer.create_texture_with_size(&pixels, [2u32, 2u32]).await?;

// 3) WGSL that samples the texture (library provides a default sampler)
let wgsl = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p  = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
  var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.),   vec2<f32>(2.,1.),  vec2<f32>(0.,-1.));
  var out: VOut; out.pos = vec4<f32>(p[i], 0., 1.); out.uv = uv[i]; return out;
}
@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return textureSample(tex, samp, v.uv); }
"#;

let shader = Shader::new(wgsl)?;
shader.set("tex", &tex)?;
let rpass = Pass::from_shader("tex_sample", &shader);

renderer.render(&rpass, &target)?;

// Prove execution (RGBA8, tightly packed)
let image = target.get_image();
assert_eq!(image.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

What’s happening under the hood
- The texture is uploaded to the GPU and bound for sampling.
- If you don’t set a sampler, FragmentColor provides a sensible default.
- set("tex", &Texture) wires up the view and sampler for you.

Pitfalls / gotchas
- Declare both `texture_2d<f32>` and `sampler` in the same group to use `textureSample`.
- For nearest vs smooth filtering or wrap vs clamp, see Texture::set_sampler_options.

Next steps
- Continue to materials.md to draw the same geometry with different textures in one pass.
