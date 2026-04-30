import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(arrayOf(64, 64))

// Create a depth texture usable as a per-pass attachment
val depth = renderer.createDepthTexture(arrayOf(64, 64))

val mesh = Mesh()
mesh.addVertex(arrayOf(0.0, 0.0, 0.0))
mesh.addVertex(arrayOf(1.0, 0.0, 0.0))
mesh.addVertex(arrayOf(0.0, 1.0, 0.0))
mesh.addVertex(arrayOf(1.0, 1.0, 0.0))
val shader = Shader.fromMesh(mesh)
val pass = Pass("scene"); pass.addShader(shader)

// Attach depth texture to enable depth testing.
// Pipeline will include a matching depth-stencil state
pass.addDepthTarget(depth)

// Render as usual
renderer.render(pass, target)