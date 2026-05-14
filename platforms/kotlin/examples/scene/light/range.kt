import org.fragmentcolor.*

val bulb = Light.point(listOf(0.0f, 0.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f)).setRange(8.0)
val cutoff = bulb.range()