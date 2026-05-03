import org.fragmentcolor.*

val m = Mesh()
val red = listOf(1.0f, 0.0f, 0.0f, 1.0f)
val green = listOf(0.0f, 1.0f, 0.0f, 1.0f)
val blue = listOf(0.0f, 0.0f, 1.0f, 1.0f)
m.addInstances(listOf(Instance().set("tint", red), Instance().set("tint", green), Instance().set("tint", blue),))