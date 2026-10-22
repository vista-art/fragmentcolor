# PassInput::loads_previous

True when the pass keeps the previous contents of its target, and false
when it clears them to its clear color first.

## Example

```rust
use fragmentcolor::Pass;

let pass = Pass::new("overlay");
pass.load_previous();
let keeps = pass.get_input().loads_previous();
# assert!(keeps);
```
