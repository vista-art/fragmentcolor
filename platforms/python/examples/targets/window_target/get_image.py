from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Renderer, Shader

renderer = Renderer()
target = renderer.create_texture_target([16, 16])
renderer.render(Shader(""), target)

image = target.get_image()
