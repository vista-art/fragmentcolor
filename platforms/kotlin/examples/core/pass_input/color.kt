import org.fragmentcolor.*

val pass = Pass("background")
pass.setClearColor(listOf(1.0f, 0.0f, 0.0f, 1.0f))
val color = pass.getInput().color()