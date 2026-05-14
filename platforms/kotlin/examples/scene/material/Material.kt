import org.fragmentcolor.*

val renderer = Renderer()
val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, floatArrayOf(0.5f, 1.0f)), )

val material = Material.pbr()?.baseColor(listOf(0.85f, 0.2f, 0.2f, 1.0f)).metallic(0.0).roughness(0.4).emissive(listOf(0.0f, 0.0f, 0.05f))

val model = Model(mesh, material)