import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set("uv0", floatArrayOf(0.5f, 1.0f)), )

val model = Model(mesh, Material.pbr())
model.mesh().addVertex( Vertex(listOf(-0.5f, -0.5f, 0.0f)).set("normal", floatArrayOf(0.0f, 0.0f, 1.0f)).set("uv0", listOf(0.0f, 0.0f)), )