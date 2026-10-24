# Vertex::set

Attach a property to the vertex. `set` returns a new `Vertex` with the
property added, so chain the calls and keep the result.

The value is a number or a list of 1 to 4 numbers, and the shader decides
its type when the mesh attaches. A number fills a float input unless the
shader declares an integer type at that location: with
`@location(1) id: u32`, `set("id", 7)` uploads the `u32` 7. An integer
input takes whole numbers that fit its type. A value that doesn't fit,
such as `-1` for a `u32` or `2.5` for an `i32`, fails the attach with an
error that names the attribute, the declared type and the value. A
property no shader reads uploads as floats.

In Rust, an `f32` or a `u32` value keeps its type and must match the
input exactly. An unsuffixed literal, an `f64` or an `i32` takes the
shader's type like a number from any other language.

Locations and mapping:

- The first `set` for a key gives the property the next free `@location`,
  starting after the position. Setting the same key again keeps its location.
- When the mesh attaches, each shader input looks for its property by name
  (instance first, then vertex), then takes the position at `@location(0)`,
  then looks by the property's own location (instance first, then vertex).
- The first shader that reads a property fixes its type for that mesh. A
  shader that declares another type for it fails to attach.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mesh, Shader, Vertex};

let shader = Shader::new(r#"
  @vertex
  fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32, @location(2) weight: f32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos * weight, f32(id) * 0.1, 1.0);
  }
  @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
"#)?;

let vertex = Vertex::new([0.0, 0.0])
    .set("id", 7)
    .set("weight", 0.5);

let mesh = Mesh::new();
mesh.add_vertex(vertex);

// id uploads as the u32 7 and weight as the f32 0.5.
shader.add_mesh(&mesh)?;
# Ok(())
# }
```
