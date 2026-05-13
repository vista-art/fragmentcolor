import org.fragmentcolor.*

val sun = Light.directional(listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
// Reorient to a late-afternoon angle.
sun.setDirection(listOf(0.7f, -0.5f, -0.5f))