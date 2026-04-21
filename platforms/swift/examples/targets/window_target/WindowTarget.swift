
import FragmentColor

// Use your platform's windowing system to create a window.
let canvas = document.createElement('canvas')

let renderer = Renderer()
let target = try await renderer.createTarget(canvas)

renderer.render(Shader(""), target)