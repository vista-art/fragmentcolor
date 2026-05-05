import org.fragmentcolor.*
val m = Mesh()
m.addVertices(listOf(Vertex(listOf(-0.01f, -0.01f)), Vertex(listOf(0.01f, -0.01f)), Vertex(listOf(0.00f, 0.01f))))
// Draw one million instances, fetching per-particle data from a storage buffer.
m.setInstanceCount(1_000_000u)