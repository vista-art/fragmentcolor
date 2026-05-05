# TextureMipChain::prepare (Swift)

Swift override for `TextureMipChain::prepare`. The Swift binding (uniffi
constructor) takes positional args `(bytes: Data, format: TextureFormat,
size: Size?)` with `size` optional. The override sticks to native
positional Swift syntax — wrapping the call in a Rust-style tuple
(`prepare((bytes, format, size))`) crashes the swift-frontend type
checker (see CI healthcheck failures up to 87e61e5a).

## Example

```swift
import FragmentColor
import Foundation

// Raw RGBA path: include the size so prepare skips decoding.
let rawRgba = Data(repeating: 200, count: 8 * 8 * 4)
let chainRaw = try TextureMipChain.prepare(
    bytes: rawRgba,
    format: .rgba8UnormSrgb,
    size: Size(width: 8, height: 8, depth: nil)
)

// Upload the chain through the regular createTexture entry point.
let renderer = Renderer()
let texture = try await renderer.createTexture(input: .prepared(chainRaw))
let _ = chainRaw.levelCount()
let __ = texture.size()
```
