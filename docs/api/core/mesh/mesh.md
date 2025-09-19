# Mesh

High-level geometry container. A Mesh owns a list of vertices and optional instances.
Internally it deduplicates vertices and always draws indexed and instanced 
(instance_count defaults to 1 when none are provided).

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex, VertexValue};

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::from([0.0, 0.5, 0.0]));
mesh.add_vertex(Vertex::from([-0.5, -0.5, 0.0]));
mesh.add_vertex(Vertex::from([0.5, -0.5, 0.0]));
```
