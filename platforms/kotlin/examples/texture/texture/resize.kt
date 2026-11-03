import org.fragmentcolor.*

val renderer = Renderer()
val depth = renderer.createDepthTexture(listOf(800u, 600u))
val pass = Pass("scene")
pass.setDepthTarget(depth)

// The window grew. The pass keeps the same depth texture at the new size.
depth.resize(listOf(1024u, 768u))