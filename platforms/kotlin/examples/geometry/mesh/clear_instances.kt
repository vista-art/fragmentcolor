import org.fragmentcolor.*

val m = Mesh()
m.addInstance(Instance().set("tint", listOf(1.0f, 0.0f, 0.0f, 1.0f)))
m.clearInstances() // back to a single uninstanced draw