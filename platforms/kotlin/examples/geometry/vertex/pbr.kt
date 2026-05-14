import org.fragmentcolor.*

val mesh = Mesh()
for (pos, uv) in arrayOf((listOf(0.0f, 0.5f, 0.0f), listOf(0.5f, 1.0f)), (listOf(-0.5f, -0.5f, 0.0f), listOf(0.0f, 0.0f)), (listOf(0.5f, -0.5f, 0.0f), listOf(1.0f, 0.0f)),) {
    // Override only what the mesh actually carries; NORMAL / COLOR0 / UV1 /
    // TANGENT use their identity defaults.
    mesh.addVertex(Vertex.pbr(pos).set(Vertex.UV0, uv))
}