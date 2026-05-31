# Material::alpha_cutoff

Sets the alpha threshold used by `AlphaMode::Mask`. Fragments whose
`base_color.a` falls below this value are `discard`ed in the fragment
shader. Stored on `material.alpha_cutoff`; default `0.5`, matching glTF 2.0.

Only `AlphaMode::Mask` reads this value; in `Opaque` and `Blend` modes the
fragment shader ignores it. Use it together with `Material::alpha_mode`
when you want hard-edged cut-out transparency (foliage, chain-link, decals).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let foliage = Material::pbr()?.alpha_cutoff(0.3);
# let _foliage = foliage;
# Ok(())
# }
```
