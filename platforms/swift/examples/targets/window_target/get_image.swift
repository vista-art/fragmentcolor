import FragmentColor
import QuartzCore
let canvas = CAMetalLayer()

let renderer = Renderer()
let target = try renderer.createTarget(canvas)
let shader = Shader.default()
try renderer.render(shader, target)

// The same frame, drawn where it can be read back.
let offscreen = try await renderer.createTextureTarget([800, 600])
try renderer.render(shader, offscreen)
let image = try await offscreen.getImage()