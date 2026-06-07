import org.fragmentcolor.*

val png: ByteArray = byteArrayOf()
// Imagine """png""" came off your asset loader on a worker thread.

// Decode + mipmap generation. Pure CPU; run it wherever you like.
val chain = Mipmap.build(png, TextureFormat.RGBA8_UNORM_SRGB, null)

// Back on the renderer thread, the upload is just a GPU write.
val renderer = Renderer()
val texture = renderer.createTexture(TextureInputMobile.Prepared(chain), null)