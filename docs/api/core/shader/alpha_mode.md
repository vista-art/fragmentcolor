# Shader::alpha_mode

Set how the renderer treats the alpha channel this shader writes. It takes an `AlphaMode`
and returns the shader, so calls can chain.

- `AlphaMode::Opaque` (the default): blending is off and depth writes are on.
- `AlphaMode::Mask`: the pipeline is the opaque one. Your fragment shader does the cut-out
  itself, with `discard`.
- `AlphaMode::Blend`: depth writes turn off and the color target over-blends with
  `SrcAlpha / OneMinusSrcAlpha`. The renderer draws blended Models on a Pass back to front
  from the Pass camera, after every opaque draw.

Use it on a Shader you pass to `Model::new` as the material, and on any Shader you draw on
its own. A [PbrMaterial](https://fragmentcolor.org/api/scene/pbrmaterial) sets the same
state for its built-in shader through
[PbrMaterial::alpha_mode](https://fragmentcolor.org/api/scene/pbrmaterial#alpha_mode).

Blending needs a color target that blends and a shader that writes a `vec4`. A format that
never blends, such as an integer one, and an output without alpha return
`PassError::InvalidColorTarget`.

This is pipeline state. The renderer keeps one pipeline per shader, alpha mode and
face setting, so a change builds a pipeline once and reuses it from then on.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{AlphaMode, Shader};

let shader = Shader::new(r#"
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.6, 0.8, 1.0, 0.25);
}
"#)?;

let glass = shader.alpha_mode(AlphaMode::Blend);
# let _ = glass;
# Ok(())
# }
```
