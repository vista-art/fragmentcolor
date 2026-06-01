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

## Example

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let normal_map = renderer.create_texture(&[
    128, 128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
    128,   128, 255, 255,
][..]).await?;
let mat = Material::pbr()?.normal_texture(&normal_map).normal_scale(1.2);
# let _ = mat;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
