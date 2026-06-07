
import org.fragmentcolor.*

val renderer = Renderer()
val bytes: ByteArray = byteArrayOf()
// 1x1 RGBA (white) raw pixel bytes
val pixels = listOf(255.0f, 255.0f, 255.0f, 255.0f)
val tex = renderer.createTexture(TextureInputMobile.Bytes(pixels.let { ba -> ByteArray(ba.size) { i -> ba[i].toInt().and(0xFF).toByte() } }), null)
val a = tex.aspect()