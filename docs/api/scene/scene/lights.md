# Scene::lights

Return a snapshot of every [`Light`](https://fragmentcolor.org/api/scene/light)
added to this Scene via [`Scene::add`](https://fragmentcolor.org/api/scene/scene/add)
— including Lights the loader instantiated from glTF
`KHR_lights_punctual` nodes (unless you skipped them via the light filter
on [`Scene::load`](https://fragmentcolor.org/api/scene/scene/load)).

Each entry is an Arc-shared clone. Mutating a returned handle
(`set_color`, `set_intensity`, `set_position`, …) propagates to every
shader the Light occupies a slot in.

The default-injected Light (auto-fired when the Scene first renders with
no user-supplied Light) appears in this list too, so consumers can grab
the default and tweak it instead of supplanting it.

## Example

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Scene;

let scene = Scene::load("path/to/model.glb")?;

// Darken every loaded light to half intensity for a moody pass.
for light in scene.lights() {
    let current = light.intensity();
    light.set_intensity(current * 0.5);
}
# Ok(())
# }
```
