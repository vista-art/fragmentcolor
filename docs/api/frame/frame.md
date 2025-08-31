# Frame

The [Frame](https://fragmentcolor.org/api/frame/) object is a collection of [Pass](https://fragmentcolor.org/api/pass/) objects that are rendered to a [Target](https://fragmentcolor.org/api/target/) by the [Renderer](https://fragmentcolor.org/api/renderer/).

It is used to render multiple passes to a single target, such as an opaque pass followed by a transparent pass.

You need to inject the [Frame](https://fragmentcolor.org/api/frame/) object into the [Renderer](https://fragmentcolor.org/api/renderer/) to render it.

## Example

```rust
use fragmentcolor::{ Shader, Pass, Renderer };

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target(&[10, 10]))?;
let object1 = Shader::default();
let object2 = Shader::default();

let pass = Pass::new("First Pass");
pass.add_shader(&object1);
pass.add_shader(&object2);

renderer.render(&pass, &target)?;

let pass2 = Pass::new("Second Pass");
pass2.add_shader(&object1);
pass2.add_shader(&object2);

let frame = Frame::new();
frame.add_pass(&pass);
frame.add_pass(&pass2);

renderer.render(&frame, &target)?;

Ok(())
# }
```

## Methods

- ### constructor()

  Creates a new Frame object.

- ### add_pass(pass: Pass)

  Adds a Pass object to the Frame.
