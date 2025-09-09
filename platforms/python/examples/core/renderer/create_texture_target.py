
from fragmentcolor import Renderer, Shader, Target
renderer = Renderer()

// Create an offscreen texture target with a size of 64x64 pixels.
target = renderer.create_texture_target([64, 64])

renderer.render(Shader.default(), target)
image = target.get_image(); // get the rendered image
