# Texture

A GPU texture resource. Public API wrapper that holds a shareable reference to the internal TextureObject and a numeric handle used by uniforms.

- Construct via [Renderer](https://fragmentcolor.org/api/core/renderer)::create_texture_* helpers (no direct constructors)
- Set on shaders with shader.set("key", &[Texture](https://fragmentcolor.org/api/core/texture))
- [Texture](https://fragmentcolor.org/api/core/texture) owns its sampler; you can tweak filtering and wrapping via set_sampler_options.

## How to use

How to use now (Rust)
â¢  Create a [Texture](https://fragmentcolor.org/api/core/texture) and set it on a [Shader](https://fragmentcolor.org/api/core/shader)

```rust,no-run
let renderer = Renderer::new();
let bytes = std::fs::read("image.png")?;
let tex = pollster::block_on(renderer.create_texture(&bytes))?;
shader.set("t_tex", &tex)?; // WGSL expects texture_2d at binding(0), sampler at binding(1)
```

â¢  In your WGSL, declare you sampler in the same group as your texture

```wgsl
// must be in the same group
@group(0) @binding(0) var t_tex: texture_2d<f32>;
@group(0) @binding(1) var t_smp: sampler;
```

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader};
let renderer = Renderer::new();
let shader = Shader::default();

let bytes = std::fs::read("./examples/assets/image.png").unwrap();
let texture = renderer.create_texture(&bytes).await?;

shader.set("texture", &texture).unwrap();

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
