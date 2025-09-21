
from fragmentcolor import Renderer
r = Renderer()
tex = r.create_storage_texture([64, 64], wgpu.TextureFormat.Rgba8Unorm, None)
