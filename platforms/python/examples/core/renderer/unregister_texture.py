from fragmentcolor import Renderer, TextureFormat
renderer = Renderer()
id = *renderer
    .create_storage_texture([16, 16], TextureFormat.Rgba, None)
    
    .id()

renderer.unregister_texture(id)