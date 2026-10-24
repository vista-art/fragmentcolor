import FragmentColor
import QuartzCore
let canvas = CAMetalLayer()

let renderer = Renderer()
let target = try await renderer.createTarget(canvas)
try renderer.render(Shader.default(), target)

let image = try await target.getImage()