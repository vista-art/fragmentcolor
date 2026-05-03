import org.fragmentcolor.*

val m = Mesh()
val offset = listOf(0.25f, 0.10f)
val tint = listOf(1.0f, 0.0f, 0.0f, 1.0f)
m.addInstance(Instance().set("offset", offset).set("tint", tint))