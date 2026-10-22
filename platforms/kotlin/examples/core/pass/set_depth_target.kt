import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64, 64))

// One depth attachment shared across the 3D-content pass.
val depth = renderer.createDepthTexture(listOf(64, 64))

val mesh = Mesh()
mesh.addVertex(listOf(0.0f, 0.0f, 0.0f))
mesh.addVertex(listOf(1.0f, 0.0f, 0.0f))
mesh.addVertex(listOf(0.0f, 1.0f, 0.0f))
mesh.addVertex(listOf(1.0f, 1.0f, 0.0f))
val shader = Shader.fromMesh(mesh)
val pass = Pass.fromShader("blobs", shader)

// Depth-test on — closer fragments win, the pass writes to the depth
// buffer so subsequent draws within the same pass see the depth.
pass.setDepthTarget(depth)

renderer.render(pass, target)