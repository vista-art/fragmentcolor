# WindowTarget

[WindowTarget](https://fragmentcolor.org/api/window_target) is an implementation of [Target](https://fragmentcolor.org/api/target) that represents a rendering destination attached to a platform-specific window.

The [Target](https://fragmentcolor.org/api/target) object is a rendering destination for the [Renderer](https://fragmentcolor.org/api/renderer).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/texture_target)).

## Example

```rust
use fragmentcolor::{Renderer, Shader, HasDisplaySize};
use wgpu::rwh::RawWindowHandle;

# struct FakeWindow;
# impl RawWindowHandle for FakeWindow { fn as_raw_window_handle(&self) -> wgpu::rwh::RawWindowHandle { wgpu::rwh::RawWindowHandle::from_handle(self) } }
# impl HasDisplaySize for FakeWindow { fn size(&self) -> [u32; 2] { [800, 600] } }
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let window = FakeWindow;
let target = pollster::block_on(renderer.create_target(&window))?;
let shader = Shader::default();
renderer.render(&shader, &target)?;
# Ok(())
# }
```
