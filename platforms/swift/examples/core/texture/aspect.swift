
import FragmentColor

let renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
let pixels = [255,255,255,255]
let tex = try await renderer.createTextureWithSize(pixels, [1, 1])
let a = tex.aspect()