# Target

The [Target](https://fragmentcolor.org/api/target) interface (trait in Rust) is a rendering destination for the [Renderer](https://fragmentcolor.org/api/renderer) implemented by both [WindowTarget](https://fragmentcolor.org/api/window_target) and [TextureTarget](https://fragmentcolor.org/api/texture_target).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/texture_target)).

[Target](https://fragmentcolor.org/api/target) constructors are private and can only be created via the [Renderer](https://fragmentcolor.org/api/renderer) using either `Renderer.create_target(window)` for on-screen rendering, or `Renderer.create_texture_target()` for offscreen rendering.

## Examples

### WindowTarget (on-screen)

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

### TextureTarget (offscreen)

```rust
use fragmentcolor::{Renderer, Shader};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 64]))?;
let shader = Shader::default();
renderer.render(&shader, &target)?;
let image = target.get_image();
# Ok(())
# }
```
