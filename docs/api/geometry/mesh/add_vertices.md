# Mesh::add_vertices

Add many vertices to the mesh.

As with `add_vertex`, a vertex added after the mesh attaches to a shader
holds every attribute that shader reads, or `Renderer.render` returns
`MeshError::InvalidVertex` naming the attribute and the vertex.

## Example

```rust
use fragmentcolor::mesh::Mesh;
let mut m = Mesh::new();
m.add_vertices([
  [0.0, 0.0],
  [1.0, 0.0],
]);
```
