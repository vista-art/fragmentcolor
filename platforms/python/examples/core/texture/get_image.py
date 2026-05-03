from fragmentcolor import Renderer, TextureFormat
renderer = Renderer()
texture = renderer.create_storage_texture([64, 64], TextureFormat.Rgba)
texture.write([0] * (64 * 64 * 4))

bytes = texture.get_image()
