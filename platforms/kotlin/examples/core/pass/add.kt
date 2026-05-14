import org.fragmentcolor.*

val renderer = Renderer()

val mesh = Mesh()
mesh.addVertex( Vertex.new(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.NORMAL, floatArrayOf(0.0f, 0.0f, 1.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)).set(Vertex.COLOR0, listOf(1.0f, 1.0f, 1.0f, 1.0f)).set(Vertex.UV1, listOf(0.0f, 0.0f)), )
val model = Model(mesh, Material.pbr()?)

val camera = Camera.perspective(60.0.toRadians(), 1.0, 0.1, 100.0).lookAt(listOf(0.0f, 0.0f, 2.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))
val sun = Light.directional(listOf(0.3f, -1.0f, -0.4f), listOf(1.0f, 0.95f, 0.9f))

val pass = Pass("scene")
pass.add(model)?.add(camera)?.add(sun)

// Updating the camera later is enough — every Model already on the pass
// picks the view_proj up at the next render.
camera.lookAt(listOf(3.0f, 1.0f, 5.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))