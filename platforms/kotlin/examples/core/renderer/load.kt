import org.fragmentcolor.*
val image = java.io.File("examples/assets/logo.png").readBytes()

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64u, 64u))

val mesh = Mesh()
mesh.addVertex(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)))

// The material keeps the encoded image as a pending upload.
val material = Material.pbr().baseColorTexture(image)
val model = Model(mesh, material)
val scene = Scene()
scene.add(model)

// Decode and upload it now, so the first render only draws.
renderer.load(scene)
renderer.render(scene, target)