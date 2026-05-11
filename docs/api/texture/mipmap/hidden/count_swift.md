# Mipmap::count (Swift)

Swift override for `Mipmap::count`. See
`build_swift.md` for the rationale.

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
let count = chain.count()
let _ = count
```
