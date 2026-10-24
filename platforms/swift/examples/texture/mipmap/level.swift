import FragmentColor
import Foundation
let pixels = Data(repeating: 0, count: 8 * 8 * 4)

let chain = try Mipmap.build(pixels, TextureFormat.rgba8UnormSrgb, [8, 8])
let level_zero_bytes = try chain.level(0)