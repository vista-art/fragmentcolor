
import FragmentColor

let renderer = Renderer()

// Use your platform's windowing system to create a window.
// iOS: window/canvas provided by CAMetalLayer at runtime

let target = try await renderer.createTextureTarget([800, 600])