import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)))
val model = Model(mesh, Material.pbr()?)

// Models start visible; toggle with """set_visible""".
val visible_now = model.visible()