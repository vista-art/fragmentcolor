import FragmentColor
let renderer = Renderer()
// Load encoded image bytes (PNG/JPEG) or use a file path
let image = "/healthcheck/public/favicon.png"
let tex = try await renderer.createTexture(image)