import org.fragmentcolor.*

val sun = Light.directional(listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
val bulb = Light.point(listOf(0.0f, 2.5f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
val torch = Light.spot(listOf(0.0f, 1.8f, 1.0f), listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))