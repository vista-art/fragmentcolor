# TextureMipChain::base_size (Swift)

Swift override for `TextureMipChain::base_size`. The Rust example
destructures a `(u32, u32)` tuple via `let (width, height) = ...`,
which doesn't translate to Swift cleanly. The Swift binding returns a
`Size` struct with `.width` and `.height`.

## Example

```swift
import FragmentColor
import Foundation

let pixels = Data(repeating: 0, count: 16 * 16 * 4)
let chain = try TextureMipChain.prepare(
    bytes: pixels,
    format: .rgba8UnormSrgb,
    size: Size(width: 16, height: 16, depth: nil)
)
let sz = chain.baseSize()
let width = sz.width
let height = sz.height
let _ = (width, height)
```
