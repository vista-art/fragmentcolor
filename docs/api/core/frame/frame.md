# Frame

The [Frame](https://fragmentcolor.org/api/core/frame) object is a collection of [Pass](https://fragmentcolor.org/api/core/pass) objects that are rendered to a [Target](https://fragmentcolor.org/api/core/target) by the [Renderer](https://fragmentcolor.org/api/core/renderer).

It is used to render multiple passes to a single target, such as an opaque pass followed by a transparent pass.

You need to inject the [Frame](https://fragmentcolor.org/api/core/frame) object into the [Renderer](https://fragmentcolor.org/api/core/renderer) to render it.

## Example

```rust
use fragmentcolor::{Frame, Pass};

let mut pass1 = Pass::new("first");
let mut pass2 = Pass::new("second");

let mut frame = Frame::new();
frame.add_pass(&pass1);
frame.add_pass(&pass2);
```

## Methods

- ### constructor()

  Creates a new [Frame](https://fragmentcolor.org/api/core/frame) object.

- ### add_pass(pass: [Pass](https://fragmentcolor.org/api/core/pass))

  Adds a [Pass](https://fragmentcolor.org/api/core/pass) object to the [Frame](https://fragmentcolor.org/api/core/frame).
