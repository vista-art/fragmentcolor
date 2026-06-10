# Scene::find_pass

Find a [`Pass`](https://fragmentcolor.org/api/core/pass) in the scene's graph
by name, returning `None` when nothing matches. Names come from
[`Pass::new`](https://fragmentcolor.org/api/core/pass/new) (or whatever label a
loader assigned), and you can read one back with
[`Pass::name`](https://fragmentcolor.org/api/core/pass/name).

This is the name-addressed counterpart to
[`get_pass`](https://fragmentcolor.org/api/scene/scene/get_pass), which
addresses by index. Names aren't required to be unique; `find_pass` returns the
first match in render order. The returned `Pass` is an Arc-shared clone, so
configuring it drives the pass the Scene renders.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Scene};

let scene = Scene::new();
scene.add_pass(&Pass::new("backdrop"));
scene.add_pass(&Pass::new("geometry"));

// Look the geometry pass up by name to reconfigure it. A name with no
// match returns None instead.
let geometry = scene.find_pass("geometry");
# assert!(geometry.is_some());
# assert!(scene.find_pass("missing").is_none());
# Ok(())
# }
```
