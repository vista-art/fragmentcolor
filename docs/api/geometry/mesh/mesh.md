# Mesh

High-level geometry container. A Mesh owns a list of vertices and optional instances.
Internally it deduplicates vertices and always draws indexed and instanced
(instance_count defaults to 1 when none are provided).

Vertex layouts are managed by the Shader. At render time, inputs declared in your
shader's vertex function (annotated with @location(N)) are derived from the source
and matched by name and type to Mesh properties across both streams (instance first,
then vertex).

Mapping is driven by shader reflection; there are no special-case names or reserved locations.
The renderer matches attributes by explicit location if provided (instance first, then vertex),
and otherwise by name.

If a required input cannot be found or its type does not match, rendering returns an error
indicating the missing attribute or mismatch.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex, VertexValue};

let mut mesh = Mesh::new();
mesh.add_vertex([0.0, 0.5, 0.0]);
mesh.add_vertex([-0.5, -0.5, 0.0]);
mesh.add_vertex([0.5, -0.5, 0.0]);
```
