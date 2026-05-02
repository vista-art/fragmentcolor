from fragmentcolor import Renderer, TextureFormat, TextureMipChain

# Encoded path â single tuple, no extra method.
chain = TextureMipChain.prepare((encoded_png_bytes, TextureFormat.Rgba8UnormSrgb))

# Raw pixel path â same method, just include the size in the tuple.
chain_raw = TextureMipChain.prepare((
    raw_rgba.as_slice(),
    TextureFormat.Rgba8UnormSrgb,
    [8, 8],
))

# Hand the chain to the unified create_texture entry â same vocabulary.
renderer = Renderer()
texture = renderer.create_texture(chain)