# Pass::get_input() -> PassInput

Returns a copy of the current input configuration for this [Pass](https://fragmentcolor.org/api/core/pass).

It includes the clear/load behavior and clear color.

## Example

```rust
use fragmentcolor::Pass;

let pass = Pass::new("example");
let input = pass.get_input();

# _ = input; // Silence unused variable warning
```
