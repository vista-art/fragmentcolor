import org.fragmentcolor.*

val pass = Pass("overlay")
pass.loadPrevious()

val input = pass.getInput()
val keeps = input.loadsPrevious()