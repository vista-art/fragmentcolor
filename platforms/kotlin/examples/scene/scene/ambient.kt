import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set("uv0", floatArrayOf(0.5f, 1.0f)), )

val scene = Scene()
// Warm dusk ambient — applies to every Material added below.
scene.ambient(listOf(0.06f, 0.04f, 0.03f))
scene.add(Model(mesh, Material.pbr()))
scene.add(Light.directional(listOf(0.3f, -1.0f, -0.4f), listOf(1.0f, 0.95f, 0.9f)))

renderer.render(scene, target)