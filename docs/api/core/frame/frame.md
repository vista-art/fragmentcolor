# Frame

The [Frame](https://fragmentcolor.org/api/core/frame) object is a collection of [Pass](https://fragmentcolor.org/api/core/pass) objects that are rendered to a [Target](https://fragmentcolor.org/api/core/target) by the [Renderer](https://fragmentcolor.org/api/core/renderer).

It is used to render multiple passes to a single target, such as an opaque pass followed by a transparent pass.

You need to inject the [Frame](https://fragmentcolor.org/api/core/frame) object into the [Renderer](https://fragmentcolor.org/api/core/renderer) to render it.

## Example

```rust talves-nao-araceca
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{ Shader, Pass, Frame, Renderer };

let renderer = Renderer::new();
let target = renderer.create_texture_target([10, 10]).await?;

let mut pass = Pass::new("First Pass");
let mut pass2 = Pass::new("Second Pass");

let mut frame = Frame::new();
frame.add_pass(&pass);
frame.add_pass(&pass2);

renderer.render(&frame, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## Methods

- ### constructor()

  Creates a new [Frame](https://fragmentcolor.org/api/core/frame) object.

- ### add_pass(pass: [Pass](https://fragmentcolor.org/api/core/pass))

  Adds a [Pass](https://fragmentcolor.org/api/core/pass) object to the [Frame](https://fragmentcolor.org/api/core/frame).
