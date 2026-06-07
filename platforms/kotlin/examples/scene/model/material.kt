import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex(listOf(0.0f, 0.0f, 0.0f)).set("normal", floatArrayOf(0.0f, 1.0f, 0.0f)).set("uv0", listOf(0.0f, 0.0f)), )

val model = Model(mesh, Material.pbr())
model.material().shader().set("camera.position", floatArrayOf(0.0f, 0.0f, 5.0f))