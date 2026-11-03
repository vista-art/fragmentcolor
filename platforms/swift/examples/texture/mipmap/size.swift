import FragmentColor
import Foundation
let pixels = Data(repeating: 0, count: 16 * 16 * 4)

let chain = try Mipmap.build(pixels, TextureFormat.rgba8UnormSrgb, [16, 16])
let size = chain.size()
let width = size.width
let height = size.height