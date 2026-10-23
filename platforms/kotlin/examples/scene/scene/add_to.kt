import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), )
val model = Model(mesh, Material.pbr())

val scene = Scene()
scene.addPass(Pass("geometry"))

// Target the pass by name (or pass its index: scene.addTo(0u, model)).
scene.addTo("geometry", model)