
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([10, 10])
let shader = Shader.default()

renderer.render(shader, target)