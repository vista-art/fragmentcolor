# ToonMaterial::alpha_mode

Set how the renderer treats the alpha of the material's output, as glTF 2.0's
`alphaMode` does. It works as
[`PbrMaterial::alpha_mode`](https://fragmentcolor.org/api/scene/pbrmaterial#alpha_mode)
does:

- `AlphaMode::Opaque` (default): blending is off and alpha is ignored.
- `AlphaMode::Mask`: the fragment shader discards a fragment whose alpha
  falls below `alpha_cutoff`, and the rest draw as opaque. It suits leaves,
  hair cards and cut-outs with hard edges.
- `AlphaMode::Blend`: depth writes turn off and the color blends over what is
  behind it. The renderer sorts blended Models back to front by the pass's
  Camera.

It is pipeline state, so each mode builds its own pipeline once and reuses it.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{AlphaMode, ToonMaterial};

let glass = ToonMaterial::new()
    .base_color([0.6, 0.8, 1.0, 0.4])?
    .alpha_mode(AlphaMode::Blend);
# let _glass = glass;
# Ok(())
# }
```
