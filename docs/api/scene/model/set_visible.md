# Model::set_visible

Toggle the Model in or out of the next frame. Hidden Models are skipped
by the renderer in both the opaque-batched and blend-sorted draw paths;
no Pass rebuild needed.

The flag is Arc-shared with `ModelEntry` (the renderer's queue), so
toggles take effect on the very next `Renderer::render` call. Cheap:
one bool flip + one entry check during the draw-queue build.

Typical uses:

- LOD switches that hide whole sets of detail Models at a distance.
- Level / chapter transitions that swap which subset of a Scene
  renders without rebuilding the Scene.
- Temporary hides for editor gizmos, debug helpers, or selection
  highlights.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Vertex};

let mesh = Mesh::new();
mesh.add_vertex(Vertex::pbr([0.0, 0.5, 0.0]));
let blob = Model::new(mesh, Material::pbr());

// Wide zoom level — skip the detail blobs.
blob.set_visible(false);
// Zoom back in — turn them on again.
blob.set_visible(true);
# Ok(())
# }
```
