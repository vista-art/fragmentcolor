import org.fragmentcolor.*

val renderer = Renderer()

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, floatArrayOf(0.5f, 1.0f)), )
val model = Model(mesh, Material.pbr()?)

// A backdrop pass that clears to a soft blue before the scene's main draw.
val backdrop = Pass("backdrop")
backdrop.setClearColor(listOf(0.05f, 0.08f, 0.12f, 1.0f))

val scene = Scene()
scene.addPass(backdrop)
scene.add(model)