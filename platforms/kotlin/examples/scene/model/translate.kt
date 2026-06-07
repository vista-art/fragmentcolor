import org.fragmentcolor.*

val renderer = Renderer()
val mesh = Mesh()
mesh.addVertex( Vertex(listOf(0.0f, 0.0f, 0.0f)).set("normal", floatArrayOf(0.0f, 1.0f, 0.0f)).set("uv0", listOf(0.0f, 0.0f)), )

val model = Model(mesh, Material.pbr())
model.translate(listOf(5.0f, 0.0f, -2.0f))