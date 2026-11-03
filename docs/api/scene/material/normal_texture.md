# Material::normal_texture

Bind a tangent-space normal map to the canonical `normal_map` slot. The
default PBR shader samples the map in `fs_main`, decodes the stored
`[0, 1]` bytes into `[-1, 1]` floats, scales the XY components by
`material.normal_scale`, and rotates the perturbed normal from tangent
space into world space using the per-vertex tangent (the glTF `tangent.w`
handedness flag is preserved).

Unset, this slot resolves to a 1×1 flat tangent-space default
`(128, 128, 255, 255)` so the decoded normal is `(0, 0, 1)` and the world
normal passes through unchanged.

Takes the inputs [`Material::base_color_texture`](https://fragmentcolor.org/api/scene/material#base_color_texture) takes, spelled as each language's `create_texture` spells them: a `Texture`, or encoded bytes, raw pixels with their size, bytes with a format or with every option, a path or a URL, alone or with a format or every option, or a `Mipmap`, alone or with options. The Material keeps a source as a map, and each `Renderer` uploads a copy of its own when it loads the scene. `base_color_texture` tells how a map loads, what it holds in memory and how it fails.

The setter refuses options beside a `Texture`, and an input that holds no
data, with `MaterialError::Texture`. JavaScript throws it as an `Error`
named `InvalidInputError`, and Python raises `InvalidInputError`. A
`Material::custom` shader that declares `normal_map` as something other than
a texture refuses the call with `MaterialError::Shader`, which JavaScript
and Python raise as `MaterialError`. Swift throws `FragmentColorError` and
Kotlin throws `FragmentColorException`. A refused call leaves the slot as it
was, and a shader that doesn't declare `normal_map` skips the call with a
log warning.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

// Raw pixels with their size upload when the scene loads.
let normal_map_bytes: Vec<u8> = vec![
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
    128, 128, 255, 255,
];
let mat = Material::pbr()
    .normal_texture((normal_map_bytes, [2, 2]))?
    .normal_scale(1.2)?;
# let _ = mat;
# Ok(())
# }
```
