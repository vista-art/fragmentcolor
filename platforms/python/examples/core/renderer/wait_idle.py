from fragmentcolor import Renderer, Shader

r = Renderer()
target = r.create_texture_target([8, 8])
shader = Shader.default()
r.render(shader, target)
r.wait_idle()
_bytes = target.get_image()