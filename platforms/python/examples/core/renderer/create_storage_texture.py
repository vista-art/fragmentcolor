from fragmentcolor import Renderer, TextureFormat

r = Renderer()
# Empty storage texture -- positional args: (size, format).
tex = r.create_storage_texture([64, 64], TextureFormat.Rgba)

# Pre-seeded with bytes -- same method, optional data kwarg.
pixels = [0] * (64 * 64 * 4)
tex2 = r.create_storage_texture([64, 64], TextureFormat.Rgba, data=pixels)
