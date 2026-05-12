import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex.new(listOf(0.0f, 0.0f, 0.0f)).set(Vertex.NORMAL, floatArrayOf(0.0f, 1.0f, 0.0f)).set(Vertex.UV0, listOf(0.0f, 0.0f)), )

val model = Model(mesh, Material.pbr())
model.translate(listOf(5.0f, 0.0f, -2.0f))