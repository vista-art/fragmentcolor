# Frame::add_pass(pass: Pass)

Adds a [Pass](https://fragmentcolor.org/api/core/pass) to this [Frame](https://fragmentcolor.org/api/core/frame).

Passes are rendered in the order they are added.

## Example

```rust
use fragmentcolor::{Frame, Pass};

let mut pass1 = Pass::new("first");
let mut pass2 = Pass::new("second");

let mut frame = Frame::new();
frame.add_pass(&pass1);
frame.add_pass(&pass2);
```
