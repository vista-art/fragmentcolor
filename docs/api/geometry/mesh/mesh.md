# Mesh

High-level geometry container. A Mesh owns a list of vertices and optional instances.
Internally it deduplicates vertices and always draws indexed and instanced
(instance_count defaults to 1 when none are provided). It draws a triangle list: without
`set_indices`, each three vertices make a triangle, and `Renderer.render` returns
`MeshError::VertexCount` for a count that isn't a multiple of 3.

The shader decides the vertex layout. When the mesh attaches through
`Shader::add_mesh` or `Pass::add_mesh`, each `@location` input of the
shader's vertex function looks for its data by name (instance first, then
vertex), then takes the position at `@location(0)`, then looks by the
attribute's own location (instance first, then vertex).

A plain number takes the type the shader declares for its input. The attach
checks every vertex and instance, and it fails with an error that names the
attribute when an input finds no data or a value doesn't fit its type. A vertex
or instance added after the attach holds every attribute the shader reads, or
`Renderer.render` returns `MeshError::InvalidVertex` or
`MeshError::InvalidInstance` naming the attribute and the item. An attribute
no attached shader reads can be left out, and it uploads as zeros: a shader
that lets go of the mesh, through `Shader.remove_mesh`, `Shader.clear_meshes`
or by dropping, takes what it reads along. `Renderer.render(mesh)` draws with
a shader of its own that checks the mesh again whenever it changes, so it adds
nothing to that list. The types a shader gave the attributes stay for as long as the
mesh lives.

A mesh draws with every `Renderer` that renders it. Each one uploads buffers
of its own from the mesh's vertices, indices and instances.

## Example

```rust
use fragmentcolor::mesh::{Mesh, Vertex};

let mut mesh = Mesh::new();
mesh.add_vertex([0.0, 0.5, 0.0]);
mesh.add_vertex([-0.5, -0.5, 0.0]);
mesh.add_vertex([0.5, -0.5, 0.0]);
```
