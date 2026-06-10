# Pass::name

Read the pass's name — the string passed to
[`Pass::new`](https://fragmentcolor.org/api/core/pass/new) (or the label a
loader gave it). Names are how you address a pass by something other than its
index: pair this with
[`Scene::find_pass`](https://fragmentcolor.org/api/scene/scene/find_pass) to
locate a pass in a scene's graph, and it's the label graphics debuggers
(RenderDoc, Xcode GPU capture) show for the pass.

Names aren't required to be unique. If two passes share a name, name lookup
returns the first one.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Pass;

let shadow = Pass::new("shadow");
let label = shadow.name();
# assert_eq!(label, "shadow");
# Ok(())
# }
```
