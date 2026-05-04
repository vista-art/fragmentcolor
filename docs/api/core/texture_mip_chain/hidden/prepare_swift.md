# TextureMipChain::prepare (Swift)

Swift override for `TextureMipChain::prepare`. The Swift binding (uniffi
constructor) takes positional args `(bytes: Data, format: TextureFormat,
size: Size?)` — `size` is optional. Wrapping the call in a Rust-style
tuple (`prepare((bytes, format, size))`) crashes the swift-frontend
type-checker (see CI healthcheck failures up to 87e61e5a); the override
sticks to native positional Swift syntax.

## Example

```swift
import FragmentColor
import Foundation

// Raw RGBA path — same method as encoded, just include the size.
let rawRgba = Data(repeating: 200, count: 8 * 8 * 4)
let chainRaw = try TextureMipChain.prepare(
    bytes: rawRgba,
    format: .rgba8UnormSrgb,
    size: Size(width: 8, height: 8, depth: nil)
)

// Hand the chain to the unified create_texture entry — same vocabulary.
let renderer = Renderer()
let texture = try await renderer.createTexture(input: .prepared(chainRaw))
let _ = chainRaw.levelCount()
let __ = texture.size()
```
