import org.fragmentcolor.*

val torch = Light.spot(listOf(0.0f, 1.8f, 1.0f), listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f)).setConeAngles(0.15f, 0.4f)
val sun = Light.directional(listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))