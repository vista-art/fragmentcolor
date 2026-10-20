import org.fragmentcolor.*
val encoded_png_bytes: ByteArray = byteArrayOf()
val raw_rgba = ByteArray(8 * 8 * 4) { 200.toByte() }

// Encoded path: bytes plus the format you want the chain to live in.
// The dimensions come from the decoded image.
val chain = Mipmap.build(encoded_png_bytes, TextureFormat.RGBA8_UNORM_SRGB, null)

// Raw path: include the size so build skips decoding.
val chain_raw = Mipmap.build(raw_rgba, TextureFormat.RGBA8_UNORM_SRGB, Size(width=8u, height=8u, depth=null))

// Either chain uploads the same way.
val renderer = Renderer()
val texture = renderer.createTexture(chain)