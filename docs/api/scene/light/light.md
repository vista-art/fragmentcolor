# Light

A `Light` is the first-class scene lighting primitive — paired with a
[`Camera`](https://fragmentcolor.org/api/scene/camera), it covers the two
inputs every shaded 3D render needs. The MVP supports a single directional
light: a parallel beam coming from a fixed world-space direction with a
tinted color. This is sun / moon / fill-light territory, and it's the
shape `Material::pbr` expects out of the box.

Pass a Light to [`Material::add`](https://fragmentcolor.org/api/scene/material#add)
to wire its `light.direction` and `light.color` into the material's shader.
The Light holds Arc-shared state, so later
[`set_direction`](https://fragmentcolor.org/api/scene/light/set_direction)
and [`set_color`](https://fragmentcolor.org/api/scene/light/set_color)
calls propagate to every Material that absorbed it.

Internally a Light carries:

- A `direction` (vec3) — the world-space direction the light *travels in*.
  `[0, -1, 0]` is "noon sun pointing straight down". Length isn't
  normalized here; shaders normalize at sample time as needed.
- A `color` (vec3) — linear RGB intensity. `[1, 1, 1]` is full white,
  `[0.3, 0.0, 0.0]` is dim red, etc. Not premultiplied; the shader scales
  the diffuse + specular response by this value directly.

Point and spot lights ship as follow-ups. The type name reserves the
abstraction — when they arrive, the API will either grow distinct
`Light::point(...)` / `Light::spot(...)` constructors or split into
distinct `DirectionalLight` / `PointLight` / `SpotLight` types; either
way today's `Light::directional` call site stays valid.

## Methods

| name            | what it does                                          |
| --------------- | ----------------------------------------------------- |
| `directional`   | construct a directional light from direction + color  |
| `direction`     | read the world-space direction                        |
| `color`         | read the linear-RGB color                             |
| `set_direction` | update the world-space direction (live propagation)   |
| `set_color`     | update the linear-RGB color (live propagation)        |

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material, Renderer};

let renderer = Renderer::new();
let material = Material::pbr(&renderer).await?;
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
material.add(&sun);

// Warm-tinted update — propagates to the Material above.
sun.set_color([1.0, 0.85, 0.7]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
