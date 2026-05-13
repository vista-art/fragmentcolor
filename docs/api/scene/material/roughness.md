# Material::roughness

Set the surface roughness in `[0, 1]`. `0` is a perfect mirror; `1` is a
fully matte surface. The PBR shader uses the squared form (alpha = r²) for
the GGX normal distribution, which is the GGX/Trowbridge-Reitz convention
shared with glTF 2.0.

Maps to the `material.roughness` uniform. Default is `1.0`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let satin = Material::pbr()?.roughness(0.35);
# let _ = satin;
# Ok(())
# }
```
