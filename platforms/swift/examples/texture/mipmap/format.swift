import FragmentColor
import Foundation
let pixels = Data(repeating: 200, count: 4 * 4 * 4)

let chain = try Mipmap.build(pixels, TextureFormat.rgba8UnormSrgb, [4, 4])
let format = chain.format()