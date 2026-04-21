
import FragmentColor

let renderer = Renderer()

// Use your platform's windowing system to create a window.
let canvas = document.createElement('canvas')

let target = try await renderer.createTarget(canvas)

// To animate, render again in your event loop...
renderer.render(Shader(""), target)
renderer.render(Shader(""), target)