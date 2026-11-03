
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let shader = Shader.default()
let pass = Pass("blend with previous")
try pass.addShader(shader)
pass.loadPrevious()

try renderer.render(pass, target)