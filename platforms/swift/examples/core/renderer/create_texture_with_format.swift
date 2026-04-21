import FragmentColor
let renderer = Renderer()
let image = "/healthcheck/public/favicon.png"
let tex = try await renderer.createTextureWithFormat(image, TextureFormat.Rgba)