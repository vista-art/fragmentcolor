
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])
let size = target.size()
let width = size.width
let height = size.height
let depth = size.depth