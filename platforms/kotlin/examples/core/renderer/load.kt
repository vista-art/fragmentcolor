import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64, 64))

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), )
// Raw 2×2 RGBA pixel bytes — uploaded lazily by `Renderer.load` below.
 
// vocabulary covers all of them.
val red_pixels = listOf(255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255)
val red_tex = renderer.createTexture(red_pixels, listOf(2, 2))
val material = Material.pbr().baseColorTexture(red_tex)
val model = Model(mesh, material)
val scene = Scene()
scene.add(model)

// Eager prewarm — uploads the pending texture(s) so the next render is
// GPU-only.
renderer.load(scene)
renderer.render(scene, target)