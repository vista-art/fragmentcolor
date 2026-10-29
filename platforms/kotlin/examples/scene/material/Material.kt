import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertices(listOf(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), Vertex.pbr(listOf(-0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(0.0f, 0.0f)), Vertex.pbr(listOf(0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(1.0f, 0.0f))))

val material = Material.pbr().baseColor(listOf(0.85f, 0.2f, 0.2f, 1.0f)).metallic(0.0f).roughness(0.4f).emissive(listOf(0.0f, 0.0f, 0.05f))

val model = Model(mesh, material)