import org.fragmentcolor.*

val renderer = Renderer()
// Encoded image bytes the caller has on hand (could come off a worker).
val png = byteArrayOf(0x89.toByte(), 0x50.toByte(), 0x4E.toByte(), 0x47.toByte(), 0x0D.toByte(), 0x0A.toByte(), 0x1A.toByte(), 0x0A.toByte())
val chain = TextureMipChain.prepare(png, TextureFormat.RGBA8_UNORM_SRGB, null)

// Hand the chain to the unified create_texture entry - same vocabulary as
// every other texture path; From<TextureMipChain> selects the GPU-only
// upload internally.
val texture = renderer.createTexture(TextureInputMobile.Prepared(chain), null)