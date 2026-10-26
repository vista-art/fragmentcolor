from fragmentcolor import Renderer, Shader

renderer = Renderer()
target = renderer.create_texture_target([16, 16])
shader = Shader.default()

# A GPU error in this frame returns as a DeviceError.
renderer.render(shader, target)