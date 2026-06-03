# Material::occlusion_strength

Blends the per-fragment ambient-occlusion factor (read from an occlusion
map) toward `1.0`, matching glTF 2.0's `occlusionTextureInfo.strength`.
`1.0` uses the map's value as-is; `0.0` ignores the map entirely.

Maps to the `material.occlusion_strength` uniform. Default is `1.0`. The
default PBR shader samples `occlusion_map.r` and `mix(1.0, sampled, strength)`s
the result into the diffuse term.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let crevices = Material::pbr().occlusion_strength(0.8);
# let _crevices = crevices;
# Ok(())
# }
```
