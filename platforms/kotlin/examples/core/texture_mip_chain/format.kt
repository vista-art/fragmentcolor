import org.fragmentcolor.*

val pixels = Array(4 * 4 * 4) { 200 }
val chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    arrayOf(4, 4),
))
val _ = chain.format()