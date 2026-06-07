# Mipmap::level (Swift)

Swift override for `Mipmap::levels`. The Swift binding exposes a
`level(index)` accessor returning `Data` (one level at a time), rather
than a `levels()` collection. `&[Vec<u8>]` doesn't marshal through
uniffi, so callers index per level.

## Example

```swift
import FragmentColor
import Foundation

let pixels = Data(repeating: 0, count: 8 * 8 * 4)
let chain = try Mipmap.build(
    bytes: pixels,
    format: .rgba8UnormSrgb,
    size: Size(width: 8, height: 8, depth: nil)
)
let levelZeroBytes = try chain.level(index: 0)
let _ = levelZeroBytes
```
