import FragmentColor

let renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
let pixels: [UInt8] = [255, 255, 255, 255]

let texture = try await renderer.createTexture(pixels, [1, 1])
let opts = SamplerOptions(repeatX: true, repeatY: true, smooth: true, compare: nil)
texture.setSamplerOptions(opts)