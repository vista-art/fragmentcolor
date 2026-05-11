import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

// One depth attachment shared across the 3D-content pass.
let depth = try await renderer.createDepthTexture([64, 64])

let mesh = Mesh()
try mesh.addVertex([0.0, 0.0, 0.0])
try mesh.addVertex([1.0, 0.0, 0.0])
try mesh.addVertex([0.0, 1.0, 0.0])
try mesh.addVertex([1.0, 1.0, 0.0])
let shader = Shader.fromMesh(mesh)
let pass = Pass("blobs"); pass.addShader(shader)

// Depth-test on — closer fragments win, the pass writes to the depth
// buffer so subsequent draws within the same pass see the depth.
try pass.addDepthTarget(depth)

try renderer.render(pass, target)