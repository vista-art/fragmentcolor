import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([16, 16])
let shader = Shader.default()

// A GPU error in this frame returns as a DeviceError.
try renderer.render(shader, target)