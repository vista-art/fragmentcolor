import org.fragmentcolor.*

val renderer = Renderer()
val normal_map_bytes = listOf(128.0f, 128.0f, 255.0f, 255.0f, 128.0f, 128.0f, 255.0f, 255.0f, 128.0f, 128.0f, 255.0f, 255.0f, 128.0f, 128.0f, 255.0f, 255.0f, .0f)
val normal_map = renderer.createTexture(TextureInputMobile.Bytes(normal_map_bytes.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)
val mat = Material.pbr().normalTexture(normal_map).normalScale(1.2f)