# Material::normal_scale

Scales the per-fragment normal perturbation read from a tangent-space normal
map, mirroring glTF 2.0's `normalTextureInfo.scale`. Higher values exaggerate
the bumps; values below 1 soften them.

Maps to the `material.normal_scale` uniform. Default is `1.0`. The
factors-only built-in PBR shader does not yet sample a normal map, so this
value is stored but not exercised — it becomes effective once normal-map
sampling lands or under a `Material::custom` shader that reads the uniform.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let detailed = Material::pbr()?.normal_scale(1.5);
# let _ = detailed;
# Ok(())
# }
```
