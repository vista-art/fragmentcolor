# Material::custom

Wrap an arbitrary `Shader` as a `Material` so it can be paired with a
`Mesh` to form a `Model`. The PBR-named setters (`base_color`, `metallic`,
…) still work — they call `shader.set(...)` under matching uniform paths
(`material.base_color`, etc.). If your shader doesn't declare those
uniforms the setters log-warn and return self unchanged; set custom uniforms
directly via `material.shader().set(...)`.

`Material::custom` is the escape hatch for any shading model that isn't
glTF-spec PBR — toon shading, flat unlit colors, wireframe debug views,
volumetric or post-process effects that you want to attach to a `Mesh`.

## What custom shaders should declare for full Material-style ergonomics

Optional but recommended:

- `struct PbrMaterial { base_color: vec4<f32>, … }` at `@group(1) @binding(1)`
  bound as `material` — lets the PBR factor setters work as-is.
- `struct MeshTransform { model: mat4x4<f32>, … }` at `@group(1) @binding(0)`
  bound as `mesh` — lets `Model::transform` / `translate` / `rotate` /
  `scale` work as-is.

Without these the Material is still functional; only the high-level setters
become no-ops.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Shader};

let wireframe = Shader::new(r#"
    struct MeshTransform { model: mat4x4<f32> }
    struct Camera { view_proj: mat4x4<f32>, position: vec3<f32> }
    @group(0) @binding(0) var<uniform> camera: Camera;
    @group(1) @binding(0) var<uniform> mesh: MeshTransform;

    @vertex
    fn vs_main(@location(0) p: vec3<f32>) -> @builtin(position) vec4<f32> {
        return camera.view_proj * mesh.model * vec4<f32>(p, 1.0);
    }
    @fragment fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(0.0, 1.0, 0.4, 1.0);
    }
"#)?;

let wire_mat = Material::custom(wireframe);
# let _ = wire_mat;
# Ok(())
# }
```
