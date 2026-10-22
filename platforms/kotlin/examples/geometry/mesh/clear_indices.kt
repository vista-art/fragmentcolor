import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertices(listOf(Vertex(listOf(-0.5f, -0.5f)), Vertex(listOf(0.5f, -0.5f)), Vertex(listOf(0.0f, 0.5f))))
mesh.setIndices(listOf(0, 1, 2))
mesh.clearIndices() // back to auto-derived dedup