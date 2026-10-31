# Target::resize(size: Size { width, height } | (u32, u32) | [u32; 2])

Resizes the [Target](https://fragmentcolor.org/api/targets/target) to the given width and height.

It returns an error when the target can't take the size, and keeps the size it had. JavaScript throws
an `Error` named after the class Python raises, Python raises `FragmentColorError` or a subclass, Swift throws `FragmentColorError`, and Kotlin throws
`FragmentColorException`. A `TextureTarget` refuses a width or height of zero. A `WindowTarget` accepts
one, such as the 0×0 a minimized window reports, keeps its last size, and draws again once the window has one.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();
let mut target = renderer.create_texture_target([64, 32]).await?;

target.resize([128, 64])?;

# let size: [u32; 2] = target.size().into();
# assert_eq!(size, [128, 64]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
