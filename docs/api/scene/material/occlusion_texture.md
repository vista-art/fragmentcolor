# Material::occlusion_texture

Bind an ambient-occlusion map to the canonical `occlusion_map` slot.
Following glTF 2.0, only the red channel is read; the sampled value is
blended toward `1.0` by `1 - material.occlusion_strength` inside the
default PBR shader, then multiplied into the diffuse term.

Unset, this slot resolves to a 1×1 white default so the multiplied
occlusion factor is `1.0` and no darkening is applied.

Takes the inputs [`Material::base_color_texture`](https://fragmentcolor.org/api/scene/material#base_color_texture) takes, spelled as each language's `create_texture` spells them: a `Texture`, or encoded bytes, raw pixels with their size, bytes with a format or with every option, a path or a URL, alone or with a format or every option, or a `Mipmap`, alone or with options. The Material keeps a source as a map, and each `Renderer` uploads a copy of its own when it loads the scene. `base_color_texture` tells how a map loads, what it holds in memory and how it fails.

The setter refuses options beside a `Texture`, and an input that holds no
data, with `MaterialError::Texture`. JavaScript throws it as an `Error`
named `InvalidInputError`, and Python raises `InvalidInputError`. A
`Material::custom` shader that declares `occlusion_map` as something other
than a texture refuses the call with `MaterialError::Shader`, which
JavaScript and Python raise as `MaterialError`. Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`. A refused
call leaves the slot as it was, and a shader that doesn't declare
`occlusion_map` skips the call with a log warning.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

// Raw pixels with their size upload when the scene loads.
let ao_bytes: Vec<u8> = vec![
    220, 0, 0, 255,
    180, 0, 0, 255,
    200, 0, 0, 255,
    160, 0, 0, 255,
];
let mat = Material::pbr().occlusion_texture((ao_bytes, [2, 2]))?;
# let _ = mat;
# Ok(())
# }
```
