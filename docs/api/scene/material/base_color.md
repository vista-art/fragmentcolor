# Material::base_color

Set the base color factor (linear RGBA). For dielectrics this is the diffuse
albedo; for metals it's the F0 reflectance. Alpha goes through the fragment
output unchanged.

Maps to the `material.base_color` uniform on the underlying shader. Default
is `[1, 1, 1, 1]`.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let red = Material::pbr()?.base_color([1.0, 0.2, 0.2, 1.0]);
# let _ = red;
# Ok(())
# }
```
