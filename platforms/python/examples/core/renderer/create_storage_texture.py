from fragmentcolor import Renderer, TextureFormat

r = Renderer()
# Empty storage texture â same single create_storage_texture entry.
tex = r.create_storage_texture(([64, 64], TextureFormat.Rgba))

# Pre-seeded with bytes â same method, three-tuple form.
pixels = [0] * (64 * 64 * 4)
tex2 = r.create_storage_texture(([64, 64], TextureFormat.Rgba, pixels))