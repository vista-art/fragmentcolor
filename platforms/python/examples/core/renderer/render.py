
from fragmentcolor import Renderer, Shader

renderer = Renderer()
target = renderer.create_texture_target([10, 10])
shader = Shader.default()

renderer.render(shader, target)
