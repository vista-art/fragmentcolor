
import FragmentColor

let renderer = Renderer()

// Use your platform's windowing system to create a window
let canvas = document.createElement('canvas')

// Create a Target from it
let target = try await renderer.createTarget(canvas)
let texture_target = try await renderer.createTextureTarget([16, 16])

// RENDERING
renderer.render(Shader(""), texture_target)

// That's it. Welcome to FragmentColor!