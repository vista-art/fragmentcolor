# TextureMipChain::levels (Swift)

Swift override for `TextureMipChain::levels`. The Swift binding exposes
a single `level(index)` accessor returning `Data`, rather than a
`levels()` collection — same data, just one level at a time.

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
let levelZeroBytes = try chain.level(index: 0)
let _ = levelZeroBytes
```
