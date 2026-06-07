import FragmentColor
import Foundation

let renderer = Renderer()
let pixels: [UInt8] = [255, 255, 255, 255]
let options = TextureOptions(
    size: Size(width: 1, height: 1, depth: nil),
    format: .rgba8UnormSrgb,
    sampler: SamplerOptions(repeatX: false, repeatY: false, smooth: true, compare: nil),
    mipmaps: false,
    usage: nil
)
// `.bytes(_:)` carries `Data`, not `[UInt8]`, so wrap the array explicitly.
let texture = try await renderer.createTexture(input: .bytes(Data(pixels)), options: options)

let opts = SamplerOptions(repeatX: true, repeatY: true, smooth: true, compare: nil)
texture.setSamplerOptions(opts: opts)