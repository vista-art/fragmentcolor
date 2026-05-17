import org.fragmentcolor.*

val lamp = Light.point(listOf(0.0f, 0.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
lamp.setPosition(listOf(3.0f, 1.5f, -2.0f))

// Directional lights have no position — the call errors.
val sun = Light.directional(listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
val result = sun.setPosition(listOf(0.0f, 0.0f, 0.0f))