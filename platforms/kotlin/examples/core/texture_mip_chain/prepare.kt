import org.fragmentcolor.*

// Encoded path: pass bytes plus the format you expect.
val encoded_png_bytes: ByteArray = byteArrayOf()
val chain = TextureMipChain.prepare(encoded_png_bytes, TextureFormat.RGBA8_UNORM_SRGB, null)

// Raw path: include the size so prepare skips decoding.
val raw_rgba: ByteArray = ByteArray(8 * 8 * 4)
val chain_raw = TextureMipChain.prepare(raw_rgba, TextureFormat.RGBA8_UNORM_SRGB, Size(width=8u, height=8u, depth=null))

// Upload the chain through the regular create_texture entry point.
val renderer = Renderer()
val texture = renderer.createTexture(TextureInputMobile.Prepared(chain), null)