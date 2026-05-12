# Material::metallic

Set the metallic factor in `[0, 1]`. `0` is a pure dielectric (uses the
default F0 of 0.04), `1` is a pure metal (F0 = `base_color`). Values in
between linearly interpolate.

Maps to the `material.metallic` uniform. Default is `0.0` (dielectric);
`Material::pbr()` deviates from the glTF spec's `1.0` default here so the
out-of-the-box surface reads as a clean dielectric rather than dark gunmetal
under the factors-only built-in shader.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let chrome = Material::pbr().metallic(1.0).roughness(0.05);
# let _ = chrome;
# Ok(())
# }
```
