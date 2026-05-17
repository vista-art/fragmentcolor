# Model::visible

Read the current visibility flag. Returns `true` for newly-built Models;
toggle with [`Model::set_visible`](https://fragmentcolor.org/api/scene/model/set_visible).

The renderer reads this every frame — hidden Models are skipped in both
the opaque-batched and blend-sorted draw paths without requiring a Pass
rebuild.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(Vertex::pbr([0.0, 0.5, 0.0]));
let model = Model::new(mesh, Material::pbr()?);

// Models start visible; toggle with `set_visible`.
let visible_now = model.visible();
# assert!(visible_now);
# Ok(())
# }
```

## See also

- [`Model::set_visible`](https://fragmentcolor.org/api/scene/model/set_visible) — the setter side.
- [`Model`](https://fragmentcolor.org/api/scene/model) — the parent page (full methods table).
