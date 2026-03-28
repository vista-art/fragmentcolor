from fragmentcolor import Renderer, TextureFormat

renderer = Renderer()
texture = renderer.create_storage_texture([64, 64], TextureFormat.Rgba, None)
frame = bytes(64 * 64 * 4)

texture.write(frame)