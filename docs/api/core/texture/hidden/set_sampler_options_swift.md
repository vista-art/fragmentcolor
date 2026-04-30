# Texture.setSamplerOptions(opts:)

Swift wrapper for `Texture::set_sampler_options`. `SamplerOptions` and
`CompareFunction` are uniffi-exported records / enums.

## Example

```swift
import FragmentColor

let renderer = Renderer()
let pixels: [UInt8] = [255, 255, 255, 255]
let texture = try await renderer.createTextureWithSize(pixels: pixels, size: Size(width: 1, height: 1))

let opts = SamplerOptions(repeatX: true, repeatY: true, smooth: true, compare: nil)
texture.setSamplerOptions(opts: opts)
```
