# Shader::double_sided

Choose whether the renderer draws the back faces of this shader's triangles. It takes a
boolean and returns the shader, so calls can chain.

A Shader culls back faces by default (`false`), as every material does and as
glTF's default is. A triangle whose vertices run counter-clockwise on screen faces
the viewer and draws, and one that runs clockwise faces away and costs no fragment
work. `Shader::default()` and the fullscreen triangles in the examples wind
counter-clockwise, so culling leaves them whole.

Pass `true` to draw both faces, for a surface seen from both sides: a leaf or grass
card, a flag, cloth, paper, an open mesh seen from inside, or a flat shape whose
vertices run clockwise.

Use it on a Shader you pass to `Model::new` as the material, and on any Shader you draw on
its own. The materials set the same state for their built-in shaders through
[PbrMaterial::double_sided](https://fragmentcolor.org/api/scene/pbrmaterial#double_sided)
and its siblings on `UnlitMaterial` and `ToonMaterial`.

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

let leaf = shader.double_sided(true);
# let _ = leaf;
# Ok(())
# }
```
