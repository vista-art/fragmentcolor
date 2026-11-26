# Shader::double_sided

Choose whether the renderer draws the back faces of this shader's triangles. It takes a
boolean and returns the shader, so calls can chain.

A Shader draws both faces by default (`true`), which suits a fullscreen pass and flat
shapes whose winding nobody checked. Pass `false` to cull back faces, with
counter-clockwise winding as the front, as a closed mesh wants: the culled faces cost
no fragment work.

Use it on a Shader you pass to `Model::new` as the material, and on any Shader you draw on
its own. A [PbrMaterial](https://fragmentcolor.org/api/scene/pbrmaterial) culls by default,
and sets the same state for its built-in shader through
[PbrMaterial::double_sided](https://fragmentcolor.org/api/scene/pbrmaterial#double_sided).

This is pipeline state. The renderer keeps one pipeline per shader, alpha mode and
face setting, so a change builds a pipeline once and reuses it from then on.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Shader;

let shader = Shader::new(r#"
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.5, 0.2, 1.0);
}
"#)?;

let solid = shader.double_sided(false);
# let _ = solid;
# Ok(())
# }
```
