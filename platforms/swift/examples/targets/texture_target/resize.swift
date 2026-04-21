
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

target.resize([128, 32])