# Mesh::add_instances

Add many per-instance attribute sets to the mesh in one call.

Each `Instance` is a bag of named attributes (transform, color, id, …) sent to
the shader's per-instance inputs. Instances carry no position; the mesh's
vertices are reused for every instance.

Adding instances also clears any previously set instance count override
(see `Mesh::set_instance_count`).

The instances land as one change, so a render on another thread sees all of
them or none.

Every instance counts, even one without attributes: three empty instances
draw the mesh three times, and the shader tells them apart with
`@builtin(instance_index)`. As with `add_instance`, an instance added after
the mesh attaches to a shader holds every instance attribute that shader reads.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Instance};

let m = Mesh::new();
m.add_instances([
    Instance::new().set("tint", [1.0, 0.0, 0.0, 1.0]),
    Instance::new().set("tint", [0.0, 1.0, 0.0, 1.0]),
    Instance::new().set("tint", [0.0, 0.0, 1.0, 1.0]),
]);
```
