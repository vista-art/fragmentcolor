import org.fragmentcolor.*

val camera = Camera.perspective(1.047f, 16.0f / 9.0f, 0.1f, 100.0f).lookAt(listOf(0.0f, 0.0f, 5.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))

val view_proj = camera.viewProj()