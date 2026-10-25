import FragmentColor
import QuartzCore
let canvas = CAMetalLayer()

let renderer = Renderer()
let target = try await renderer.createTarget(canvas)

try target.resize([1024, 768])