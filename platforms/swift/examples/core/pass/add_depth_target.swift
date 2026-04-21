import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

// Create a depth texture usable as a per-pass attachment
let depth = try await renderer.createDepthTexture([64, 64])

let mesh = Mesh()
mesh.addVertex([0.0, 0.0, 0.0])
mesh.addVertex([1.0, 0.0, 0.0])
mesh.addVertex([0.0, 1.0, 0.0])
mesh.addVertex([1.0, 1.0, 0.0])
let shader = Shader.fromMesh(mesh)
let pass = Pass("scene"); pass.addShader(shader)

// Attach depth texture to enable depth testing.
// Pipeline will include a matching depth-stencil state
pass.addDepthTarget(depth)

// Render as usual
renderer.render(pass, target)