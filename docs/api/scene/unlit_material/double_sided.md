# UnlitMaterial::double_sided

Draw both faces of every triangle when `true`. When `false` (the default),
the renderer culls back faces, the ones whose vertices run clockwise on
screen. Mirrors glTF 2.0's `doubleSided`.

A sprite or a label on a single quad that the camera can see from behind
needs both faces. Like `alpha_mode`, it is pipeline state.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::UnlitMaterial;

let card = UnlitMaterial::new().double_sided(true);
# let _card = card;
# Ok(())
# }
```
