from fragmentcolor import Mipmap, Renderer, TextureFormat
png = open("examples/assets/logo.png", "rb").read()

# Imagine `png` came off your asset loader on a worker thread.

# Decode + mipmap generation. Pure CPU; run it wherever you like.
chain = Mipmap.build(png, TextureFormat.Rgba8UnormSrgb)

# Back on the renderer thread, the upload is just a GPU write.
renderer = Renderer()
texture = renderer.create_texture(chain)