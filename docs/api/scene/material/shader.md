# Material::shader

Borrow the underlying `Shader` to drop down to direct uniform manipulation.
This is the escape hatch when the Material's typed setters don't cover what you need
(custom uniforms beyond the glTF PBR field set, raw texture binds, anything
the [Camera](https://fragmentcolor.org/api/scene/camera) and
[Light](https://fragmentcolor.org/api/scene/light) helpers don't speak to).

The returned reference is the same `Shader` the Material is built around, so
changes propagate immediately to every Model that uses this Material. If you
want a Material variant with different state, build a fresh
`Material::pbr().<setters>` rather than cloning. `Material`
clones share their Shader handle (Arc-clone) so mutations are visible across
all clones.

For camera + light state, prefer absorbing the typed
[Camera](https://fragmentcolor.org/api/scene/camera) and
[Light](https://fragmentcolor.org/api/scene/light) handles into the
[Pass](https://fragmentcolor.org/api/core/pass) that's about to render this
Material rather than calling `shader().set(...)` by hand. The typed
handles propagate updates live across every absorbed shader.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Material;

// Direct uniform access for a custom field that isn't covered by the
// Material setters or by Camera / Light.
let material = Material::pbr();
material.shader().set("material.alpha_cutoff", 0.25)?;
# Ok(())
# }
```
