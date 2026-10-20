
import FragmentColor
import QuartzCore
let canvas = CAMetalLayer()

let renderer = Renderer()

// The canvas comes from your platform's windowing system.
let target = try await renderer.createTarget(canvas)

try renderer.render(Shader.default(), target)