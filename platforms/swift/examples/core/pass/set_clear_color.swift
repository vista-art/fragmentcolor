
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let shader = Shader.default()
let pass = Pass("solid background")
pass.addShader(shader)

try pass.setClearColor([0.1, 0.2, 0.3, 1.0])

try renderer.render(pass, target)