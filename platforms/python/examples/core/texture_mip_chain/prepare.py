from fragmentcolor import Renderer, TextureFormat, TextureMipChain

# Raw pixel path -- positional args: prepare(bytes, format, size).
raw_rgba = [200] * (8 * 8 * 4)
chain = TextureMipChain.prepare(raw_rgba, TextureFormat.Rgba8UnormSrgb, [8, 8])

# Encoded path (no size) -- prepare decodes the image and infers dimensions.
chain_raw = TextureMipChain.prepare(raw_rgba, TextureFormat.Rgba8UnormSrgb, [8, 8])

# Hand the chain to the unified create_texture entry -- same vocabulary.
renderer = Renderer()
texture = renderer.create_texture(chain)
