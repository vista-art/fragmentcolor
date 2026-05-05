import FragmentColor
import Foundation

// Raw RGBA path: include the size so prepare skips decoding.
let rawRgba = Data(repeating: 200, count: 8 * 8 * 4)
let chainRaw = try TextureMipChain.prepare(
    bytes: rawRgba,
    format: .rgba8UnormSrgb,
    size: Size(width: 8, height: 8, depth: nil)
)

// Upload the chain through the regular createTexture entry point.
let renderer = Renderer()
let texture = try await renderer.createTexture(input: .prepared(chainRaw))
let _ = chainRaw.levelCount()
let __ = texture.size()