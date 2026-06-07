import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

// One depth attachment shared across the 3D-content pass.
val depth = renderer.createDepthTexture(64u, 64u)

val mesh = Mesh()
mesh.addVertex(Vertex(listOf(0.0f, 0.0f, 0.0f)))
mesh.addVertex(Vertex(listOf(1.0f, 0.0f, 0.0f)))
mesh.addVertex(Vertex(listOf(0.0f, 1.0f, 0.0f)))
mesh.addVertex(Vertex(listOf(1.0f, 1.0f, 0.0f)))
val shader = Shader.fromMesh(mesh)
val pass = Pass("blobs"); pass.addShader(shader)

// Depth-test on — closer fragments win, the pass writes to the depth
// buffer so subsequent draws within the same pass see the depth.
pass.addDepthTarget(depth)

renderer.render(pass, target)