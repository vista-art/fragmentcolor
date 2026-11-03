
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 32])

try target.resize([128, 64])