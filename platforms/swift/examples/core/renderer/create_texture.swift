import FragmentColor
import Foundation
let image = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))
let renderer = Renderer()
let tex = try await renderer.createTexture(image)