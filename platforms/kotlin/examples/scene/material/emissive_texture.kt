import org.fragmentcolor.*

val renderer = Renderer()
val glow_bytes = listOf(255.0f, 0.0f, 0.0f, 255.0f, 255.0f, 0.0f, 0.0f, 255.0f, 255.0f, 0.0f, 0.0f, 255.0f, 255.0f, 0.0f, 0.0f, 255.0f, .0f)
val glow = renderer.createTexture(TextureInputMobile.Bytes(glow_bytes.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)
val mat = Material.pbr().emissive(listOf(0.8f, 0.0f, 0.0f)).emissiveTexture(glow)