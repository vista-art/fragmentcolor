from fragmentcolor import Renderer
image = open("examples/assets/logo.png", "rb").read()
renderer = Renderer()
tex = renderer.create_texture(image)