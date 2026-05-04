
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([16, 16])
try renderer.render(Shader(""), target)

let image = target.getImage().await