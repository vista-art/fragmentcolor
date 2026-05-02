import FragmentColor
let renderer = Renderer()
// Encoded image bytes (PNG / JPEG / etc.) â single tuple, no extra method.
let image = "/healthcheck/public/favicon.png"
let tex = try await renderer.createTexture(image[..])