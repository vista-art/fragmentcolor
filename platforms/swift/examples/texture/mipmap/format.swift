import FragmentColor
import Foundation

let pixels = Data(repeating: 200, count: 4 * 4 * 4)
let chain = try Mipmap.build(
    bytes: pixels,
    format: .rgba8UnormSrgb,
    size: Size(width: 4, height: 4, depth: nil)
)
let _ = chain.format()