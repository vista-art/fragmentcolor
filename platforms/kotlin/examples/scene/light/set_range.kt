import org.fragmentcolor.*

val lamp = Light.point(listOf(0.0f, 2.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
lamp.setRange(8.0)
val negative = lamp.setRange(-1.0)

// Directional lights have no range — the call errors.
val sun = Light.directional(listOf(0.0f, -1.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))
val unsupported = sun.setRange(5.0)