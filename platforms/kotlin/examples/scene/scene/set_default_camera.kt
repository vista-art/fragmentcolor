import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), )
val scene = Scene()
scene.add(Model(mesh, Material.pbr()))

val camera = Camera.perspective(1.047f, 16.0f / 9.0f, 0.1f, 100.0f).lookAt(listOf(0.0f, 1.5f, 4.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))
scene.setDefaultCamera(camera)