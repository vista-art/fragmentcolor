# Scene::get_pass

Read one [`Pass`](https://fragmentcolor.org/api/core/pass) from the graph by
index, in render order. Returns `None` when the index is out of range.

The returned `Pass` is an Arc-shared clone of the one the Scene renders.
Configuring it (`load_previous`, `set_clear_color`, `set_target`) drives the
live pass, no re-insert needed. To borrow the whole graph at once, use
[`Scene::list_passes`](https://fragmentcolor.org/api/scene/scene/list_passes).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Scene};

let scene = Scene::new();
scene.add_pass(&Pass::new("backdrop"));
scene.add_pass(&Pass::new("geometry"));

let second = scene.get_pass(1).expect("two passes were added");
second.load_previous();
# assert!(scene.get_pass(2).is_none());
# Ok(())
# }
```
