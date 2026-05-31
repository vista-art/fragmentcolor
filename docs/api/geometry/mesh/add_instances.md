# Mesh::add_instances

Add many per-instance attribute sets to the mesh in one call.

Each `Instance` is a bag of named attributes (transform, color, id, …) sent to
the shader's per-instance inputs. Instances carry no position; the mesh's
vertices are reused for every instance.

Adding instances also clears any previously set instance count override
(see `Mesh::set_instance_count`).

## Example

```rust
use fragmentcolor::mesh::{Mesh, Instance};

let m = Mesh::new();
let red:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
let green: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
let blue:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];
m.add_instances([
    Instance::new().set("tint", red),
    Instance::new().set("tint", green),
    Instance::new().set("tint", blue),
]);
```
