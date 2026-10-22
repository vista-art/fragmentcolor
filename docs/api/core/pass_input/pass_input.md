# PassInput

What a [Pass](https://fragmentcolor.org/api/core/pass) does with its target
before it draws: keep the previous contents, or clear them to a color.

Read it with `Pass::get_input`. Change it on the Pass with `load_previous`
or `set_clear_color`; a `PassInput` is a copy, so it doesn't change when
the Pass does.

## Example

```rust
use fragmentcolor::Pass;

let pass = Pass::new("overlay");
pass.load_previous();

let input = pass.get_input();
let keeps = input.loads_previous();
# assert!(keeps);
```
