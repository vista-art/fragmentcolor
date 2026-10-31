import org.fragmentcolor.*
val image = java.io.File("examples/assets/logo.png").readBytes()

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64u, 64u))

val mesh = Mesh()
mesh.addVertices(listOf(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), Vertex.pbr(listOf(-0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(0.0f, 0.0f)), Vertex.pbr(listOf(0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(1.0f, 0.0f))))

// The material keeps the encoded image as a map.
val material = Material.pbr().baseColorTexture(image)
val model = Model(mesh, material)
val scene = Scene()
scene.add(model)

// Decode and upload it now, so the first render only draws.
renderer.load(scene)
renderer.render(scene, target)

// A second Renderer draws the same map from a copy of its own.
val other = Renderer()
val frame = other.createTextureTarget(listOf(64u, 64u))
other.load(scene)
other.render(scene, frame)