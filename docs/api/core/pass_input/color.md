# PassInput::color

The RGBA color the pass clears its target to, as four numbers from 0 to 1.
The pass stores its clear color at 8 bits per channel, so a value reads
back rounded to the nearest 1/255.

## Example

```rust
use fragmentcolor::Pass;

let pass = Pass::new("background");
pass.set_clear_color([1.0, 0.0, 0.0, 1.0]);
let color = pass.get_input().color();
# assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
```
