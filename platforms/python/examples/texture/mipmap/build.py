from fragmentcolor import Mipmap, Renderer, TextureFormat
encoded_png_bytes = open("examples/assets/logo.png", "rb").read()
raw_rgba = bytes([200] * (8 * 8 * 4))

# Encoded path: bytes plus the format you want the chain to live in.
# The dimensions come from the decoded image.
chain = Mipmap.build(encoded_png_bytes, TextureFormat.Rgba8UnormSrgb)

# Raw path: include the size so build skips decoding.
chain_raw = Mipmap.build(raw_rgba, TextureFormat.Rgba8UnormSrgb, size=[8, 8])

# Either chain uploads the same way.
renderer = Renderer()
texture = renderer.create_texture(chain)