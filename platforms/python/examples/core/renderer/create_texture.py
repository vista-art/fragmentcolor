from fragmentcolor import Renderer
renderer = Renderer()
# Load encoded image bytes (PNG/JPEG) or use a file path
image = open("logo.png", "rb").read()
tex = renderer.create_texture(image)