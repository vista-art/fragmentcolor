# Texture::set_sampler_options

Update the texture sampler options (filtering, wrapping, etc.).

Note: changes take effect on next bind; the renderer recreates the sampler as needed.

## Example

```rust
use fragmentcolor::SamplerOptions;
let opts = SamplerOptions { repeat_x: true, repeat_y: true, smooth: true, compare: None };
tex.set_sampler_options(opts);
```
