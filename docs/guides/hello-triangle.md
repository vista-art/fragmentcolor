# Guide: Hello Triangle (first pixels)

Let's draw our very first pixels. We'll render a fullscreen triangle into a tiny offscreen image. No window needed — perfect for tests and CI.

What we'll build

- A Renderer and a 64×64 offscreen Target
- A tiny shader that paints a color
- One Pass that draws once

Why this matters

- It shows the core flow: Shader → Pass → render(Target)
- It introduces the Shader, the heart of FragmentColor — set values by name and it “just works”

```rust
# // Hidden harness so this runs as a doctest and transpiles to other languages

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Pass, Target};

// 1) Renderer + offscreen target (headless and CI‑safe)
let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

// 2) A tiny WGSL program: a fullscreen triangle + a color uniform
let wgsl = r#"
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  // 3 vertices: (-1,-1), (3,-1), (-1,3) cover the whole screen
  let p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
  return vec4<f32>(p[i], 0., 1.);
}

@group(0) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> { return color; }
"#;

// 3) Shader is the star: set uniforms by name (magic!)
let shader = Shader::new(wgsl)?;
shader.set("color", [1.0, 0.2, 0.8, 1.0])?;

// 4) One pass, one draw
let rpass = Pass::from_shader("hello", &shader);
renderer.render(&rpass, &target)?;

// 5) Quick check to prove it ran
assert_eq!(target.size().width, 64);
assert_eq!(target.size().height, 64);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

What's happening under the hood

- The Renderer sets up GPU state when first used.
- Shader parses your WGSL and exposes variables as simple set("name", value) calls.
- A Pass groups shaders. Rendering the pass records commands and produces an image.

Pitfalls / gotchas

- Make sure your shader has both a vertex and fragment entry point.
- Colors are premultiplied alpha by default.
- Offscreen targets are RGBA8; get_image() returns tightly packed RGBA pixels.

Next steps

- Continue to Textures and Sampling to bind an image and sample it in the fragment stage: textures-and-sampling.md
