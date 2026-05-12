# Material::alpha_cutoff

Sets the alpha threshold used when (a future) alpha mode is `MASK` —
fragments whose alpha falls below this value are discarded. Stored on
`material.alpha_cutoff`; default `0.5`, matching glTF 2.0.

Alpha modes themselves are not yet wired through the renderer in this
release; setting this value today is forward-compatible — once the alpha-mode
plumbing lands you don't have to rebuild any Materials you've already
authored.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

let foliage = Material::pbr().alpha_cutoff(0.3);
# let _ = foliage;
# Ok(())
# }
```
