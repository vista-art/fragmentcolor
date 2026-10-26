# Light::point

Construct a point light. `position` is the world-space location the light
radiates from; `color` is linear-RGB intensity (`[1, 1, 1]` for full white).
Falloff is inverse-square distance. Bring
[`set_range`](https://fragmentcolor.org/api/scene/light#set_range) in to
cap the influence radius, and
[`set_intensity`](https://fragmentcolor.org/api/scene/light#set_intensity)
to scale the color uniformly.

Use this for lamps, candles, fireballs: anything where light radiates
outward from a fixed point. For a parallel-ray sun-like light see
[`Light::directional`](https://fragmentcolor.org/api/scene/light#directional);
for a cone-constrained beam see
[`Light::spot`](https://fragmentcolor.org/api/scene/light#spot).

A point light carries no direction or cone. Calling
[`set_direction`](https://fragmentcolor.org/api/scene/light#set_direction)
or [`set_cone_angles`](https://fragmentcolor.org/api/scene/light#set_cone_angles)
on a point light returns `LightError::FieldNotApplicable`.

A point light with a range lights only what lies within it, which lets the
renderer skip it everywhere else. Without one, every fragment pays for it.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Camera, Light, Material, Mesh, Model, Pass, Renderer, Vertex};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let wall = Mesh::new();
wall.add_vertices([
    Vertex::pbr([-2.0, -2.0, 0.0]),
    Vertex::pbr([2.0, -2.0, 0.0]),
    Vertex::pbr([2.0, 2.0, 0.0]),
    Vertex::pbr([-2.0, 2.0, 0.0]),
]);
wall.set_indices([0, 1, 2, 0, 2, 3]);

let camera = Camera::perspective(1.047, 1.0, 0.1, 100.0)?
    .look_at([0.0, 0.0, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])?;
let bulb = Light::point([0.0, 0.5, 1.0], [1.0, 0.95, 0.8]).set_intensity(15.0);
bulb.set_range(3.0)?;

let pass = Pass::new("lamp");
pass.add(&Model::new(wall, Material::pbr()))?;
pass.add(&camera)?;
pass.add(&bulb)?;
renderer.render(&pass, &target)?;

# assert_eq!(bulb.position(), Some([0.0, 0.5, 1.0]));
# assert_eq!(bulb.color(), [1.0, 0.95, 0.8]);
# assert!((bulb.intensity() - 15.0).abs() < 1.0e-6);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
