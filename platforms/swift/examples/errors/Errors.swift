import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([16, 16])
let shader = Shader.default()

// A GPU error in this frame returns as a DeviceError.
try renderer.render(shader, target)

// The kind of GPU error a DeviceError carries.
let kind = ErrorKind.memory