# Pass::set_viewport(viewport)

Sets the viewport region for this [Pass](https://fragmentcolor.org/api/core/pass).

The pass's shaders draw only inside this rectangle of the [Target](https://fragmentcolor.org/api/targets/target), and each shader's output scales to fill it.

The `viewport` argument accepts anything convertible into a `ScreenRegion`: an `[x, y, w, h]` array, a `(w, h)` tuple, an origin and a size as two pairs, `((x, y), (w, h))`, in each language's list or tuple form, or a `ScreenRegion` constructed explicitly. Pixels count from the top-left corner of the target.

A width or height of 0 is an error that leaves the pass's viewport as it was, and so is a far edge past `u32::MAX`, the largest pixel coordinate. A region whose max lies before its min, which a region built field by field or a JavaScript `{ minX, minY, maxX, maxY }` object can hold, returns `ScreenRegionError::Reversed` and is never swapped back. To draw on the whole target, leave the viewport unset.

A region that reaches past the target keeps its size and loses the part outside, and a region wholly outside the target draws nothing. When the pass has its own target from `set_target`, the region is measured against that target. A side past the device's largest 2D texture, `max_texture_dimension_2d`, makes `render` return `PassError::ViewportLimit`, which names the size and the limit, before it takes a frame.

The clear color still fills the whole target. To split one target between passes, call `load_previous` on each pass after the first. In the fragment stage, `@builtin(position)` counts pixels from the target's corner, not the region's.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Pass, Shader, ScreenRegion};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let shader = Shader::default();
let mut pass = Pass::new("clipped");
pass.add_shader(&shader)?;

pass.set_viewport(ScreenRegion::new((0, 0), (32, 32))?)?;

renderer.render(&pass, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
