import org.fragmentcolor.*

val m = Mesh()
val red = arrayOf(1.0, 0.0, 0.0, 1.0)
val green = arrayOf(0.0, 1.0, 0.0, 1.0)
val blue = arrayOf(0.0, 0.0, 1.0, 1.0)
m.addInstances([
    Instance.new().set("tint", red),
    Instance.new().set("tint", green),
    Instance.new().set("tint", blue),
])