import org.fragmentcolor.*

val renderer = Renderer()
val albedo_bytes = listOf(255.0f, 200.0f, 120.0f, 255.0f, 255.0f, 240.0f, 180.0f, 255.0f, 230.0f, 180.0f, 100.0f, 255.0f, 255.0f, 220.0f, 150.0f, 255.0f, .0f)
val albedo = renderer.createTexture(TextureInputMobile.Bytes(albedo_bytes.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)

// Every Material that points at """albedo""" reuses the same uploaded GPU
// texture; passing the same handle into N Material instances costs one
// upload and N shader-uniform references.
val blob = Material.pbr().baseColorTexture(albedo)