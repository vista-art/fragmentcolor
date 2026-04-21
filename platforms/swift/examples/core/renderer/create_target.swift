
import FragmentColor

let renderer = Renderer()

// Use your platform's windowing system to create a window.
let canvas = document.createElement('canvas')

let target = try await renderer.createTarget(canvas)