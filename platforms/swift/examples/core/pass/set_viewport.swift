
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let shader = Shader.default()
let pass = Pass("clipped")
pass.addShader(shader)

pass.setViewport([(0, 0), (32, 32)])

renderer.render(pass, target)