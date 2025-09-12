# Renderer

The [Renderer](https://fragmentcolor.org/api/core/renderer) is the main entry point for
[FragmentColor](https://fragmentcolor.org) and normally the first object you create.

It is used to render
[Shaders](https://fragmentcolor.org/api/core/shader),
[Passes](https://fragmentcolor.org/api/core/pass), and
[Frames](https://fragmentcolor.org/api/core/frame)
to a [Target](https://fragmentcolor.org/api/core/target) (canvas, window, or texture).

The [Renderer](https://fragmentcolor.org/api/core/renderer) internals are lazily initialized
when the user creates a [Target](https://fragmentcolor.org/api/core/target).

See the constructor [Renderer::new()](https://fragmentcolor.org/api/core/renderer/#renderernew)
description below for details.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Shader, Renderer, Target};

let renderer = Renderer::new();

// Use your platform's windowing system to create a window
let window = fragmentcolor::headless_window([800, 600]);

// Create a Target from it
let target = renderer.create_target(window).await?;
let texture_target = renderer.create_texture_target([16, 16]).await?;

// RENDERING
renderer.render(&Shader::default(), &texture_target)?;

// That's it. Welcome to FragmentColor!

# let s = target.size();
# assert_eq!([s.width, s.height], [800, 600]);
# let s2 = texture_target.size();
# assert_eq!([s2.width, s2.height], [16, 16]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## Methods
