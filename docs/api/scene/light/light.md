# Light

`Light` is the unified type covering the three glTF `KHR_lights_punctual`
kinds: **directional** (parallel rays for sun, sky, fill), **point**
(radiates from a position with inverse-square falloff: lamps, candles,
fireballs), and **spot** (point light constrained to a cone: flashlights,
headlamps, theater lighting). The kind is set once at construction via
[`Light::directional`](https://fragmentcolor.org/api/scene/light#directional),
[`Light::point`](https://fragmentcolor.org/api/scene/light#point), or
[`Light::spot`](https://fragmentcolor.org/api/scene/light#spot).

The unified surface keeps the public API simple: one type, one
[`Scene::add`](https://fragmentcolor.org/api/scene/scene#add) method, one
[`Pass::add`](https://fragmentcolor.org/api/core/pass#add) method, and one
binding per platform. Setters fall into two groups:

- **Universal**: [`set_color`](https://fragmentcolor.org/api/scene/light#set_color)
  and [`set_intensity`](https://fragmentcolor.org/api/scene/light#set_intensity)
  apply to every kind and return `Self` for chaining.
- **Kind-specific**: [`set_position`](https://fragmentcolor.org/api/scene/light#set_position),
  [`set_direction`](https://fragmentcolor.org/api/scene/light#set_direction),
  [`set_range`](https://fragmentcolor.org/api/scene/light#set_range), and
  [`set_cone_angles`](https://fragmentcolor.org/api/scene/light#set_cone_angles)
  return `Result<Self, LightError>`. Calling one on the wrong kind returns
  `LightError::FieldNotApplicable { kind, field }`.

The matching getters use `Option<T>` for the kind-specific fields:
[`position`](https://fragmentcolor.org/api/scene/light#position),
[`direction`](https://fragmentcolor.org/api/scene/light#direction),
[`range`](https://fragmentcolor.org/api/scene/light#range),
[`inner_cone_angle`](https://fragmentcolor.org/api/scene/light#inner_cone_angle),
[`outer_cone_angle`](https://fragmentcolor.org/api/scene/light#outer_cone_angle)
return `Some` only on the kinds that carry the value, `None` otherwise.

Lights hold Arc-shared state, so a single handle can be absorbed by
multiple Passes, and a later setter reaches every shader the Light lights
at the next render.

## Lifetime

A Light lights as long as a `Scene`, a `Pass` or a handle of yours holds
it. Each `Pass` and each `Scene` holds the lights added to it, so
`scene.add(&light)` keeps lighting after the local `light` handle drops.
A `Model` holds no lights. When it joins a pass, its shader takes every
light that pass holds, and so does a shader added with
[`Pass::add_shader`](https://fragmentcolor.org/api/core/pass#add_shader).
Once the last pass, scene and handle holding a Light are gone, the
shaders it reached draw without it.

## Cost model

`Material::pbr` shades each fragment with the lights that reach it. At
each render the renderer sorts every light with a finite `range` into a
grid of clusters over the pass's camera view, and a fragment loops over
the lights of its own cluster. The cost of a fragment grows with the
lights that overlap it, not with every light in the scene.

A light without a range reaches every fragment, so every fragment pays
for it. That covers each directional light and every point or spot light
left at the default range of 0. Keep those to a few suns and fill lights,
and give lamps, torches and other local lights a range.

A still scene sorts its lights once. Moving a light, changing its range
or cone, or moving the camera sorts them again at the next render. A new
color or intensity sorts nothing and uploads only the lights from the
first one that changed to the last.

### How many lights a shader takes

The device sets the cap. A device with compute shaders and storage
buffers in fragment shaders, as on WebGPU, Vulkan, Metal and DirectX 12,
takes as many lights as its largest storage buffer binding holds: about
two million at the common 128 MiB. Every OpenGL backend, WebGL2
included, takes the lights through a uniform block, and so does any
other device, so the cap is what the device's largest uniform block
holds, from 253 lights in 16 KiB up to 1,021.

Past the cap, `pass.add(&light)` returns
`PassError::LightCapReached { cap }`, and so does adding a `Model` whose
shader would take more lights than that when it joins the pass. Lights
added before the renderer opened a device are checked at render, which
returns `RendererError::PassError` with the same error.

### Lights bind to the shader, not the Pass

Each shader keeps its own list of lights, and the cap applies to that
list. Two Passes that share a Material, and so its shader, share its
lights:

- Adding the same `Light` to Pass A and Pass B lists it once on the
  shared shader. Both Passes render with it.
- Adding one `Light` to Pass A and another to Pass B lists both on the
  shared shader, so **both lights illuminate both Passes**. Think of a
  Material as the lighting model and the Passes as views into the same
  lit world.

For independent lighting setups, build a separate Material per scene
(each `Material::pbr()` makes a new shader with its own list) or pass
your own shader to `Material::custom`. One scene with one Material
covers the typical case, and the shared list is what lets
`Scene::add(&light)` reach every Pass that renders the Scene.

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
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);
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
