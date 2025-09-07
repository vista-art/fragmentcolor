# get_input() -> PassInput

Returns a copy of the current input configuration for this [Pass](https://fragmentcolor.org/api/pass).

It includes the clear/load behavior and clear color.

## Example

```rust
use fragmentcolor::Pass;

let pass = Pass::new("example");
let _input = pass.get_input();
// Inspect fields via dedicated APIs; internal fields are not public
```
