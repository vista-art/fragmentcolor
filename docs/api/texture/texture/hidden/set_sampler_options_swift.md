# Texture::set_sampler_options (Swift)

Swift override for `Texture::set_sampler_options`. The generic conversion
can't turn Rust's `SamplerOptions { .. }` struct literal into Swift, so this
page builds the options with the record's initializer.

## Example

```swift
import FragmentColor

let renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
let pixels: [UInt8] = [255, 255, 255, 255]

let texture = try await renderer.createTexture(pixels, [1, 1])
let opts = SamplerOptions(repeatX: true, repeatY: true, smooth: true, compare: nil)
texture.setSamplerOptions(opts)
```
