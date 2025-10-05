# Mesh::set_instance_count

Override how many instances to draw without providing per-instance attributes.

Use this when driving instance data from a storage buffer and indexing via @builtin(instance_index).

## Example

```rust
use fragmentcolor::mesh::Mesh;
let mut m = Mesh::new();
m.add_vertices([
    [-0.01, -0.01],
    [ 0.01, -0.01],
    [ 0.00,  0.01],
]);
// draw one million instances
m.set_instance_count(1_000_000);
```
