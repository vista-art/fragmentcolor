# TextureSlot

Names one of the five texture slots of a PBR `Material`, the maps glTF 2.0
defines for metallic-roughness materials.

Each slot and the setter that binds its texture:

- `BaseColor`, which `Material::base_color_texture` binds
- `MetallicRoughness`, which `Material::metallic_roughness_texture` binds
- `Normal`, which `Material::normal_texture` binds
- `Occlusion`, which `Material::occlusion_texture` binds
- `Emissive`, which `Material::emissive_texture` binds

Each slot samples its own UV set through its own transform.
[`Material::texture_uv`](https://fragmentcolor.org/api/scene/material#texture_uv)
picks the set, and
[`Material::texture_transform`](https://fragmentcolor.org/api/scene/material#texture_transform)
sets the transform. A glTF file loaded with `Scene::load` sets both from
each texture's `texCoord` and `KHR_texture_transform`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, TextureSlot};

let material = Material::pbr()
    .texture_uv(TextureSlot::Occlusion, 1)?
    .texture_transform(TextureSlot::Normal, [0.0, 0.0], [4.0, 4.0], 0.0);
# let _ = material;
# Ok(())
# }
```
