# Light

`Light` is the unified type covering the three glTF `KHR_lights_punctual`
kinds: **directional** (parallel rays for sun, sky, fill), **point**
(radiates from a position with inverse-square falloff: lamps, candles,
fireballs), and **spot** (point light constrained to a cone: flashlights,
headlamps, theatre lighting). The kind is set once at construction via
[`Light::directional`](https://fragmentcolor.org/api/scene/light/directional),
[`Light::point`](https://fragmentcolor.org/api/scene/light/point), or
[`Light::spot`](https://fragmentcolor.org/api/scene/light/spot).

The unified surface keeps the public API simple: one type, one
[`Scene::add`](https://fragmentcolor.org/api/scene/scene/add) method, one
[`Pass::add`](https://fragmentcolor.org/api/core/pass/add) method, and one
binding per platform. Setters fall into two groups:

- **Universal**: [`set_color`](https://fragmentcolor.org/api/scene/light/set_color)
  and [`set_intensity`](https://fragmentcolor.org/api/scene/light/set_intensity)
  apply to every kind and return `Self` for chaining.
- **Kind-specific**: [`set_position`](https://fragmentcolor.org/api/scene/light/set_position),
  [`set_direction`](https://fragmentcolor.org/api/scene/light/set_direction),
  [`set_range`](https://fragmentcolor.org/api/scene/light/set_range), and
  [`set_cone_angles`](https://fragmentcolor.org/api/scene/light/set_cone_angles)
  return `Result<Self, LightError>`. Calling one on the wrong kind returns
  `LightError::FieldNotApplicable { kind, field }`.

The matching getters use `Option<T>` for the kind-specific fields:
[`position`](https://fragmentcolor.org/api/scene/light/position),
[`direction`](https://fragmentcolor.org/api/scene/light/direction),
[`range`](https://fragmentcolor.org/api/scene/light/range),
[`inner_cone_angle`](https://fragmentcolor.org/api/scene/light/inner_cone_angle),
[`outer_cone_angle`](https://fragmentcolor.org/api/scene/light/outer_cone_angle)
return `Some` only on the kinds that carry the value, `None` otherwise.

Lights hold Arc-shared state, so a single handle can be absorbed by
multiple Passes; later mutators propagate to every shader the Light has
been wired into.

## Cost model

The default Material::pbr shader uses **forward shading**. Every fragment
loops over every active light and accumulates the contribution. That makes
the per-frame cost roughly `O(fragments × lights)`, so the practical
ceiling depends on resolution and overdraw. The current cap of **32
lights** fits the forward path comfortably; for scenes with hundreds of
lights, a clustered / storage-buffer path is on the roadmap.

### Cap semantics: lights bind to the shader, not the Pass

The 32-light cap is enforced **per `ShaderObject`**, not per `Pass`. Two
Passes that share the same Material (and therefore the same shader)
share its light slots:

- Adding the same `Light` to Pass A and Pass B reuses one shader slot
  (the dedup is by shader-pointer identity). Both Passes render with
  that light visible.
- Adding two different `Light`s, one to Pass A, one to Pass B, packs
  them into two separate slots on the shared shader. **Both lights
  illuminate both Passes**, even though each was only `add(&...)`-ed
  to one. That's surprising if you think of a Pass as an isolated
  scene; it's the right answer if you think of a Material as the
  lighting model and the Passes as views into the same lit world.
- The cap exceeds at 32 *distinct* Lights on one shader. The 33rd
  returns `Err(PassError::LightCapReached { cap: 32 })`.

For genuinely independent lighting setups, build a separate Material
per scene (each `Material::pbr()` allocates a fresh shader, so its 32
slots are independent) or pass your own shader to `Material::custom`.
A single scene with one Material covers the typical case. The per-
shader semantic is what lets `Scene::add(&light)` propagate through
every Pass that renders the Scene without re-attaching.

## Constructors

| name          | what it does                                                |
| ------------- | ----------------------------------------------------------- |
| `directional` | parallel rays from a world-space direction                  |
| `point`       | radiates from a world-space position with distance falloff  |
| `spot`        | point light constrained to a cone                           |

## Methods

| name                 | what it does                                          |
| -------------------- | ----------------------------------------------------- |
| `kind`               | which kind this Light was constructed as              |
| `color`              | read the linear-RGB color                             |
| `intensity`          | read the scalar intensity multiplier                  |
| `position`           | read the world-space position (None on directional)   |
| `direction`          | read the world-space direction (None on point)        |
| `range`              | read the influence-radius cap (None on directional)   |
| `inner_cone_angle`   | read the inner cone half-angle (Some only on spot)    |
| `outer_cone_angle`   | read the outer cone half-angle (Some only on spot)    |
| `set_color`          | update the linear-RGB color (every kind)              |
| `set_intensity`      | update the scalar intensity (every kind)              |
| `set_position`       | update the world-space position (Err on directional)  |
| `set_direction`      | update the world-space direction (Err on point)       |
| `set_range`          | update the influence-radius cap (Err on directional)  |
| `set_cone_angles`    | update the cone half-angles (Err on non-spot)         |

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Light, Material, Mesh, Model, Pass, Renderer, Vertex};

let renderer = Renderer::new();
let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
);
let model = Model::new(mesh, Material::pbr());

let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9])
    .set_intensity(1.5);
let lamp = Light::point([0.0, 2.5, 0.0], [1.0, 0.95, 0.8]).set_intensity(15.0);
let torch = Light::spot([0.0, 1.8, 1.0], [0.0, -0.3, -1.0], [1.0, 0.9, 0.7])
    .set_intensity(5.0)
    .set_cone_angles(0.15, 0.4)?;

let pass = Pass::new("scene");
pass.add(&model)?.add(&sun)?.add(&lamp)?.add(&torch)?;
# let _ = renderer;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
