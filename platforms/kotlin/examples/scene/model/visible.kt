import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)))
val model = Model(mesh, Material.pbr()?)
