# Frame::connect (removed)

This API was removed. Use Pass::require to express dependencies.

## Migration

- Replace `frame.connect(parent, child)` with `child.require(&parent)`.
- Prefer rendering a root Pass directly for DAG pipelines.
- Frame remains useful for sequential pipelines with `add_pass`.

## Example

```rust
use fragmentcolor::Pass;
let parent = Pass::new("parent");
let child = Pass::new("child");
child.require(&parent)?; // was: frame.connect(&parent, &child)?
# Ok::<(), fragmentcolor::FrameError>(())
```
