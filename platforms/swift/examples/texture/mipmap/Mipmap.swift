import FragmentColor
import Foundation

let renderer = Renderer()

// Minimal 1×1 RGBA raw pixel bytes.
let pixels = Data([255, 0, 0, 255])
let chain = try Mipmap.build(
    bytes: pixels,
    format: .rgba8UnormSrgb,
    size: Size(width: 1, height: 1, depth: nil)
)

// Hand the chain to the unified create_texture entry.
let texture = try await renderer.createTexture(input: .prepared(chain))
let _ = texture.size()