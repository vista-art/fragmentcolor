# Material::occlusion_strength

Blends the per-fragment ambient-occlusion factor (read from an occlusion
map) toward `1.0`, matching glTF 2.0's `occlusionTextureInfo.strength`.
`1.0` uses the map's value as-is; `0.0` ignores the map entirely.

Maps to the `material.occlusion_strength` uniform. Default is `1.0`. The
default PBR shader samples `occlusion_map.r` and `mix(1.0, sampled, strength)`s
the result into the diffuse term.

NaN or an infinity returns `MaterialError::NotFinite`, which names the
call, and the material keeps the value it had. The other languages also
refuse a number past the 32-bit float range, such as `1e39`, with a
message like `Material.occlusionStrength: value must be a finite number
that fits a 32-bit float, got 1e39`. JavaScript throws an `Error` named
`InvalidInputError`, Python raises `InvalidInputError`, Swift throws
`FragmentColorError` and Kotlin throws `FragmentColorException`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let crevices = Material::pbr().occlusion_strength(0.8)?;
# let _crevices = crevices;
# Ok(())
# }
```
