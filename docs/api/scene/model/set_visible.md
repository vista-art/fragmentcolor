# Model::set_visible

Toggle the Model in or out of the next frame. Hidden Models are skipped
by the renderer in both the opaque-batched and blend-sorted draw paths;
no Pass rebuild needed. Hiding a Model hides what its Material draws, and
each shader that draws any of the pass's Models skips the hidden Model's Mesh.
A shader of your own that draws none of them, such as an outline, still draws
the Mesh when it reads no Model columns (`model_0` to `model_3`). One that
reads them makes `Renderer::render` return `MeshError::InvalidInstance`,
whether the Model is hidden or not.

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
let detail = Model::new(mesh, Material::pbr());

// Far from the camera, hide the detail Model.
detail.set_visible(false);
// Closer again, show it.
detail.set_visible(true);
# Ok(())
# }
```
