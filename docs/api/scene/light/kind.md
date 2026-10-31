# Light::kind

Read which kind this Light was constructed as: a `LightKind` value,
`Directional`, `Point`, or `Spot`, in every language.

Use this when inspecting a Light built elsewhere (e.g. loaded from a
glTF scene) to decide which kind-specific setters or getters to call.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, LightKind};

let sun = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
let bulb = Light::point([0.0, 2.5, 0.0], [1.0, 1.0, 1.0]);
let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
# assert_eq!(sun.kind(), LightKind::Directional);
# assert_eq!(bulb.kind(), LightKind::Point);
# assert_eq!(torch.kind(), LightKind::Spot);
# Ok(())
# }
```
