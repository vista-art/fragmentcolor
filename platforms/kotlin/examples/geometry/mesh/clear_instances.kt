import org.fragmentcolor.*

val m = Mesh()
val red = arrayOf(1.0, 0.0, 0.0, 1.0)
m.addInstance(Instance.new().set("tint", red))
m.clearInstances(); // back to a single uninstanced draw