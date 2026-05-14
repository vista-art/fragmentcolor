# Light

A `Light` is the first-class scene lighting primitive — paired with a
[`Camera`](https://fragmentcolor.org/api/scene/camera), it covers the two
inputs every shaded 3D render needs. The variant is fixed at construction:

- [`Light::directional`](https://fragmentcolor.org/api/scene/light/directional) —
  parallel rays from a fixed world-space direction. Sun, moon, fill.
- [`Light::point`](https://fragmentcolor.org/api/scene/light/point) — radiates
  from a fixed world-space position with inverse-square distance falloff.
  Bulbs, candles, fireballs.
- [`Light::spot`](https://fragmentcolor.org/api/scene/light/spot) — point
  light constrained to a cone aligned with `-direction`, with smooth
  falloff between an inner and outer cone angle. Flashlights, headlamps,
  stage lighting.

Pass a Light to [`Pass::add`](https://fragmentcolor.org/api/core/pass#add)
or [`Scene::add`](https://fragmentcolor.org/api/scene/scene/add) to wire
its uniform fields into every shader the pass renders. The Light holds
Arc-shared state, so later mutators propagate to every shader the Light
has been wired into.

The underlying uniform follows glTF's
[`KHR_lights_punctual`](https://github.com/KhronosGroup/glTF/tree/main/extensions/2.0/Khronos/KHR_lights_punctual)
shape — one binding with a `kind` discriminator plus the union of fields
for all three variants. Fields irrelevant to the active variant are
stored but ignored by the shader (`position` on a directional, `direction`
on a point, cone angles on anything other than a spot).

## Fields

| name                | applies to             | what it does                                          |
| ------------------- | ---------------------- | ----------------------------------------------------- |
| `direction`         | directional, spot      | world-space travel direction; cone axis is `-direction` |
| `position`          | point, spot            | world-space origin                                     |
| `color`             | all                    | linear-RGB tint                                       |
| `intensity`         | all                    | scalar multiplier applied to `color`                   |
| `range`             | point, spot            | maximum influence radius (0 = unlimited)               |
| `inner_cone_angle`  | spot                   | full-intensity half-angle (radians)                    |
| `outer_cone_angle`  | spot                   | zero-contribution half-angle (radians)                 |

## Methods

| name                | what it does                                          |
| ------------------- | ----------------------------------------------------- |
| `directional`       | construct a directional light from direction + color  |
| `point`             | construct a point light from position + color         |
| `spot`              | construct a spot light from position + direction + color |
| `direction`         | read the world-space direction                        |
| `position`          | read the world-space position                         |
| `color`             | read the linear-RGB color                             |
| `intensity`         | read the scalar intensity multiplier                  |
| `range`             | read the maximum influence radius                     |
| `inner_cone_angle`  | read the inner cone half-angle                        |
| `outer_cone_angle`  | read the outer cone half-angle                        |
| `set_direction`     | update the world-space direction (live propagation)   |
| `set_position`      | update the world-space position (live propagation)    |
| `set_color`         | update the linear-RGB color (live propagation)        |
| `set_intensity`     | update the scalar intensity (live propagation)        |
| `set_range`         | update the influence radius (live propagation)        |
| `set_cone_angles`   | update the inner + outer cone angles (live propagation) |

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material, Mesh, Model, Pass, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0])
        .set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0])
        .set(Vertex::UV1, [0.0, 0.0]),
);
let model = Model::new(mesh, Material::pbr()?);

// A sun (key) and a torch (rim) in the same scene.
let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -0.3, -1.0], [1.0, 0.9, 0.7])
    .set_intensity(5.0)
    .set_cone_angles(0.15, 0.4);

let pass = Pass::new("scene");
pass.add(&model)?;
pass.add(&sun);
// Both lights accumulate per-fragment — the PBR shader loops over
// `lights.count` slots (cap of 8) and sums their contributions.
pass.add(&torch);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
