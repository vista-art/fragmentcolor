# Pass::add_mesh

Attach a Mesh to this Pass.

- The mesh is attached to the last shader previously added to this Pass.
- Validates compatibility with that shader’s vertex inputs.
- Returns Result<(), ShaderError>; on error, the mesh is not attached.

If a Shader wasn't provided earlier, FragmentColor will create a default one.

## Example

```rust
use fragmentcolor::{Pass, Shader, Mesh, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(Vertex::new([0.0, 0.0]));

let shader = Shader::from_mesh(&mesh);
let pass = Pass::from_shader("pass", &shader);

pass.add_mesh(&mesh).expect("mesh is compatible");
```
