# Scene::set_passes

Replace the scene's entire pass graph with a new ordered list. The passes
render in vec order. This is the sugar for "load, reorder, render" when you'd
rather hand over a fresh ordering than call
[`remove_pass`](https://fragmentcolor.org/api/scene/scene/remove_pass) and
[`add_pass`](https://fragmentcolor.org/api/scene/scene/add_pass) one at a
time.

If the default pass that [`Scene::add`](https://fragmentcolor.org/api/scene/scene/add)
routes objects into survives in the new list, the Scene keeps using it. If you
drop it, the Scene forgets it and the next `add` builds a fresh one at the end.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Scene};

let scene = Scene::new();
scene.add_pass(&Pass::new("scratch"));

// Swap in a deliberate order: shadow map, then geometry, then overlay.
scene.set_passes(vec![
    Pass::new("shadow"),
    Pass::new("geometry"),
    Pass::new("overlay"),
]);
# assert_eq!(scene.list_passes().len(), 3);
# Ok(())
# }
```
