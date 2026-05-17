import org.fragmentcolor.*

val sun = Light.directional(listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
sun.setDirection(listOf(0.3f, -0.8f, -0.5f))

// Point lights have no direction — the call errors.
val lamp = Light.point(listOf(0.0f, 2.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
val result = lamp.setDirection(listOf(0.0f, -1.0f, 0.0f))