import FragmentColor
import QuartzCore
let canvas = CAMetalLayer()

let renderer = Renderer()
let target = try await renderer.createTarget(canvas)
let size = target.size()