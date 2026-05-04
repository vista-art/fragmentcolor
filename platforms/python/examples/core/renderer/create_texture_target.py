from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Renderer, Shader
renderer = Renderer()

# Create an offscreen texture target with a size of 64x64 pixels.
target = renderer.create_texture_target([64, 64])

renderer.render(Shader(""), target)

# get the rendered image
image = target.get_image()
