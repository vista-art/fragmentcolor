# Material::base_color

Set the base color factor (linear RGBA). For dielectrics this is the diffuse
albedo; for metals it's the F0 reflectance. Alpha goes through the fragment
output unchanged.

Maps to the `material.base_color` uniform on the underlying shader. Default
is `[1, 1, 1, 1]`.

Returns a handle to the same Material (Arc-shared) so calls chain. See
[Material: Cloning is an Arc-share](https://fragmentcolor.org/api/scene/material#cloning-is-an-arc-share-not-a-deep-copy)
for what that means for mutation visibility.

An item that is NaN or an infinity returns `MaterialError::NotFinite`,
which names the call and the item, and the material keeps the value it
had. The other languages also refuse an item past the 32-bit float range,
such as `1e39`, with a message like `Material.baseColor: item 3 must be a
finite number that fits a 32-bit float, got 1e39`. JavaScript throws an
`Error` named `InvalidInputError`, Python raises `InvalidInputError`,
Swift throws `FragmentColorError` and Kotlin throws
`FragmentColorException`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let red = Material::pbr().base_color([1.0, 0.2, 0.2, 1.0])?;
# let _ = red;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
