# Light

A `Light` is the first-class scene lighting primitive — paired with a
[`Camera`](https://fragmentcolor.org/api/scene/camera), it covers the two
inputs every shaded 3D render needs. The MVP supports a single directional
light: a parallel beam coming from a fixed world-space direction with a
tinted color. This is sun / moon / fill-light territory, and it's the
shape `Material::pbr` expects out of the box.

Internally a Light carries:

- A `direction` (vec3) — the world-space direction the light *travels in*.
  `[0, -1, 0]` is "noon sun pointing straight down". Length isn't
  normalized here; shaders normalize at sample time as needed.
- A `color` (vec3) — linear RGB intensity. `[1, 1, 1]` is full white,
  `[0.3, 0.0, 0.0]` is dim red, etc. Not premultiplied; the shader scales
  the diffuse + specular response by this value directly.

[`Light::bind`](https://fragmentcolor.org/api/scene/light/bind) writes the
two values into a Shader as `light.direction` and `light.color`. If the
shader doesn't declare those uniforms the call is a best-effort no-op
with a debug log — same pattern as `Camera::bind` and `Material`'s setters.

Point and spot lights are a follow-up. The type name reserves the
abstraction — when we add them, the API will either grow distinct
`Light::point(...)` / `Light::spot(...)` constructors or split into
distinct `DirectionalLight` / `PointLight` / `SpotLight` types; either
way today's `Light::directional` call site stays valid.

## Methods

| name          | what it does                                          |
| ------------- | ----------------------------------------------------- |
| `directional` | construct a directional light from direction + color  |
| `direction`   | read the world-space direction                        |
| `color`       | read the linear-RGB color                             |
| `bind`        | write `light.*` uniforms into a Shader                |

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material};

let material = Material::pbr()?;
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
sun.bind(material.shader());
# Ok(())
# }
```
