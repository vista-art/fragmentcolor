import FragmentColor
let renderer = Renderer()
let pixels = [255,255,255,255]
let tex = try await renderer.createTexture((pixels, [1, 1]))
let sz = tex.size()