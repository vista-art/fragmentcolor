
import FragmentColor

let renderer = Renderer()

// Use your platform's windowing system to create a window
// iOS: window/canvas provided by CAMetalLayer at runtime

// Create a Target from it
let target = try await renderer.createTextureTarget([800, 600])
let texture_target = try await renderer.createTextureTarget([16, 16])

// RENDERING
try renderer.render(Shader(""), texture_target)

// That's it. Welcome to FragmentColor!