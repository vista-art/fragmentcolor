# create_target(target: Canvas | Window)

Creates a [Target](https://fragmentcolor.org/api/core/target) attached to a platform-specific canvas or window.

## Example

```rust
use fragmentcolor::Renderer;

# // Platform-specific window binding (winit shown as an example)
# // use winit::event_loop::EventLoop;
# // use winit::window::WindowBuilder;
# // fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
# // let event_loop = EventLoop::new()?;
# // let window = WindowBuilder::new().build(&event_loop)?;
# // let target = renderer.create_target(&window)?;
# // Ok(())
# // }
```
