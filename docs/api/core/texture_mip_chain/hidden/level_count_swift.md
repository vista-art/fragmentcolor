# TextureMipChain::level_count (Swift)

Swift override for `TextureMipChain::level_count`. See
`prepare_swift.md` for the rationale.

## Example

```swift
import FragmentColor
import Foundation

let pixels = Data(repeating: 0, count: 8 * 8 * 4)
let chain = try TextureMipChain.prepare(
    bytes: pixels,
    format: .rgba8UnormSrgb,
    size: Size(width: 8, height: 8, depth: nil)
)
let count = chain.levelCount()
let _ = count
```
