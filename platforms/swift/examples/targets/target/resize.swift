
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 32])

target.resize([128, 64])