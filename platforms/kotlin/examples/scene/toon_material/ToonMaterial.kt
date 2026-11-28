import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertices(listOf(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)), Vertex.pbr(listOf(-0.5f, -0.5f, 0.0f)), Vertex.pbr(listOf(0.5f, -0.5f, 0.0f))))

val hero = ToonMaterial().baseColor(listOf(0.9f, 0.3f, 0.2f, 1.0f)).bands(2u).highlight(listOf(1.0f, 1.0f, 1.0f)).rim(listOf(0.3f, 0.5f, 1.0f))
val model = Model(mesh, hero)