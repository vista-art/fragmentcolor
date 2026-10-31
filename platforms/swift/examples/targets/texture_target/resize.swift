
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

try target.resize([128, 32])