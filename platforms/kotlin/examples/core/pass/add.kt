import org.fragmentcolor.*

val renderer = Renderer()

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set("uv0", floatArrayOf(0.5f, 1.0f)), )
val model = Model(mesh, Material.pbr())

val camera = Camera.perspective(1.047f, 1.0f, 0.1f, 100.0f).lookAt(listOf(0.0f, 0.0f, 2.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))
val sun = Light.directional(listOf(0.3f, -1.0f, -0.4f), listOf(1.0f, 0.95f, 0.9f))

val pass = Pass("scene")
pass.add(model)
pass.add(camera)
pass.add(sun)

// Updating the camera later is enough — every Model already on the pass
// picks the view_proj up at the next render.
camera.lookAt(listOf(3.0f, 1.0f, 5.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))