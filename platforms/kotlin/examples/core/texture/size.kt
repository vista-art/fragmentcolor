import org.fragmentcolor.*
val renderer = Renderer()
val pixels = listOf(255.0f, 255.0f, 255.0f, 255.0f)
val tex = renderer.createTexture(TextureInputMobile.Bytes(pixels.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)
val sz = tex.size()