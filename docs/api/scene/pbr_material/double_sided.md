# PbrMaterial::double_sided

Toggle backface culling for this material's pipeline. When `true`, the renderer skips
its default back-face cull and draws both faces of every triangle. When `false`
(the default), back faces are culled, the standard CCW-winding-is-front convention.

Seen from behind, a face lights as the side the camera sees: the shader turns its normal
around, and with it the tangent frame a normal map reads, as glTF asks. On a `Model`
mirrored by a negative scale, whose outer faces wind the other way on screen, those faces
still light as the front.

Mirrors glTF 2.0's `doubleSided`. Reach for it on thin geometry that legitimately reads
from both sides: a single-quad leaf card, a cloth banner, decals applied to non-closed
meshes, or any authored asset that comes out of a tool that didn't bother to mirror its
back faces. For closed solid geometry, leave it off; the culled back faces are a free
~50% fragment-shader saving on every draw.

Like `alpha_mode`, this is a pipeline-state flag and bakes into the WebGPU pipeline.
The renderer caches a separate pipeline per `(shader, alpha_mode, double_sided)` key,
so toggling it doesn't fight the shader cache as long as you stay on a small set of
values.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{AlphaMode, PbrMaterial, Renderer};

let renderer = Renderer::new();
// Leaf cards: thin, single-quad geometry; needs both sides + alpha cut-out.
let leaf = PbrMaterial::new()
    .double_sided(true)
    .alpha_mode(AlphaMode::Mask)
    .alpha_cutoff(0.5)?;

// The default is single-sided, with back-face culling on.
let solid_mesh = PbrMaterial::new().double_sided(false);
# let _ = (leaf, solid_mesh);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
