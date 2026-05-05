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