import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

// Create a depth texture usable as a per-pass attachment
val depth = renderer.createDepthTexture(64u, 64u)

val mesh = Mesh()
mesh.addVertex(Vertex(listOf(0.0f, 0.0f, 0.0f)))
mesh.addVertex(Vertex(listOf(1.0f, 0.0f, 0.0f)))
mesh.addVertex(Vertex(listOf(0.0f, 1.0f, 0.0f)))
mesh.addVertex(Vertex(listOf(1.0f, 1.0f, 0.0f)))
val shader = Shader.fromMesh(mesh)
val pass = Pass("scene"); pass.addShader(shader)

// Attach depth texture to enable depth testing.
// Pipeline will include a matching depth-stencil state
pass.addDepthTarget(depth)

// Render as usual
renderer.render(pass, target)