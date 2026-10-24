import FragmentColor
import Foundation
let png = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))

// Imagine `png` came off your asset loader on a worker thread.

// Decode + mipmap generation. Pure CPU; run it wherever you like.
let chain = try Mipmap.build(png, TextureFormat.rgba8UnormSrgb)

// Back on the renderer thread, the upload is just a GPU write.
let renderer = Renderer()
let texture = try await renderer.createTexture(chain)