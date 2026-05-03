
import FragmentColor

// Use your platform's windowing system to create a window.
// iOS: window/canvas provided by CAMetalLayer at runtime

let renderer = Renderer()
let target = try await renderer.createTextureTarget([800, 600])

try renderer.render(Shader(""), target)