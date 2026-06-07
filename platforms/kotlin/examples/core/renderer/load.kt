import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set("uv0", floatArrayOf(0.5f, 1.0f)), )
val bytes: ByteArray = byteArrayOf()
// Raw 2×2 RGBA pixel bytes — uploaded lazily by """Renderer.load""" below.
 
// vocabulary covers all of them.
val red_pixels = listOf(255.0f, 0.0f, 0.0f, 255.0f, 0.0f, 255.0f, 0.0f, 255.0f, 0.0f, 0.0f, 255.0f, 255.0f, 255.0f, 255.0f, 255.0f, 255.0f, .0f)
val red_tex = renderer.createTexture(TextureInputMobile.Bytes(red_pixels.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)
val material = Material.pbr().baseColorTexture(red_tex)
val model = Model(mesh, material)
val scene = Scene()
scene.add(model)

// Eager prewarm — uploads the pending texture(s) so the next render is
// GPU-only.
renderer.load(scene)
renderer.render(scene, target)