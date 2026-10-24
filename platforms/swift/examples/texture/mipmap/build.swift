import FragmentColor
import Foundation
let encoded_png_bytes = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))
let raw_rgba = Data(repeating: 200, count: 8 * 8 * 4)

// Encoded path: bytes plus the format you want the chain to live in.
// The dimensions come from the decoded image.
let chain = try Mipmap.build(encoded_png_bytes, TextureFormat.rgba8UnormSrgb)

// Raw path: include the size so build skips decoding.
let chain_raw = try Mipmap.build(raw_rgba, TextureFormat.rgba8UnormSrgb, [8, 8])

// Either chain uploads the same way.
let renderer = Renderer()
let texture = try await renderer.createTexture(chain)