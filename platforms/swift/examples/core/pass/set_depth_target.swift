import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

// One depth attachment shared across the 3D-content pass.
let depth = try await renderer.createDepthTexture([64, 64])

let mesh = Mesh()
try mesh.addVertex([0.0, 0.0, 0.0])
try mesh.addVertex([1.0, 0.0, 0.0])
try mesh.addVertex([0.0, 1.0, 0.0])
let shader = try Shader.fromMesh(mesh)
let pass = Pass.fromShader("meshes", shader)

// Depth test on: closer fragments win, and the pass writes the depth
// buffer, so later draws in the same pass see it.
try pass.setDepthTarget(depth)

try renderer.render(pass, target)