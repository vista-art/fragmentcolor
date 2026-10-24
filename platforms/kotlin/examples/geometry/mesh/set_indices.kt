import org.fragmentcolor.*

// A quad split into two triangles via explicit indexing. The four corners
// happen to carry distinct UVs (only positions repeat), so we keep them
// all and reference each by index.
val mesh = Mesh()
mesh.addVertices(listOf(Vertex(listOf(-0.5f, -0.5f)).set("uv", listOf(0.0f, 0.0f)), Vertex(listOf(0.5f, -0.5f)).set("uv", listOf(1.0f, 0.0f)), Vertex(listOf(0.5f, 0.5f)).set("uv", listOf(1.0f, 1.0f)), Vertex(listOf(-0.5f, 0.5f)).set("uv", listOf(0.0f, 1.0f))))
mesh.setIndices(listOf(0u, 1u, 2u, 0u, 2u, 3u))