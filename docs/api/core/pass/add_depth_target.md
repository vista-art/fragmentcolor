# Pass::add_depth_target

Select a depth attachment for this pass.

The target must be a stable texture created by the same Renderer with usage including `RENDER_ATTACHMENT` and a depth/stencil format (e.g., `Depth32Float`).

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Pass, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

// Create a depth texture usable as a per-pass attachment
let depth = renderer.create_depth_texture([64u32, 64u32]).await?;

// Simple scene shader with @location(0) position
let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VOut { var o: VOut; o.pos = vec4f(pos,1.0); return o; }
@fragment
fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4f(0.7,0.8,1.0,1.0); }
"#;
let shader = Shader::new(wgsl)?;
let pass = Pass::from_shader("scene", &shader);

// Attach depth texture to enable depth testing
pass.add_depth_target(&depth)?;

// Render as usual
renderer.render(&pass, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
