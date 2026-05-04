import org.fragmentcolor.*

// Encoded path — single tuple, no extra method.
val encoded_png_bytes: ByteArray = byteArrayOf()
val chain = TextureMipChain.prepare(encoded_png_bytes, TextureFormat.RGBA8_UNORM_SRGB, null)

// Raw pixel path — same method, just include the size in the tuple.
val raw_rgba: ByteArray = ByteArray(8 * 8 * 4)
val chain_raw = TextureMipChain.prepare(raw_rgba, TextureFormat.RGBA8_UNORM_SRGB, Size(width=8u, height=8u, depth=null))

// Hand the chain to the unified create_texture entry — same vocabulary.
val renderer = Renderer()
val texture = renderer.createTexture(TextureInputMobile.Prepared(chain), null)