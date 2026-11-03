import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64u, 64u))

val mesh = Mesh()
mesh.addVertices(listOf(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), Vertex.pbr(listOf(-0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(0.0f, 0.0f)), Vertex.pbr(listOf(0.5f, -0.5f, 0.0f)).set(Vertex.UV0, listOf(1.0f, 0.0f))))

val scene = Scene()
// A warm dusk ambient. It applies to every Material added below.
scene.ambient(listOf(0.06f, 0.04f, 0.03f))
scene.add(Model(mesh, Material.pbr()))
scene.add(Light.directional(listOf(0.3f, -1.0f, -0.4f), listOf(1.0f, 0.95f, 0.9f)))

renderer.render(scene, target)