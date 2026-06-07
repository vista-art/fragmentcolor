import org.fragmentcolor.*

val renderer = Renderer()
val ao_bytes = listOf(220.0f, 0.0f, 0.0f, 255.0f, 180.0f, 0.0f, 0.0f, 255.0f, 200.0f, 0.0f, 0.0f, 255.0f, 160.0f, 0.0f, 0.0f, 255.0f, .0f)
val ao = renderer.createTexture(TextureInputMobile.Bytes(ao_bytes.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)
val mat = Material.pbr().occlusionTexture(ao)