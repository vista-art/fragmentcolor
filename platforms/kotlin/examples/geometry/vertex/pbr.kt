import org.fragmentcolor.*

// Build a triangle; override only what the mesh actually carries — NORMAL
// / COLOR0 / UV1 / TANGENT use their identity defaults from Vertex.pbr.
val mesh = Mesh()
mesh.addVertex(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set("uv0", floatArrayOf(0.5f, 1.0f)))
mesh.addVertex(Vertex.pbr(listOf(-0.5f, -0.5f, 0.0f)).set("uv0", floatArrayOf(0.0f, 0.0f)))
mesh.addVertex(Vertex.pbr(listOf(0.5f, -0.5f, 0.0f)).set("uv0", floatArrayOf(1.0f, 0.0f)))