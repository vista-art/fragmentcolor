import org.fragmentcolor.*

val pixels = Array(8 * 8 * 4) { 0 }
val chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    arrayOf(8, 8),
))
val count = chain.levelCount()
val _ = count