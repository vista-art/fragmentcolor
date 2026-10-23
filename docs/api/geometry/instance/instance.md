# Instance

An `Instance` holds the per-instance attributes of one copy of a
[Mesh](https://fragmentcolor.org/api/geometry/mesh): an offset, a tint, an
id. The renderer draws the mesh once per instance and feeds each copy its
own values.

Instance attributes have their own `@location` numbering, separate from the
vertex attributes, and the shader matches them the same way: by location
first, then by name. On a name clash, the instance attribute wins over the
vertex attribute.

Most code doesn't build one by hand.
[`Vertex::create_instance`](https://fragmentcolor.org/api/geometry/vertex#create_instance)
copies a vertex's properties, and `Mesh::add_instance` takes a `Vertex` too.
Build an `Instance` when the per-instance data doesn't come from a vertex.

## Example

```rust
use fragmentcolor::mesh::{Instance, Mesh};

let mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0]);

let instance = Instance::new().set("offset", [0.5, 0.0]);
mesh.add_instance(instance);
```
