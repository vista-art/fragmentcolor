# ToonMaterial::double_sided

Draw both faces of every triangle when `true`. When `false` (the default),
the renderer culls back faces, the ones whose vertices run clockwise on
screen. Mirrors glTF 2.0's `doubleSided`.

Thin geometry the camera can see from both sides, such as a flag or a
leaf, needs both faces. Like `alpha_mode`, it is pipeline state.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::ToonMaterial;

let flag = ToonMaterial::new().double_sided(true);
# let _flag = flag;
# Ok(())
# }
```
