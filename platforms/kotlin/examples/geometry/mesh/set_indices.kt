import org.fragmentcolor.*

// A quad split into two triangles via explicit indexing. The four corners
// happen to carry distinct UVs (only positions repeat), so we keep them
// all and reference each by index.
val mesh = Mesh()
val uv00 = listOf(0.0f, 0.0f)
val uv10 = listOf(1.0f, 0.0f)
val uv11 = listOf(1.0f, 1.0f)
val uv01 = listOf(0.0f, 1.0f)
mesh.addVertices(listOf(Vertex.new(listOf(-0.5f, -0.5f)).set("uv", uv00), Vertex.new(listOf(0.5f, -0.5f)).set("uv", uv10), Vertex.new(listOf(0.5f, 0.5f)).set("uv", uv11), Vertex.new(listOf(-0.5f, 0.5f)).set("uv", uv01)))
mesh.setIndices(listOf(0.0f, 1.0f, 2.0f, 0.0f, 2.0f, 3.0f))