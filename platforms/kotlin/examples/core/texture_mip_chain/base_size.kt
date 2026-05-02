import org.fragmentcolor.*

val pixels = Array(16 * 16 * 4) { 0 }
val chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    arrayOf(16, 16),
))
val (width, height) = chain.baseSize()
val _ = (width, height)