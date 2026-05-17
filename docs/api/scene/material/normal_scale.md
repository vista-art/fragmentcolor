# Material::normal_scale

Scales the per-fragment normal perturbation read from a tangent-space normal
map, mirroring glTF 2.0's `normalTextureInfo.scale`. Higher values exaggerate
the bumps; values below 1 soften them.

Maps to the `material.normal_scale` uniform. Default is `1.0`. The default
PBR shader samples `normal_map`, decodes the bytes, and scales the XY
perturbation by this value before combining with the world-space normal.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let detailed = Material::pbr()?.normal_scale(1.5);
# let _detailed = detailed;
# Ok(())
# }
```
