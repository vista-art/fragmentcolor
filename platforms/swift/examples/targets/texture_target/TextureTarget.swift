
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let shader = Shader.default()
try renderer.render(shader, target)

let image = try await target.getImage()