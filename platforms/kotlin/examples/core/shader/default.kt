
import org.fragmentcolor.*

val shader = Shader.default()

// The default shader includes a "resolution" uniform
shader.set("resolution", listOf(800.0f, 600.0f))
val res = shader.get("resolution")