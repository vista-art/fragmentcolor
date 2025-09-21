from fragmentcolor import Renderer
renderer = Renderer()
# Load encoded image bytes (PNG/JPEG) or use a file path
bytes = std.fs.read("./examples/assets/image.png")
tex = renderer.create_texture(bytes)