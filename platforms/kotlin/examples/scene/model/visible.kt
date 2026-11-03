import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertices(listOf(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)), Vertex.pbr(listOf(-0.5f, -0.5f, 0.0f)), Vertex.pbr(listOf(0.5f, -0.5f, 0.0f))))
val model = Model(mesh, Material.pbr())

// Models start visible; toggle with `set_visible`.
val visible_now = model.visible()