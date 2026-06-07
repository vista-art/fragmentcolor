import org.fragmentcolor.*

val renderer = Renderer()
val mr_map_bytes = listOf(0.0f, 200.0f, 50.0f, 255.0f, 0.0f, 240.0f, 80.0f, 255.0f, 0.0f, 180.0f, 30.0f, 255.0f, 0.0f, 220.0f, 60.0f, 255.0f, .0f)
val mr_map = renderer.createTexture(TextureInputMobile.Bytes(mr_map_bytes.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)
val mat = Material.pbr().metallicRoughnessTexture(mr_map)