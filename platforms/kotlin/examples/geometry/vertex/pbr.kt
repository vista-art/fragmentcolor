import org.fragmentcolor.*

// Build a triangle and set only what the mesh carries. NORMAL, COLOR0,
// UV1 and TANGENT keep their identity defaults from Vertex.pbr.
val mesh = Mesh()
mesh.addVertex(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)))
mesh.addVertex(Vertex.pbr(listOf(-0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(0.0f, 0.0f)))
mesh.addVertex(Vertex.pbr(listOf(0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(1.0f, 0.0f)))