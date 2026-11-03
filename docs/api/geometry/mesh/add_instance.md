# Mesh::add_instance

Add a single per-instance attribute set to the mesh.

An `Instance` is a bag of named attributes (transform, color, id, …) sent to
the shader's per-instance inputs. It carries no position; the mesh's vertices
are reused for every instance.

Adding an instance also clears any previously set instance count override
(see `Mesh::set_instance_count`).

An instance added after the mesh attaches to a shader holds every instance
attribute that shader reads. `Renderer.render` returns
`MeshError::InvalidInstance` for one that lacks an attribute, naming the
attribute and the instance.

## Example

```rust
use fragmentcolor::{Mesh, Instance};

let m = Mesh::new();
m.add_instance(Instance::new().set("offset", [0.25, 0.10]).set("tint", [1.0, 0.0, 0.0, 1.0]));
```
