# Material::custom

Wrap an arbitrary `Shader` as a `Material` so it can be paired with a
`Mesh` to form a `Model`. The PBR-named setters (`base_color`, `metallic`,
…) still work; they call `shader.set(...)` under matching uniform paths
(`material.base_color`, etc.). If your shader doesn't declare those
uniforms the setters log-warn and return self unchanged; set custom uniforms
directly via `material.shader().set(...)`.

`Material::custom` is the escape hatch for any shading model that isn't
glTF-spec PBR: toon shading, flat unlit colors, wireframe debug views,
volumetric or post-process effects that you want to attach to a `Mesh`.

## What custom shaders should declare for full Material-style ergonomics

Optional but recommended:

- `struct PbrMaterial { base_color: vec4<f32>, … }` at `@group(1) @binding(0)`
  bound as `material`. This lets the PBR factor setters work as-is.
- Four `vec4<f32>` vertex inputs named `model_0` to `model_3`, the columns of
  the Model's transform. The renderer fills them at draw time from each
  Model's `set_transform`, `translate`, `rotate` and `scale`. The PBR shader
  reads them at locations 3 to 6. Any free locations work, since the renderer
  matches these inputs by name. It fills them only for the Models a pass
  holds, through each Model's Material. A shader that reads them for a Mesh
  with no Model behind it in the pass, such as one attached with `add_mesh`,
  makes `Renderer::render` return `MeshError::InvalidInstance`.

Without these the Material still draws. The factor setters do nothing, and
every Model draws at its Mesh's own coordinates, whatever its transform.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Shader};

let wireframe = Shader::new(r#"
    struct Camera { view_proj: mat4x4<f32>, position: vec3<f32> }
    @group(0) @binding(0) var<uniform> camera: Camera;

    @vertex
    fn vs_main(
        @location(0) p: vec3<f32>,
        @location(3) model_0: vec4<f32>,
        @location(4) model_1: vec4<f32>,
        @location(5) model_2: vec4<f32>,
        @location(6) model_3: vec4<f32>,
    ) -> @builtin(position) vec4<f32> {
        let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
        return camera.view_proj * model * vec4<f32>(p, 1.0);
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
