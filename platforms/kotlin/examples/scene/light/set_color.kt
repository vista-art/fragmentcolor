import org.fragmentcolor.*

val lamp = Light.point(listOf(0.0f, 2.0f, 0.0f), listOf(1.0f, 1.0f, 1.0f))

// Warm-tint the lamp later — every Pass that absorbed """lamp""" sees the
// color on the next render.
lamp.setColor(listOf(1.0f, 0.7f, 0.4f))