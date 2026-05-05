import org.fragmentcolor.*

val m = Mesh()
val red = listOf(1.0f, 0.0f, 0.0f, 1.0f)
m.addInstance(Instance().set("tint", red))
m.clearInstances(); // back to a single uninstanced draw