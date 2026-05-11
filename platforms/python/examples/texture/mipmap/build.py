from fragmentcolor import Renderer, TextureFormat, Mipmap

# Raw pixel path -- positional args: prepare(bytes, format, size).
raw_rgba = [200] * (8 * 8 * 4)
chain = Mipmap.build(raw_rgba, TextureFormat.Rgba8UnormSrgb, [8, 8])

# Hand the chain to the unified create_texture entry.
renderer = Renderer()
texture = renderer.create_texture(chain)