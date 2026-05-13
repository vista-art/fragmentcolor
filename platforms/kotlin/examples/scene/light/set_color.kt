import org.fragmentcolor.*

val lamp = Light.directional(listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
// Warm-tinted bulb after the user toggles the warm-light switch.
lamp.setColor(listOf(1.0f, 0.85f, 0.7f))