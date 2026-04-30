# Mesh::add_instance

Add a single per-instance attribute set to the mesh.

An `Instance` is a bag of named attributes (transform, color, id, …) sent to
the shader's per-instance inputs. It carries no position — the mesh's vertices
are reused for every instance.

Adding an instance also clears any previously set instance count override
(see `Mesh::set_instance_count`).

## Example

```rust
use fragmentcolor::mesh::{Mesh, Instance};

let m = Mesh::new();
let offset: [f32; 2] = [0.25, 0.10];
let tint:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
m.add_instance(Instance::new().set("offset", offset).set("tint", tint));
```
