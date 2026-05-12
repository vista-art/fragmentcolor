# Shader::duplicate

Build a fresh [Shader](https://fragmentcolor.org/api/core/shader) that compiles
the same WGSL source as this one but starts with its own uniform-storage and
mesh-attachment state. Calling `set` on the duplicate does not affect the
original, and vice versa.

The underlying GPU pipeline is shared (wgpu deduplicates by module hash), so
the cost is one uniform buffer's worth of allocation per duplicate — not a
full pipeline recompile.

This is what [Model::new](https://fragmentcolor.org/api/scene/model) calls
internally so each model gets its own `mesh.model` uniform slot when many
models share a single [Material](https://fragmentcolor.org/api/scene/material).
Reach for it directly when you're building a similar per-instance state
abstraction for a non-PBR shading model.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Shader;

let template = Shader::new(r#"
    struct Tint { color: vec4<f32> }
    @group(0) @binding(0) var<uniform> tint: Tint;
    @vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
        let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
        return vec4<f32>(p[i], 0.0, 1.0);
    }
    @fragment fn fs() -> @location(0) vec4<f32> { return tint.color; }
"#)?;
template.set("tint.color", [1.0, 0.0, 0.0, 1.0])?;

// Independent copy — sets on `red` do not bleed into `template` or `blue`.
let red = template.duplicate()?;
let blue = template.duplicate()?;
blue.set("tint.color", [0.0, 0.4, 1.0, 1.0])?;

# let _: [f32; 4] = red.get("tint.color")?;
# let _: [f32; 4] = blue.get("tint.color")?;
# Ok(())
# }
```
