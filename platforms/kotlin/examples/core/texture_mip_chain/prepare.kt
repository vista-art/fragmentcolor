import org.fragmentcolor.*

// Encoded path â single tuple, no extra method.
val chain = TextureMipChain.prepare((encoded_png_bytes, TextureFormat.Rgba8UnormSrgb))

// Raw pixel path â same method, just include the size in the tuple.
val chain_raw = TextureMipChain.prepare((
    raw_rgba.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    arrayOf(8, 8),
))

// Hand the chain to the unified create_texture entry â same vocabulary.
val renderer = Renderer()
val texture = renderer.createTexture(chain)