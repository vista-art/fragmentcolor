# Material::normal_scale

Scales the per-fragment normal perturbation read from a tangent-space normal
map, mirroring glTF 2.0's `normalTextureInfo.scale`. Higher values exaggerate
the bumps; values below 1 soften them.

Maps to the `material.normal_scale` uniform. Default is `1.0`. The default
PBR shader samples `normal_map`, decodes the bytes, and scales the XY
perturbation by this value before combining with the world-space normal.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let detailed = Material::pbr()?.normal_scale(1.5);
# let _ = detailed;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
