import org.fragmentcolor.*

val m = Mesh()
val offset = arrayOf(0.25, 0.10)
val tint = arrayOf(1.0, 0.0, 0.0, 1.0)
m.addInstance(Instance.new().set("offset", offset).set("tint", tint))