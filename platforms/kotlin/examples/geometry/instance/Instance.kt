import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex(listOf(0.0f, 0.0f))

val instance = Instance().set("offset", listOf(0.5f, 0.0f))
mesh.addInstance(instance)