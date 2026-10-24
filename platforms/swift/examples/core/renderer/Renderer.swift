
import FragmentColor
import QuartzCore
let canvas = CAMetalLayer()

let renderer = Renderer()

// The canvas comes from your platform's windowing system
let target = try await renderer.createTarget(canvas)
let texture_target = try await renderer.createTextureTarget([16, 16])

// RENDERING
try renderer.render(Shader.default(), texture_target)

// That's it. Welcome to FragmentColor!