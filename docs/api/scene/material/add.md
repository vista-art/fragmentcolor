# Material::add

Absorb a [Camera](https://fragmentcolor.org/api/scene/camera) or
[Light](https://fragmentcolor.org/api/scene/light) (or any custom
[`Component`](https://fragmentcolor.org/api/scene)) into the Material's
shader-uniform surface. The component is added by reference: later
mutations on the same value (`camera.look_at(...)`, `light.set_color(...)`)
propagate to every Material that absorbed it, with no further `add` call
required.

Chainable: returns the Material so multiple components can stack on a
single statement.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Light, Material, Renderer};

let renderer = Renderer::new();
let material = Material::pbr(&renderer).await?;

let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0)
    .look_at([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

material.add(&camera).add(&sun);

// Updating the camera later is enough — the Material picks the new
// view_proj up at the next render without re-adding.
camera.look_at([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
