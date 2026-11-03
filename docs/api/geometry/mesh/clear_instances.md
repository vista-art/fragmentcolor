# Mesh::clear_instances

Remove the mesh's instances and its instance count override.

Specifically, this:
- Drops any per-instance attributes added via `add_instance` / `add_instances`.
- Clears any count override previously set with `set_instance_count`.

After calling, a shader that reads no instance attributes draws the mesh once,
unless you add instances again or call `set_instance_count`. A shader that
reads instance attributes needs instances that hold them, so while it holds
the mesh, `Renderer.render` returns `MeshError::InvalidInstance` naming the
attribute until you add them again, or until the shader lets the mesh go
through `Shader.remove_mesh`, `Shader.clear_meshes` or by dropping.
`set_instance_count` doesn't stand in for them.

## Example

```rust
use fragmentcolor::{Mesh, Instance};

let m = Mesh::new();
m.add_instance(Instance::new().set("tint", [1.0, 0.0, 0.0, 1.0]));
m.clear_instances(); // no instances left
```
