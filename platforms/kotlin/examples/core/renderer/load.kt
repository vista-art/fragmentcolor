import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

val mesh = Mesh()
mesh.addVertex( Vertex.new(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.NORMAL, floatArrayOf(0.0f, 0.0f, 1.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)).set(Vertex.COLOR0, listOf(1.0f, 1.0f, 1.0f, 1.0f)).set(Vertex.UV1, listOf(0.0f, 0.0f)), )
// Raw 2×2 RGBA pixel bytes — uploaded lazily by """Renderer.load""" below.
 
// vocabulary covers all of them.
val red_pixels = listOf(255.0f, 0.0f, 0.0f, 255.0f, 0.0f, 255.0f, 0.0f, 255.0f, 0.0f, 0.0f, 255.0f, 255.0f, 255.0f, 255.0f, 255.0f, 255.0f, .0f)
val material = Material.pbr()?.baseColorTexture((red_pixels, listOf(2.0f, 2.0f)))
val model = Model(mesh, material)
val scene = Scene()
scene.add(model)

// Eager prewarm — uploads the pending texture(s) so the next render is
// GPU-only.
renderer.load(scene)
renderer.render(scene, target)