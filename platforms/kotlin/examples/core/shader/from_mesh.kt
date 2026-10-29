import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertices(listOf(listOf(-0.5f, -0.5f, 0.0f), listOf(0.5f, -0.5f, 0.0f), listOf(0.0f, 0.5f, 0.0f)))
val shader = Shader.fromMesh(mesh)