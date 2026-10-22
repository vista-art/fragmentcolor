# Pass::new(name: &str) -> Self

Creates a new render Pass.

The name is required. Graphics debuggers show it as the pass label,
[`Scene::find_pass`](https://fragmentcolor.org/api/scene/scene#scenefind_pass)
looks passes up by it, and
[`require`](https://fragmentcolor.org/api/core/pass#passrequiredeps) tells
dependencies apart by it.

## Example

```rust
use fragmentcolor::Pass;

let pass = Pass::new("first pass");

# _ = pass;
```
