# Mesh::set_instance_count

Override how many instances to draw without providing per-instance attributes.

Use this when driving instance data from a storage buffer and indexing via
`@builtin(instance_index)` in the vertex shader. Common for compute-driven
simulations and large particle systems.

The override is automatically cleared if you later call `add_instance` /
`add_instances` (those carry their own count) or `clear_instances`.

## Example

```rust
use fragmentcolor::mesh::Mesh;
let mut m = Mesh::new();
m.add_vertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.01],
]);
// Draw one million instances, fetching per-particle data from a storage buffer.
m.set_instance_count(1_000_000);
```
