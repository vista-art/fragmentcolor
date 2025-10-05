# Guide: Uniforms and push constants (simple parameters)

Let’s give our shader a couple of knobs. We’ll use a uniform for position (u.offset) and a push constant for color (pc.color). On desktop, push constants are used natively; on the web, FragmentColor rewrites them into uniforms automatically — you don’t need to change your code.

Why this matters
- You can parameterize your shader with simple values
- set("path", value) feels natural and works across platforms



```rust
# // Hidden harness so this runs as a doctest and transpiles to other languages

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Pass, Target};

// 1) Renderer + offscreen target
let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

// 2) WGSL with a uniform (u.offset) and a push constant (pc.color)
let wgsl = r#"
struct U { offset: vec2<f32> };
@group(0) @binding(0) var<uniform> u: U;

struct PC { color: vec4<f32> };
var<push_constant> pc: PC;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  // fullscreen triangle, slightly shifted by u.offset
  let p = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0)
  );
  return vec4<f32>(p[i] + u.offset, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
  return pc.color; // set from code via "pc.color"
}
"#;

let shader = Shader::new(wgsl)?;

// 3) Set parameters by name: uniforms and push constants
shader.set("u.offset", [0.05f32, 0.02])?;        // small shift
shader.set("pc.color", [0.2f32, 0.8, 0.2, 1.0])?; // green-ish

let pass = Pass::from_shader("params", &shader);
renderer.render(&pass, &target)?;

// 4) Quick check
let image = target.get_image();
assert_eq!(image.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

What’s happening under the hood
- u.offset uploads as a uniform; pc.color uses native push constants on desktop.
- On the web, FragmentColor rewrites push constants to uniforms seamlessly — same set("pc.*") calls work.

Pitfalls / gotchas
- Keep values in the expected shapes: vec2 for offset, vec4 for color in this example.
- If you mix uniforms and push constants, structure names (u.*, pc.*) keep things tidy.

Next steps
- Try storage-buffers-1m-particles to move instance data into a storage buffer and draw a big crowd efficiently.
