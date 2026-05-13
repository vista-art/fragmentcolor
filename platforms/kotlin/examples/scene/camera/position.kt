import org.fragmentcolor.*

val camera = Camera.perspective(60.0.toRadians(), 16.0 / 9.0, 0.1, 100.0).lookAt(listOf(3.0f, 2.0f, 8.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))

val eye = camera.position()