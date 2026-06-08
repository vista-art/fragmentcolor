# Scene::remove_pass

Remove a [`Pass`](https://fragmentcolor.org/api/core/pass) from the scene's
pass graph. Identity is by handle: `Pass` is `Clone` and Arc-backed, so the
handle you pass in is matched against the graph by pointer, not by name.
Returns `true` when a pass left the graph, `false` when the handle wasn't
there.

If the removed pass was the one absorbing [`Scene::add`](https://fragmentcolor.org/api/scene/scene/add)
objects, the Scene forgets it. The next `add` builds a fresh absorb pass at
the end of the graph.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Scene};

let scene = Scene::new();
let backdrop = Pass::new("backdrop");
let overlay = Pass::new("overlay");
scene.add_pass(&backdrop);
scene.add_pass(&overlay);

// Drop the backdrop; the overlay stays.
let removed = scene.remove_pass(&backdrop);
# assert!(removed);
# assert_eq!(scene.list_passes().len(), 1);
# Ok(())
# }
```
