import FragmentColor
import Foundation

// Raw RGBA path: include the size so build skips decoding.
let rawRgba = Data(repeating: 200, count: 8 * 8 * 4)
let chainRaw = try Mipmap.build(
    bytes: rawRgba,
    format: .rgba8UnormSrgb,
    size: Size(width: 8, height: 8, depth: nil)
)

// Upload the chain through the regular createTexture entry point.
let renderer = Renderer()
let texture = try await renderer.createTexture(input: .prepared(chainRaw))
let _ = chainRaw.count()
let __ = texture.size()