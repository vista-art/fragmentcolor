import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64u, 64u))

// One depth attachment shared across the 3D-content pass.
val depth = renderer.createDepthTexture(listOf(64u, 64u))

val mesh = Mesh()
mesh.addVertex(listOf(0.0f, 0.0f, 0.0f))
mesh.addVertex(listOf(1.0f, 0.0f, 0.0f))
mesh.addVertex(listOf(0.0f, 1.0f, 0.0f))
val shader = Shader.fromMesh(mesh)
val pass = Pass.fromShader("meshes", shader)

// Depth test on: closer fragments win, and the pass writes the depth
// buffer, so later draws in the same pass see it.
pass.setDepthTarget(depth)

renderer.render(pass, target)