# Mesh::set_indices

Provide the index buffer directly, bypassing the mesh's automatic vertex
dedup pass. Use this when an asset already carries its own indexing: glTF
loaders, OBJ importers, or hand-authored meshes whose corners share
positions but need to keep distinct UVs, normals, or tangents (the typical
case for sharp creases and texture seams).

By default the mesh dedupes vertices by full attribute equality before
producing an index array. That's fine for hand-built meshes, wrong for assets
where two corners with identical positions must stay separate because
their other attributes differ. After `set_indices`, every vertex you added
with `add_vertex` is packed in insertion order and the indices you supply
are used verbatim. Call `clear_indices` to return to the auto-derived
path.

## Example

```rust
use fragmentcolor::{Mesh, Vertex};

// A quad split into two triangles via explicit indexing. The four corners
// happen to carry distinct UVs (only positions repeat), so we keep them
// all and reference each by index.
let mesh = Mesh::new();
let uv00: [f32; 2] = [0.0, 0.0];
let uv10: [f32; 2] = [1.0, 0.0];
let uv11: [f32; 2] = [1.0, 1.0];
let uv01: [f32; 2] = [0.0, 1.0];
mesh.add_vertices([
    Vertex::new([-0.5, -0.5]).set("uv", uv00),
    Vertex::new([ 0.5, -0.5]).set("uv", uv10),
    Vertex::new([ 0.5,  0.5]).set("uv", uv11),
    Vertex::new([-0.5,  0.5]).set("uv", uv01),
]);
mesh.set_indices([0, 1, 2, 0, 2, 3]);
```
