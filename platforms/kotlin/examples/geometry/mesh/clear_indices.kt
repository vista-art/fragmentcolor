import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertices(listOf(Vertex.new(listOf(-0.5f, -0.5f)), Vertex.new(listOf(0.5f, -0.5f)), Vertex.new(listOf(0.0f, 0.5f))))
mesh.setIndices(listOf(0.0f, 1.0f, 2.0f))
mesh.clearIndices(); // back to auto-derived dedup