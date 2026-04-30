from fragmentcolor import Renderer, TextureFormat

r = Renderer()
seed = [0] * (8 * 8 * 4)
tex = r.create_storage_texture_with_data([8, 8], TextureFormat.Rgba, seed, None)
