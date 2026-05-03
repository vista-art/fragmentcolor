
import FragmentColor

let renderer = Renderer()
// iOS: window/canvas provided by CAMetalLayer at runtime
let target = try await renderer.createTextureTarget([800, 600])
let shader = Shader.default()

let pass = Pass("First Pass")
pass.addShader(shader)

let pass2 = Pass("Second Pass")
pass2.addShader(shader)

// standalone
try renderer.render(pass, target)

// vector of passes rendered in order (any iterable of Pass is renderable)
try renderer.render([pass, pass2], target)