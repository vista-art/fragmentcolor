import FragmentColor

let renderer = Renderer()
let pixels: [UInt8] = [255, 255, 255, 255]
let texture = try await renderer.createTextureWithSize(pixels: pixels, size: Size(width: 1, height: 1))

let opts = SamplerOptions(repeatX: true, repeatY: true, smooth: true, compare: nil)
texture.setSamplerOptions(opts: opts)