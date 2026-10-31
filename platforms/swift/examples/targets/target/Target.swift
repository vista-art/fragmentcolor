
import FragmentColor
import QuartzCore
let canvas = CAMetalLayer()

let renderer = Renderer()

// The canvas comes from your platform's windowing system.
let target = try renderer.createTarget(canvas)

// To animate, render again in your event loop...
try renderer.render(Shader.default(), target)
try renderer.render(Shader.default(), target)