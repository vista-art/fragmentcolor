import FragmentColor
let renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
let pixels = [255,255,255,255]

let texture = try await renderer.createTextureWithSize(pixels, [1,1])
let opts = {repeat_x: true, repeat_y: true, smooth: true, compare: nil}
texture.setSamplerOptions(opts)
