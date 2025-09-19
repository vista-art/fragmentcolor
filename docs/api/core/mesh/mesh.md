# Mesh

High-level geometry container. A Mesh owns a list of vertices and optional instances.
Internally it deduplicates vertices and always draws indexed and instanced
(instance_count defaults to 1 when none are provided).

Attribute mapping is AST-driven: at render time the vertex entry-point parameters
annotated with @location(N) are reflected from the shader and matched by name and type
against mesh properties across both streams (instance first, then vertex). Special
case: the shader name "pos" or "position" maps to the built-in mesh keys
"position2" (vec2<f32>) or "position3" (vec3<f32>) when present.

If a required input cannot be found or its type does not match, rendering returns an error
indicating the missing attribute or mismatch.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex, VertexValue};

let mut mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.5, 0.0]));
mesh.add_vertex(Vertex::new([-0.5, -0.5, 0.0]));
mesh.add_vertex(Vertex::new([0.5, -0.5, 0.0]));
```
