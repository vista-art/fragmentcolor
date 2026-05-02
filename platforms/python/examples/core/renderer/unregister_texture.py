from fragmentcolor import Renderer, TextureFormat

renderer = Renderer()
texture = renderer.create_storage_texture([16, 16], TextureFormat.Rgba)
id = texture.id()

renderer.unregister_texture(id)