# Instance::set

Set a per-instance attribute. `set` returns a new `Instance` with the
attribute added, so chain the calls and keep the result.

The first `set` for a key gives it the next free instance `@location`,
in call order; setting the same key again keeps its location.

The value is a number or a list of 1 to 4 numbers, and the shader decides
its type when the mesh attaches, the same way it does for
[`Vertex::set`](https://fragmentcolor.org/api/geometry/vertex#set). A
number fills a float input unless the shader declares an integer type at
that location. An integer input takes whole numbers that fit its type,
and any other value fails the attach with an error that names the
attribute, the declared type, the value and the instance.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Instance, Mesh, Shader};

let shader = Shader::new(r#"
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>, @location(1) offset: vec2<f32>, @location(2) layer: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos + offset, f32(layer) * 0.1, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
"#)?;

let mesh = Mesh::new();
mesh.add_vertices([
    [-0.5, -0.5],
    [0.5, -0.5],
    [0.0, 0.5],
]);

let instance = Instance::new()
    .set("offset", [0.5, 0.0])
    .set("layer", 2);
mesh.add_instance(instance);

// offset uploads as a vec2<f32> and layer as the u32 2.
shader.add_mesh(&mesh)?;
# Ok(())
# }
```
