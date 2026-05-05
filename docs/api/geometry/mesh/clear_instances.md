# Mesh::clear_instances

Reset the mesh's instance state to the default (1 draw, no per-instance data).

Specifically, this:
- Drops any per-instance attributes added via `add_instance` / `add_instances`.
- Clears any count override previously set with `set_instance_count`.

After calling, the mesh renders as a single instance unless you populate it
again or call `set_instance_count`.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Instance};

let m = Mesh::new();
let red: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
m.add_instance(Instance::new().set("tint", red));
m.clear_instances(); // back to a single uninstanced draw
```
